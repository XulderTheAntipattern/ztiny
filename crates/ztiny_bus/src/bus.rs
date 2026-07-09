use ztiny_core::numeric::{AddressType, WordType};

use crate::{
    Attachment, Device, DeviceId, Region,
    map::{AddressMap, VecAddressMap},
};

#[derive(Default)]
pub struct Bus<A: AddressType, W: WordType> {
    devices: Vec<Box<dyn Device<Address = A, Word = W>>>,
    map: VecAddressMap<A>,
}

impl<A, W> Bus<A, W>
where
    A: AddressType,
    W: WordType,
{
    pub fn new() -> Self {
        Self { devices: Vec::new(), map: VecAddressMap::new() }
    }

    pub fn attach(
        &mut self,
        device: Box<dyn Device<Address = A, Word = W>>,
        region: Region<A>,
    ) -> DeviceId {
        let id = DeviceId(self.devices.len());
        self.devices.push(device);
        self.map.insert(Attachment { region, device: id });
        id
    }
}

impl<A, W> Bus<A, W>
where
    A: AddressType + std::ops::Sub<Output = A>,
    W: WordType,
{
    pub fn read(&mut self, address: A) -> Option<W> {
        let attachment = self.map.lookup(address)?;
        let device_id = attachment.device.0;
        let offset = attachment.region.offset(address)?;
        Some(self.devices.get_mut(device_id)?.read(offset))
    }

    pub fn write(&mut self, address: A, value: W) -> Option<()> {
        let attachment = self.map.lookup(address)?;
        let device_id = attachment.device.0;
        let offset = attachment.region.offset(address)?;

        #[allow(renamed_and_removed_lints)]
        #[allow(unit_arg)]
        Some(self.devices.get_mut(device_id)?.write(offset, value))
    }
}
