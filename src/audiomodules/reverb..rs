
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
        (2345, 0.78), (3456, 0.65), (4567, 0.55), (5678, 0.45)
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
        // Здесь мы создаем несколько гребенчатых и фазовых фильтров
        // с разными, тщательно подобранными (обычно простые числа) длинами задержек,
        // чтобы избежать "металлического" призвука.
        // ...
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
