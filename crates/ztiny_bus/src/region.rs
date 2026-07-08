use ztiny_core::numeric::AddressType;

pub struct Region<A>
where
    A: AddressType,
{
    base: A,
    end: A,
}
