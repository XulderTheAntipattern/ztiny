use ztiny_core::numeric::AddressType;

use crate::{Attachment, DeviceId};

// SECTION: Address map trait
pub trait AddressMap<A: AddressType> {
    // TODO: Add error handling once the API needs richer failures.
    /// Register a new attachment in the address map.
    fn insert(&mut self, attachment: Attachment<A>);

    /// Look up an attachment record for a given address.
    ///
    /// Returns `None` when the address is unmapped.
    fn lookup(&self, address: A) -> Option<&Attachment<A>>;

    /// Find the device ID that covers a global address.
    ///
    /// Returns `None` when the address is unmapped.
    /// Default implementation uses lookup.
    fn find_device(&self, address: A) -> Option<DeviceId> {
        self.lookup(address).map(|a| a.device)
    }
}
// !SECTION

// SECTION: Vector-based address map implementation
#[derive(Default)]
pub struct VecAddressMap<A: AddressType> {
    attachments: Vec<Attachment<A>>,
}

impl<A: AddressType> VecAddressMap<A> {
    /// Create an empty address map.
    pub fn new() -> Self {
        Self { attachments: Vec::new() }
    }

    /// Initialize an address map from a prebuilt attachment list.
    pub fn with_attachments(attachments: Vec<Attachment<A>>) -> Self {
        Self { attachments }
    }
}

impl<A: AddressType> AddressMap<A> for VecAddressMap<A> {
    fn insert(&mut self, attachment: Attachment<A>) {
        self.attachments.push(attachment);
    }

    fn lookup(&self, address: A) -> Option<&Attachment<A>> {
        self.attachments.iter().find(|attachment| attachment.region.contains(address))
    }
}
// !SECTION
