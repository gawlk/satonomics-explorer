mod date;
mod height;

use chrono::NaiveDate;
use date::*;
use height::*;

use super::AnyDatasets;

pub struct PriceDatasets {
    date: DateDataset,
    height: HeightDataset,
}

impl PriceDatasets {
    pub fn import() -> color_eyre::Result<Self> {
        let path = format!("./{}", Self::name());

        Ok(Self {
            date: DateDataset::import(&path)?,
            height: HeightDataset::import(&path)?,
        })
    }

    pub fn date_to_close(&mut self, date: NaiveDate) -> color_eyre::Result<f32> {
        self.date.get(date)
    }

    pub fn height_to_close(&mut self, height: usize, timestamp: u32) -> color_eyre::Result<f32> {
        self.height.get(height, timestamp)
    }
}

impl AnyDatasets for PriceDatasets {
    fn to_any_dataset_vec(&self) -> Vec<&(dyn super::AnyDataset + Send + Sync)> {
        vec![&self.date, &self.height]
    }

    fn name<'a>() -> &'a str {
        "price"
    }
}
