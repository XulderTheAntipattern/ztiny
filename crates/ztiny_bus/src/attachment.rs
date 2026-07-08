use ztiny_core::numeric::AddressType;

use crate::{DeviceId, Region};

#[allow(dead_code)]
pub struct Attachment<A>
where
    A: AddressType,
{
    pub region: Region<A>,
    pub device: DeviceId,
}
