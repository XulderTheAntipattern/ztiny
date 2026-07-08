use ztiny_core::numeric::{AddressType, WordType};

use crate::{Attachment, Device};

// pub trait Bus {
//     type Address: AddressType;
//     type Word: WordType;

//     fn read(&mut self, address: Self::Address) -> Self::Word;

//     fn write(&mut self, address: Self::Address, value: Self::Word);
// }

pub struct Bus<A: AddressType, W: WordType> {
    devices: Vec<Box<dyn Device<Address = A, Word = W>>>,
    attachments: Vec<Attachment<A>>,
}
