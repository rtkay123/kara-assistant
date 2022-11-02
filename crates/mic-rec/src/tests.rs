use cpal::traits::HostTrait;

#[test]
fn test_audio() {
    let host = cpal::default_host();
    assert!(cpal::Host::is_available());
    assert!(host.input_devices().is_ok());
    assert!(host.default_input_device().is_some());
}
