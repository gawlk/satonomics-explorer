use rayon::prelude::*;

use crate::structs::AnyHeightMap;

use super::ProcessedData;

pub trait HeightDatasetTrait {
    fn get_min_last_height(&self) -> Option<usize> {
        self.to_vec()
            .iter()
            .map(|dataset| dataset.get_last_height())
            .min()
            .and_then(|opt| opt)
    }

    fn get_min_initial_first_unsafe_height(&self) -> Option<usize> {
        self.to_vec()
            .iter()
            .map(|dataset| dataset.get_initial_first_unsafe_height())
            .min()
            .and_then(|opt| opt)
    }

    fn export(&self) -> color_eyre::Result<()> {
        self.to_vec()
            .iter()
            .try_for_each(|dataset| dataset.export())
    }

    fn insert(&self, processed_data: &ProcessedData);

    fn to_vec(&self) -> Vec<&(dyn AnyHeightMap + Send + Sync)>;
}

pub trait HeightDatasetsTrait {
    fn get_min_last_height(&self) -> Option<usize> {
        self.to_vec()
            .iter()
            .map(|dataset| dataset.get_min_last_height())
            .min()
            .and_then(|opt| opt)
    }

    fn export_if_needed(&self, height: usize) -> color_eyre::Result<()> {
        self.to_vec()
            .par_iter()
            .filter(|dataset| dataset.get_min_initial_first_unsafe_height().unwrap_or(0) <= height)
            .try_for_each(|dataset| dataset.export())
    }

    fn export(&self) -> color_eyre::Result<()> {
        self.to_vec()
            .par_iter()
            .try_for_each(|dataset| dataset.export())
    }

    fn insert(&self, processed_data: ProcessedData) {
        let ProcessedData { height, .. } = processed_data;

        self.to_vec()
            .par_iter()
            .filter(|dataset| dataset.get_min_initial_first_unsafe_height().unwrap_or(0) <= height)
            .for_each(|dataset| dataset.insert(&processed_data));
    }

    fn to_vec(&self) -> Vec<&(dyn HeightDatasetTrait + Send + Sync)>;
}
