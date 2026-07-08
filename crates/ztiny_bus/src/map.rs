use ztiny_core::numeric::{AddressType, WordType};

use crate::{Attachment, DeviceId};

pub(crate) struct AddressMap<A: AddressType>
where
    A: AddressType,
{
    attachments: Vec<Attachment<A>>,
}

impl<A> AddressMap<A>
where
    A: AddressType,
{
    pub fn find(&self, address: A) -> DeviceId {
        return DeviceId(0);
    }
}
