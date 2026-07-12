use ztiny_core::numeric::AddressType;

use crate::{Attachment, DeviceId};

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

// TODO: Implement som basic
