use std::sync::{Arc, Mutex};
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig, SupportedStreamConfig};

/// Твой неизменённый init_audio_device
fn init_audio_device() -> Option<(Device, SupportedStreamConfig)> {
    let host = cpal::default_host();
    let device = host.default_output_device()?;
    let config = device.default_output_config().ok()?;
    Some((device, config))
}

/// Трейт аудиомодуля
trait AudioModule: Send {
    fn process(&mut self, buffer: &mut [f32]);
}

/// Пример модуля, который ничего не делает
struct DummyModule;
impl AudioModule for DummyModule {
    fn process(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = 0.0;
        }
    }
}

/// Функция создания и запуска аудиопотока
fn start_audio_stream(
    device: Device,
    config: StreamConfig,
    modules: Vec<Arc<Mutex<dyn AudioModule>>>,
) -> Stream {
    let stream = device
        .build_output_stream(
            &config,
            {
                let modules = modules.clone();
                move |data: &mut [f32], _| {
                    for module in &modules {
                        if let Ok(mut m) = module.lock() {
                            m.process(data);
                        }
                    }
                }
            },
            move |_err| {
                // Ошибки можно обработать здесь (пока игнорируем)
            },
            None,
        )
        .expect("Не удалось создать аудиопоток");
    stream
}

fn main() {
    // Используем твою функцию, не меняя её
    let (device, supported_config) = match init_audio_device() {
        Some(val) => val,
        None => return, // Если не получилось, просто завершаемся молча
    };
    let config = supported_config.config();

    let modules: Vec<Arc<Mutex<dyn AudioModule>>> = vec![
        Arc::new(Mutex::new(DummyModule)),
        // Добавляй свои модули сюда
    ];

    let stream = start_audio_stream(device, config, modules);
    stream.play().expect("Не удалось запустить поток");

    std::thread::park(); // Чтобы поток не завершился сразу
}