use crate::audiomodules::AudioModule;

pub struct ReverbEffect {
    // Параметры, которыми будет управлять пользователь
    dry_wet_mix: f32, // Баланс между сухим и обработанным сигналом (0.0 до 1.0)
    decay_time: f32,  // Время затухания
    pre_delay: DelayLine,
    early_reflections: EarlyReflections,
    late_reflections: LateReflections,
}
struct DelayLine {
    buffer: Vec<f32>,
    write_head: usize,
}
impl DelayLine {
    // Метод, который задерживает один сэмпл
    fn process_sample(&mut self, input: f32) -> f32 {
        let read_head = self.write_head; // Для простой задержки читаем то, что сейчас перезапишем
        let output = self.buffer[read_head];
        self.buffer[read_head] = input;
        self.write_head = (self.write_head + 1) % self.buffer.len();
        output
    }
}
struct EarlyReflections {
    delay_line: DelayLine, // Используем наш базовый блок
}

impl EarlyReflections {
    // Задаем параметры наших отражений (задержка в сэмплах, громкость)
    const TAPS: &'static [(usize, f32)] = &[
        (2345, 2.0), (3456, 2.8), (4567, 0.9), (5678, 1.2)
    ];

    fn process_sample(&mut self, input: f32) -> f32 {
        // Сначала записываем текущий сэмпл в линию задержки
        self.delay_line.process_sample(input); 

        // Теперь суммируем сигналы со всех отводов
        let mut output = 0.0;
        for (delay, gain) in Self::TAPS {
            let read_head = (self.delay_line.write_head + self.delay_line.buffer.len() - delay) % self.delay_line.buffer.len();
            output += self.delay_line.buffer[read_head] * gain;
        }
        output
    }
}
struct CombFilter {
    delay_line: DelayLine,
    feedback: f32, // Управляет временем затухания
}

impl CombFilter {
    fn process_sample(&mut self, input: f32) -> f32 {
        let delayed = self.delay_line.buffer[self.delay_line.write_head];
        let new_value = input + delayed * self.feedback;
        self.delay_line.process_sample(new_value);
        delayed // Выходом является задержанный сигнал
    }
}
struct AllPassFilter {
    delay_line: DelayLine,
    gain: f32, // Коэффициент "размазывания"
}

impl AllPassFilter {
    fn process_sample(&mut self, input: f32) -> f32 {
        let delayed = self.delay_line.buffer[self.delay_line.write_head];
        let output = -input * self.gain + delayed;
        let new_value_for_buffer = output * self.gain + input;
        self.delay_line.process_sample(new_value_for_buffer);
        output
    }
}
// "Оркестр" из эхо для создания хвоста
struct LateReflections {
    comb_filters: Vec<CombFilter>,
    all_pass_filters: Vec<AllPassFilter>,
}

impl LateReflections {
    fn new() -> Self {
        let comb_delays = [1557, 1617, 1491, 1422];
        // Длины задержек для all-pass фильтров
        let all_pass_delays = [225, 556];

        // Создаём гребенчатые фильтры с разными фидбэками
        let comb_filters = comb_delays
            .iter()
            .map(|&delay| CombFilter {
                delay_line: DelayLine {
                    buffer: vec![0.0; delay],
                    write_head: 0,
                },
                feedback: 0.78, // можно вынести как параметр, если нужно тонко настраивать
            })
            .collect();

        // Создаём all-pass фильтры
        let all_pass_filters = all_pass_delays
            .iter()
            .map(|&delay| AllPassFilter {
                delay_line: DelayLine {
                    buffer: vec![0.0; delay],
                    write_head: 0,
                },
                gain: 0.7,
            })
            .collect();

        Self {
            comb_filters,
            all_pass_filters,
        }
    }

    fn process_sample(&mut self, input: f32) -> f32 {
        // 1. Подаем звук на все гребенчатые фильтры параллельно и суммируем их выходы
        let mut comb_output = 0.0;
        for filter in &mut self.comb_filters {
            comb_output += filter.process_sample(input);
        }


        // 2. Пропускаем результат через цепочку фазовых фильтров для "размазывания"
        let mut final_output = comb_output;
        for filter in &mut self.all_pass_filters {
            final_output = filter.process_sample(final_output);
        }
        final_output
    }
}
impl AudioModule for ReverbEffect {
    fn process(&mut self, output: &mut [f32]) {
        for sample in output.iter_mut() {
            let dry_sample = *sample;

            // 1. Применяем предварительную задержку
            let delayed_sample = self.pre_delay.process_sample(dry_sample);

            // 2. Генерируем ранние и поздние отражения
            let early = self.early_reflections.process_sample(delayed_sample);
            let late = self.late_reflections.process_sample(delayed_sample);

            // 3. Смешиваем отражения
            let wet_sample = early + late;

            // 4. Смешиваем "сухой" и "мокрый" сигналы
            *sample = (dry_sample * (1.0 - self.dry_wet_mix)) + (wet_sample * self.dry_wet_mix);
        }
    }
}
impl ReverbEffect {
    pub fn new(dry_wet_mix: f32, decay_time: f32, sample_rate: usize) -> Self {
        // Длина предварительной задержки в сэмплах (например, 50 мс)
        let pre_delay_samples = (0.05 * sample_rate as f32) as usize;

        // Буфер для ранних отражений (берём с запасом)
        let early_reflections_buffer_len = 6000;

        Self {
            dry_wet_mix,
            decay_time,
            pre_delay: DelayLine {
                buffer: vec![0.0; pre_delay_samples],
                write_head: 0,
            },
            early_reflections: EarlyReflections {
                delay_line: DelayLine {
                    buffer: vec![0.0; early_reflections_buffer_len],
                    write_head: 0,
                },
            },
            late_reflections: LateReflections::new(),
        }
    }
}
