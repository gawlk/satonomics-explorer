use std::f64::EPSILON;

use crate::bitcoin::sats_to_btc;

pub struct LiquidityClassification {
    illiquid: f64,
    liquid: f64,
    highly_liquid: f64,
}

impl LiquidityClassification {
    /// Following this:
    /// https://insights.glassnode.com/bitcoin-liquid-supply/
    /// https://www.desmos.com/calculator/ezvbhwk4ph
    pub fn new(sent: u64, received: u64) -> Self {
        let liquidity = {
            let liquidity = sats_to_btc(sent) / sats_to_btc(received);

            if liquidity.is_nan() {
                0.0
            } else {
                liquidity
            }
        };

        let illiquid = Self::compute_illiquid(liquidity);
        let liquid = Self::compute_liquid(liquidity);

        Self {
            illiquid,
            liquid,
            highly_liquid: 1.0 - liquid - illiquid,
        }
    }

    #[inline(always)]
    pub fn split(&self, value: f64) -> LiquiditySplitResult {
        LiquiditySplitResult {
            all: value,
            illiquid: value * self.illiquid,
            liquid: value * self.liquid,
            highly_liquid: value * self.highly_liquid,
        }
    }

    /// Returns value in range 0.0..1.0
    #[inline(always)]
    fn compute_illiquid(x: f64) -> f64 {
        Self::compute_ratio(x, 0.25)
    }

    /// Returns value in range 0.0..1.0
    #[inline(always)]
    fn compute_liquid(x: f64) -> f64 {
        Self::compute_ratio(x, 0.75)
    }

    #[inline(always)]
    fn compute_ratio(x: f64, x0: f64) -> f64 {
        let l = 1.0;
        let k = 25.0;

        l / (1.0 + EPSILON.powf(k * (x - x0)))
    }
}

#[derive(Debug, Default)]
pub struct LiquiditySplitResult {
    pub all: f64,
    pub illiquid: f64,
    pub liquid: f64,
    pub highly_liquid: f64,
}

#[derive(Debug, Default)]
pub struct SplitByLiquidity<T>
where
    T: Default,
{
    pub all: T,
    pub illiquid: T,
    pub liquid: T,
    pub highly_liquid: T,
}
