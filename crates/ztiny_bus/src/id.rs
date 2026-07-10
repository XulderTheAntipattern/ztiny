// SECTION: Identifier types
// NOTE: These lightweight wrappers keep bus and device IDs distinct.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DeviceId(pub(crate) usize);

// TODO: Remove or implement once the ID model is finalized.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BusId(pub(crate) usize);

// TODO: Remove or implement once attachment IDs are wired into the bus layer.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AttachmentId(pub(crate) usize);
