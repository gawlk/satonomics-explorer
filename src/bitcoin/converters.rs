use super::SATOSHIS_PER_BITCOIN;

pub fn sats_to_btc(sats: u64) -> f64 {
    sats as f64 / SATOSHIS_PER_BITCOIN as f64
}
