// SECTION: Identifier types
// These are small wrappers for internal indexing and domain clarity.
// NOTE: If the repo adopts a generic ID trait, these can be adapted.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DeviceId(pub(crate) usize);

// TODO: Remove or implement.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BusId(pub(crate) usize);

// TODO: Remove or implement.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AttachmentId(pub(crate) usize);
