#[derive(Clone, Debug)]
pub enum DeviceTransport {
    Any(String),
    EmulatorAny(String),
    UsbAny(String),
    Usb(String),
}

impl DeviceTransport {
    pub fn default() -> Self {
        DeviceTransport::Any(String::from("host:transport-any"))
    }

    pub fn default_usb() -> Self {
        DeviceTransport::UsbAny(String::from("host:transport-usb"))
    }

    pub fn default_emulator() -> Self {
        DeviceTransport::EmulatorAny(String::from("host:transport-local"))
    }

    pub fn usb(serial: String) -> Self {
        DeviceTransport::Usb(format!("host:transport:{}", serial))
    }

    pub fn get_device_transport(&self) -> &str {
        match self {
            DeviceTransport::Any(s) => s,
            DeviceTransport::EmulatorAny(s) => s,
            DeviceTransport::UsbAny(s) => s,
            DeviceTransport::Usb(s) => s,
        }
    }
}