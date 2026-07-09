use ztiny_core::numeric::AddressType;

use crate::{DeviceId, Region};

// SECTION: Attachment bookkeeping
// A device attachment ties a device to a region of the bus address space.
// NOTE: Future work can validate overlapping regions on attach.
#[allow(dead_code)]
pub struct Attachment<A>
where
    A: AddressType,
{
    /// The address region covered by this attachment.
    pub region: Region<A>,

    /// The ID of the attached device.
    pub device: DeviceId,
}
