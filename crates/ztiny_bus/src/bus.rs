use ztiny_core::numeric::{AddressType, WordType};

use crate::{Attachment, Device, DeviceId, Region, map::AddressMap};

pub trait BusAccess {
    type Address: AddressType;
    type Word: WordType;

    fn read(&mut self, address: Self::Address) -> Option<Self::Word>;

    // TODO: Will refactor to return Result after implementing error handling.
    fn write(
        &mut self,
        address: Self::Address,
        value: Self::Word,
    ) -> Option<()>;
}

// REVIEW: What owns a bus? The motherboard does. The cpu doesn't, the devices don't.
// This may need to be turned into a
#[derive(Default)]
pub struct Bus<A: AddressType, W: WordType, M = dyn AddressMap<A>>
where
    M: AddressMap<A>,
{
    devices: Vec<Box<dyn Device<Address = A, Word = W>>>,
    map: M,
}

impl<A, W, M> Bus<A, W, M>
where
    A: AddressType,
    W: WordType,
    M: AddressMap<A>,
{
    pub fn with_map(map: M) -> Self {
        Self { devices: Vec::new(), map }
    }

    pub fn new() -> Self
    where
        M: Default,
    {
        Self::with_map(M::default())
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

impl<A, W, M> Bus<A, W, M>
where
    A: AddressType + std::ops::Sub<Output = A>,
    W: WordType,
    M: AddressMap<A>,
{
    fn read_impl(&mut self, address: A) -> Option<W> {
        let attachment = self.map.lookup(address)?;
        let device_id = attachment.device.0;
        let offset = attachment.region.offset(address)?;
        Some(self.devices.get_mut(device_id)?.read(offset))
    }

    pub fn read(&mut self, address: A) -> Option<W> {
        self.read_impl(address)
    }

    fn write_impl(&mut self, address: A, value: W) -> Option<()> {
        let attachment = self.map.lookup(address)?;
        let device_id = attachment.device.0;
        let offset = attachment.region.offset(address)?;

        let device = self.devices.get_mut(device_id)?;
        device.write(offset, value);
        Some(())
    }

    pub fn write(&mut self, address: A, value: W) -> Option<()> {
        self.write_impl(address, value)
    }
}

impl<A, W, M> BusAccess for Bus<A, W, M>
where
    A: AddressType + std::ops::Sub<Output = A>,
    W: WordType,
    M: AddressMap<A>,
{
    type Address = A;
    type Word = W;

    fn read(&mut self, address: Self::Address) -> Option<Self::Word> {
        self.read_impl(address)
    }

    fn write(
        &mut self,
        address: Self::Address,
        value: Self::Word,
    ) -> Option<()> {
        self.write_impl(address, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct CustomMap<A: AddressType> {
        attachments: Vec<Attachment<A>>,
    }

    impl<A: AddressType> AddressMap<A> for CustomMap<A> {
        fn insert(&mut self, attachment: Attachment<A>) {
            self.attachments.push(attachment);
        }

        fn lookup(&self, address: A) -> Option<&Attachment<A>> {
            self.attachments
                .iter()
                .find(|attachment| attachment.region.contains(address))
        }
    }

    #[test]
    fn bus_accepts_custom_address_map() {
        let _bus: Bus<u16, u8, CustomMap<u16>> =
            Bus::with_map(CustomMap::default());
    }
}
