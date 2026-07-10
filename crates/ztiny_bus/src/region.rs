use ztiny_core::numeric::AddressType;

// SECTION: Address region helper
pub struct Region<A>
where
    A: AddressType,
{
    base: A,
    end: A,
}

impl<A> Region<A>
where
    A: AddressType,
{
    /// Build a region from a start address to an inclusive end address.
    pub fn new(base: A, end: A) -> Self {
        Self { base, end }
    }

    /// Check whether a global address falls into this region.
    pub fn contains(&self, address: A) -> bool {
        address >= self.base && address <= self.end
    }
}

impl<A> Region<A>
where
    A: AddressType + std::ops::Sub<Output = A>,
{
    /// Compute the device-local offset for a given global address.
    pub fn offset(&self, address: A) -> Option<A> {
        if self.contains(address) { Some(address - self.base) } else { None }
    }
}
