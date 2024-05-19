pub mod device;

use std::{collections::HashMap, os::fd::AsRawFd};

use anyhow::Result;
use device::Device;
use nix::unistd::Uid;
use tokio::io::unix::{AsyncFd, AsyncFdReadyGuard};

pub struct Context {
    pub platform_kbds: HashMap<String, Device>,
    pub external_kbds: HashMap<String, Device>,
}

impl Context {
    pub fn initialize() -> Result<Self> {
        // test root permissions
        if !Uid::effective().is_root() {
            anyhow::bail!("This program must be run as root");
        }
        let (platform_kbds, external_kbds) = device::enumrate_input_devices()?
            .filter(|device| device.is_keyboard_device())
            .map(|device| (device.sysname().to_string(), device))
            .partition(|(_, device)| device.is_platform_device());
        Ok(Self {
            platform_kbds,
            external_kbds,
        })
    }

    pub async fn start(mut self) -> ! {
        if !self.external_kbds.is_empty() {
            self.disable_platform_kbds();
        }

        let monitor = udev::MonitorBuilder::new()
            .unwrap()
            .match_subsystem("input")
            .unwrap()
            .listen()
            .unwrap();
        let fd = AsyncFd::new(monitor).unwrap();
        loop {
            poll_in(&fd, |guard| {
                for event in guard.get_inner().iter().filter(|event| {
                    event.sysname().to_string_lossy().contains("event")
                        && event.property_value("ID_INPUT_KEYBOARD").is_some()
                }) {
                    let sysname = event.sysname().to_string_lossy().to_string();
                    if let Some(action) = event.action() {
                        match action.to_string_lossy().to_string().as_str() {
                            "add" => self.on_external_device_added(sysname),
                            "remove" => self.on_external_device_removed(sysname),
                            _ => (),
                        }
                    }
                }
            })
            .await;
        }
    }

    fn on_external_device_added(&mut self, sysname: String) {
        self.external_kbds
            .insert(sysname.clone(), Device::from_sysname(sysname).unwrap());
        self.disable_platform_kbds();
    }

    fn on_external_device_removed(&mut self, sysname: String) {
        self.external_kbds.remove(&sysname);
        if self.external_kbds.is_empty() {
            self.enable_platform_kbds();
        }
    }

    fn disable_platform_kbds(&mut self) {
        self.platform_kbds
            .values_mut()
            .for_each(|kbd| kbd.grab().expect("Do you have root permissions?"));
    }

    fn enable_platform_kbds(&mut self) {
        self.platform_kbds
            .values_mut()
            .for_each(|kbd| kbd.ungrab().expect("Do you have root permissions?"));
    }
}

async fn poll_in<T: AsRawFd>(fd: &AsyncFd<T>, f: impl FnOnce(&AsyncFdReadyGuard<T>)) {
    if let Ok(mut guard) = fd.ready(tokio::io::Interest::READABLE).await {
        f(&guard);
        guard.clear_ready();
    }
}
