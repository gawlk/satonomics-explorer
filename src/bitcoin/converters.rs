use super::SATOSHIS_PER_BITCOIN;

#[allow(unused)]
#[inline(always)]
pub fn sats_to_btc(sats: u64) -> f64 {
    sats as f64 / SATOSHIS_PER_BITCOIN as f64
}

#[allow(unused)]
#[inline(always)]
pub fn btc_to_sats(btc: f64) -> u64 {
    (btc * SATOSHIS_PER_BITCOIN as f64) as u64
}
