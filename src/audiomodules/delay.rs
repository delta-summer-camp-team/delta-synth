use crate::audiomodules::AudioModule;

pub struct Delay {
    buffer: Vec<f32>,
    write_pos: usize,
    sample_rate: f32,
    delay_time: f32, // в секундах
    feedback: f32,   // 0.0..1.0
    mix: f32,        // 0.0..1.0
}

impl Delay {
    pub fn new(sample_rate: f32, max_delay_sec: f32) -> Self {
        let buffer_len = (sample_rate * max_delay_sec).ceil() as usize + 1;
        Self {
            buffer: vec![0.0; buffer_len],
            write_pos: 0,
            sample_rate,
            delay_time: 0.5, // по умолчанию 500 мс
            feedback: 0.5,
            mix: 0.5,
        }
    }

    pub fn set_delay_time(&mut self, sec: f32) {
        self.delay_time = sec.clamp(0.0, self.buffer.len() as f32 / self.sample_rate);
    }

    pub fn set_feedback(&mut self, fb: f32) {
        self.feedback = fb.clamp(0.0, 0.99);
    }

    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

}
impl AudioModule for Delay {
    fn process(&mut self, input: &mut [f32]) {
    let delay_samples = self.delay_time * self.sample_rate;
    let buffer_len = self.buffer.len();

    for sample in input.iter_mut() {
        // Вычисляем позицию для чтения (с поддержкой дробной задержки)
        let read_pos = (self.write_pos as f32 - delay_samples + buffer_len as f32) % buffer_len as f32;
        let i0 = read_pos.floor() as usize;
        let i1 = (i0 + 1) % buffer_len;
        let frac = read_pos - i0 as f32;

        // Интерполяция
        let delayed_sample = (1.0 - frac) * self.buffer[i0] + frac * self.buffer[i1];

        // Сохраняем оригинал
        let input_sample = *sample;

        // Применяем микс
        let output = input_sample * (1.0 - self.mix) + delayed_sample * self.mix;

        // Записываем обратно в input (in-place обработка)
        *sample = output;

        // Обновляем буфер с учётом обратной связи
        self.buffer[self.write_pos] = input_sample + delayed_sample * self.feedback;

        // Продвигаем позицию записи
        self.write_pos = (self.write_pos + 1) % buffer_len;
    }
}
}