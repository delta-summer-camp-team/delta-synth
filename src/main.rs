use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, SupportedStreamConfig};

/// Инициализация аудиоустройства и конфигурации
fn init_audio_device() -> Option<(Device, SupportedStreamConfig)> {
    let host = cpal::default_host();
    let device = host.default_output_device()?;
    let config = device.default_output_config().ok()?;
    Some((device, config))
}

fn main() {
    let _ = init_audio_device();
}
 