use crate::chunk::Block;

pub struct Inventory {
    num_slots: u32,
    data: Vec<Box<dyn Item>>
}

impl Inventory {
    pub fn new(num_slots: u32) -> Self {
        Self { num_slots, data: Vec::new() }
    }
}

pub trait Item {

}

impl Item for Block {

}