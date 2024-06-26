use bincode::{Decode, Encode};

#[derive(Encode, Decode, Debug)]
pub struct BlockData {
    pub height: u32,
    pub price: f32,
    pub amount: u64,
    pub spendable_outputs: u32,
}

impl BlockData {
    pub fn new(height: u32, price: f32) -> Self {
        Self {
            height,
            price,
            amount: 0,
            spendable_outputs: 0,
        }
    }
}
