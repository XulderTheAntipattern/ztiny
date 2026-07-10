use ztiny_core::numeric::AddressType;

use crate::{DeviceId, Region};

// SECTION: Attachment bookkeeping
// NOTE: Overlapping regions should be validated when attachments are added.
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
