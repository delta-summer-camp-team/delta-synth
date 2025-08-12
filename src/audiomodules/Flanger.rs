use crate::audiomodules::{AudioModule, Oscillator, Waveforma}; // Убедитесь, что пути верные

pub struct FlangerEffect {
    sample_rate: f32,
    delay_line: Vec<f32>,
    write_head: usize,
    lfo: Oscillator, // Осциллятор используется как LFO
    
    // Параметры эффекта
    min_delay_ms: f32,
    depth_ms: f32,
    feedback: f32,
}

impl FlangerEffect {
    pub fn new(sample_rate: f32, lfo_freq: f32, min_delay_ms: f32, depth_ms: f32, feedback: f32) -> Self {
        let max_delay_ms = min_delay_ms + depth_ms;
        let buffer_size = (sample_rate * max_delay_ms / 1000.0).ceil() as usize;

        Self {
            sample_rate,
            delay_line: vec![0.0; buffer_size],
            write_head: 0,
            // Создаем LFO. Амплитуда 1.0 важна для полного диапазона модуляции.
            lfo: Oscillator::new(lfo_freq, sample_rate, Waveforma::Sine, 1.0),
            min_delay_ms,
            depth_ms,
            feedback,
        }
    }
}

impl AudioModule for FlangerEffect {
    fn process(&mut self, output: &mut [f32]) {
        let buffer_len = output.len();
        
        // 1. Создаем временный буфер для значений LFO
        let mut lfo_buffer = vec![0.0; buffer_len];

        // 2. Заполняем этот буфер, вызвав process нашего LFO
        self.lfo.process(&mut lfo_buffer);
        
        // 3. Теперь проходим по основному буферу, используя значения из lfo_buffer
        for (i, sample) in output.iter_mut().enumerate() {
            let dry_sample = *sample;

            // Получаем значение LFO для текущего сэмпла из нашего временного буфера
            let lfo_out = lfo_buffer[i];
            
            // --- Остальная логика идентична предыдущей версии ---

            let lfo_mapped = (lfo_out + 1.0) / 2.0;
            let current_delay_ms = self.min_delay_ms + lfo_mapped * self.depth_ms;
            let delay_samples = (current_delay_ms / 1000.0 * self.sample_rate).floor() as usize;

            let read_head = (self.write_head + self.delay_line.len() - delay_samples) % self.delay_line.len();
            let delayed_sample = self.delay_line[read_head];

            let feedback_sample = delayed_sample * self.feedback;
            self.delay_line[self.write_head] = dry_sample + feedback_sample;
            
            *sample = (dry_sample + delayed_sample) * 0.5;

            self.write_head = (self.write_head + 1) % self.delay_line.len();
        }
    }
}