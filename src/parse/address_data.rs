use bincode::{Decode, Encode};

use crate::bitcoin::sats_to_btc;

use super::{EmptyAddressData, LiquidityClassification, RawAddressType};

#[derive(Encode, Decode, Debug, Clone, Copy)]
pub struct AddressData {
    pub address_type: RawAddressType,
    pub amount: u64,
    pub sent: u64,
    pub received: u64,
    pub mean_price_paid: f32,
    pub outputs_len: u32,
}

impl AddressData {
    pub fn new(address_type: RawAddressType) -> Self {
        Self {
            address_type,
            amount: 0,
            sent: 0,
            received: 0,
            mean_price_paid: 0.0,
            outputs_len: 0,
        }
    }

    pub fn compute_liquidity_classification(&self) -> LiquidityClassification {
        LiquidityClassification::new(self.sent, self.received)
    }
}

impl AddressData {
    pub fn receive(&mut self, sat_amount: u64, price: f32) {
        let price = price as f64;
        let previous_mean_price_paid = self.mean_price_paid as f64;

        let previous_sat_amount = self.amount;
        let new_sat_amount = previous_sat_amount + sat_amount;

        let btc_amount = sats_to_btc(sat_amount);
        let priced_btc_value = btc_amount * price;

        let previous_btc_amount = sats_to_btc(previous_sat_amount);
        let new_btc_amount = sats_to_btc(new_sat_amount);

        self.mean_price_paid = ((previous_mean_price_paid * previous_btc_amount + priced_btc_value)
            / new_btc_amount) as f32;

        self.amount = new_sat_amount;

        self.received += sat_amount;

        self.outputs_len += 1;
    }

    pub fn spend(&mut self, sat_amount: u64, price: f32) -> f32 {
        let price = price as f64;
        let previous_mean_price_paid = self.mean_price_paid as f64;

        let previous_sat_amount = self.amount;
        let new_sat_amount = previous_sat_amount - sat_amount;

        let btc_value = sats_to_btc(sat_amount);
        let priced_btc_value = btc_value * price;

        let previous_btc_amount = sats_to_btc(previous_sat_amount);
        let new_btc_amount = sats_to_btc(new_sat_amount);

        self.mean_price_paid = (((previous_mean_price_paid * previous_btc_amount)
            - priced_btc_value)
            / new_btc_amount) as f32;

        self.amount = new_sat_amount;

        self.sent += sat_amount;

        self.outputs_len -= 1;

        (priced_btc_value - (btc_value * previous_mean_price_paid)) as f32
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.amount == 0
    }

    pub fn from_empty(empty: &EmptyAddressData) -> Self {
        Self {
            address_type: empty.address_type,
            amount: 0,
            sent: empty.transfered,
            received: empty.transfered,
            mean_price_paid: 0.0,
            outputs_len: 0,
        }
    }
}
