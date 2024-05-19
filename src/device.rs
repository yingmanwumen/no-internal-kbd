use anyhow::Result;

pub struct Device {
    name: String,
    sysname: String,
    udev: udev::Device,
    evdev: evdev::Device,
}

impl Device {
    pub fn from_sysname(sysname: String) -> Result<Self> {
        let udev = udev::Device::from_subsystem_sysname(String::from("input"), sysname.clone())?;
        let evdev = evdev::Device::open(format!("/dev/input/{}", sysname))?;
        let path = format!("/sys/class/input/{}/device/name", sysname);
        let name = std::fs::read_to_string(path)?.trim().to_string();
        Ok(Self {
            name,
            sysname,
            udev,
            evdev,
        })
    }

    pub fn is_keyboard_device(&self) -> bool {
        self.udev.property_value("ID_INPUT_KEYBOARD").is_some()
    }

    pub fn is_platform_device(&self) -> bool {
        self.udev.syspath().to_string_lossy().contains("platform")
    }

    pub fn grab(&mut self) -> Result<()> {
        Ok(self.evdev.grab()?)
    }

    pub fn ungrab(&mut self) -> Result<()> {
        Ok(self.evdev.ungrab()?)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn sysname(&self) -> &str {
        &self.sysname
    }
}

pub fn enumrate_input_devices() -> Result<impl Iterator<Item = Device>> {
    let dir = std::fs::read_dir("/dev/input/")?;
    Ok(dir
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|e| e.contains("event"))
        .filter_map(|sysname| Device::from_sysname(sysname).ok()))
}
