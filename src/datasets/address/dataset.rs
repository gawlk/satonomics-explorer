use chrono::NaiveDate;
use itertools::Itertools;

use crate::{
    datasets::{AnyDataset, ProcessedBlockData},
    parse::{AnyDateMap, AnyHeightMap, BiMap, RawAddressSplit},
    states::LiquiditySplitProcessedAddressState,
};

use super::AddressSubDataset;

pub struct MetadataDataset {
    min_initial_first_unsafe_date: Option<NaiveDate>,
    min_initial_first_unsafe_height: Option<usize>,

    address_count: BiMap<usize>,
}

impl MetadataDataset {
    pub fn import(parent_path: &str) -> color_eyre::Result<Self> {
        let f = |s: &str| format!("{parent_path}/{s}");

        let mut s = Self {
            address_count: BiMap::new_on_disk_bin(&f("address_count")),

            min_initial_first_unsafe_date: None,
            min_initial_first_unsafe_height: None,
        };

        s.min_initial_first_unsafe_date = s.compute_min_initial_first_unsafe_date();
        s.min_initial_first_unsafe_height = s.compute_min_initial_first_unsafe_height();

        Ok(s)
    }
}

impl AnyDataset for MetadataDataset {
    fn to_any_height_map_vec(&self) -> Vec<&(dyn AnyHeightMap + Send + Sync)> {
        vec![&self.address_count.height]
    }

    fn to_any_date_map_vec(&self) -> Vec<&(dyn AnyDateMap + Send + Sync)> {
        vec![&self.address_count.date]
    }

    fn get_min_initial_first_unsafe_date(&self) -> &Option<NaiveDate> {
        &self.min_initial_first_unsafe_date
    }

    fn get_min_initial_first_unsafe_height(&self) -> &Option<usize> {
        &self.min_initial_first_unsafe_height
    }
}

pub struct CohortDataset {
    min_initial_first_unsafe_date: Option<NaiveDate>,
    min_initial_first_unsafe_height: Option<usize>,

    split: RawAddressSplit,

    metadata: MetadataDataset,

    all_dataset: AddressSubDataset,
    illiquid_dataset: AddressSubDataset,
    liquid_dataset: AddressSubDataset,
    highly_liquid_dataset: AddressSubDataset,
}

impl CohortDataset {
    pub fn import(
        parent_path: &str,
        name: &str,
        split: RawAddressSplit,
    ) -> color_eyre::Result<Self> {
        let folder_path = format!("{parent_path}/{name}");

        let f = |s: &str| format!("{folder_path}/{s}");

        let mut s = Self {
            min_initial_first_unsafe_date: None,
            min_initial_first_unsafe_height: None,
            split,
            metadata: MetadataDataset::import(&folder_path)?,
            all_dataset: AddressSubDataset::import(&folder_path)?,
            illiquid_dataset: AddressSubDataset::import(&f("illiquid"))?,
            liquid_dataset: AddressSubDataset::import(&f("liquid"))?,
            highly_liquid_dataset: AddressSubDataset::import(&f("highly_liquid"))?,
        };

        s.min_initial_first_unsafe_date = s.compute_min_initial_first_unsafe_date();
        s.min_initial_first_unsafe_height = s.compute_min_initial_first_unsafe_height();

        Ok(s)
    }

    pub fn sub_datasets_vec(&self) -> Vec<&AddressSubDataset> {
        vec![
            &self.all_dataset,
            &self.illiquid_dataset,
            &self.liquid_dataset,
            &self.highly_liquid_dataset,
        ]
    }

    pub fn needs_metadata(&self, date: NaiveDate, height: usize) -> bool {
        self.metadata.process_date(date) || self.metadata.process_height(height)
    }

    pub fn needs_utxos_metadata(&self, date: NaiveDate, height: usize) -> bool {
        self.sub_datasets_vec()
            .iter()
            .any(|sub| !sub.utxos_metadata.are_date_and_height_safe(date, height))
    }

    pub fn needs_supply(&self, date: NaiveDate, height: usize) -> bool {
        self.sub_datasets_vec()
            .iter()
            .any(|sub| !sub.supply.are_date_and_height_safe(date, height))
    }

    pub fn needs_price_paid(&self, date: NaiveDate, height: usize) -> bool {
        self.sub_datasets_vec()
            .iter()
            .any(|sub| !sub.price_paid.are_date_and_height_safe(date, height))
    }

    fn needs_realized_data(&self, date: NaiveDate, height: usize) -> bool {
        self.sub_datasets_vec()
            .iter()
            .any(|sub| !sub.realized.are_date_and_height_safe(date, height))
    }

    fn needs_unrealized_data(&self, date: NaiveDate, height: usize) -> bool {
        self.sub_datasets_vec()
            .iter()
            .any(|sub| !sub.unrealized.are_date_and_height_safe(date, height))
    }

    fn insert_realized_data(&self, processed_block_data: &ProcessedBlockData) {
        let split_realized_state = processed_block_data
            .split_realized_states
            .as_ref()
            .unwrap()
            .get_state(&self.split)
            .unwrap();

        self.all_dataset
            .realized
            .insert(processed_block_data, &split_realized_state.all);

        self.illiquid_dataset
            .realized
            .insert(processed_block_data, &split_realized_state.illiquid);

        self.liquid_dataset
            .realized
            .insert(processed_block_data, &split_realized_state.liquid);

        self.highly_liquid_dataset
            .realized
            .insert(processed_block_data, &split_realized_state.highly_liquid);
    }

    fn insert_metadata(
        &self,
        &ProcessedBlockData {
            height,
            date,
            is_date_last_block,
            states,
            ..
        }: &ProcessedBlockData,
    ) {
        let address_count = states
            .split_address
            .get_state(&self.split)
            .unwrap()
            .address_count;

        self.metadata
            .address_count
            .height
            .insert(height, address_count);

        if is_date_last_block {
            self.metadata.address_count.date.insert(date, address_count);
        }
    }

    fn insert_supply(
        &self,
        processed_block_data: &ProcessedBlockData,
        liquidity_split_state: &LiquiditySplitProcessedAddressState,
    ) {
        self.all_dataset.supply.insert(
            processed_block_data,
            &liquidity_split_state.split.all.supply,
        );

        self.illiquid_dataset.supply.insert(
            processed_block_data,
            &liquidity_split_state.split.illiquid.supply,
        );

        self.liquid_dataset.supply.insert(
            processed_block_data,
            &liquidity_split_state.split.liquid.supply,
        );

        self.highly_liquid_dataset.supply.insert(
            processed_block_data,
            &liquidity_split_state.split.highly_liquid.supply,
        );
    }

    fn insert_utxos_metadata(
        &self,
        processed_block_data: &ProcessedBlockData,
        liquidity_split_state: &LiquiditySplitProcessedAddressState,
    ) {
        self.all_dataset.utxos_metadata.insert(
            processed_block_data,
            &liquidity_split_state.split.all.utxos_metadata,
        );

        self.illiquid_dataset.utxos_metadata.insert(
            processed_block_data,
            &liquidity_split_state.split.illiquid.utxos_metadata,
        );

        self.liquid_dataset.utxos_metadata.insert(
            processed_block_data,
            &liquidity_split_state.split.liquid.utxos_metadata,
        );

        self.highly_liquid_dataset.utxos_metadata.insert(
            processed_block_data,
            &liquidity_split_state.split.highly_liquid.utxos_metadata,
        );
    }

    fn insert_unrealized(&self, processed_block_data: &ProcessedBlockData) {
        let height_state = processed_block_data
            .split_unrealized_states_height
            .as_ref()
            .unwrap()
            .get_state(&self.split)
            .unwrap();

        let date_state = processed_block_data
            .split_unrealized_states_height
            .as_ref()
            .unwrap()
            .get_state(&self.split)
            .unwrap();

        self.all_dataset.unrealized.insert(
            processed_block_data,
            &height_state.all,
            &date_state.all,
        );

        self.illiquid_dataset.unrealized.insert(
            processed_block_data,
            &height_state.illiquid,
            &date_state.illiquid,
        );

        self.liquid_dataset.unrealized.insert(
            processed_block_data,
            &height_state.liquid,
            &date_state.liquid,
        );

        self.highly_liquid_dataset.unrealized.insert(
            processed_block_data,
            &height_state.highly_liquid,
            &date_state.highly_liquid,
        );
    }

    fn insert_price_paid(&self, processed_block_data: &ProcessedBlockData) {
        let state = processed_block_data
            .split_price_paid_states
            .as_ref()
            .unwrap()
            .get_state(&self.split)
            .unwrap();

        self.all_dataset
            .price_paid
            .insert(processed_block_data, &state.all);
        self.illiquid_dataset
            .price_paid
            .insert(processed_block_data, &state.illiquid);
        self.liquid_dataset
            .price_paid
            .insert(processed_block_data, &state.liquid);
        self.highly_liquid_dataset
            .price_paid
            .insert(processed_block_data, &state.highly_liquid);
    }
}

impl AnyDataset for CohortDataset {
    fn insert_block_data(&self, processed_block_data: &ProcessedBlockData) {
        let &ProcessedBlockData { height, date, .. } = processed_block_data;

        let needs_metadata = self.needs_metadata(date, height);
        let needs_unrealized_data = self.needs_unrealized_data(date, height);
        let needs_realized = self.needs_realized_data(date, height);
        let needs_price_paid = self.needs_price_paid(date, height);
        let needs_supply = needs_price_paid || self.needs_supply(date, height);
        let needs_utxos_metadata = self.needs_utxos_metadata(date, height);

        let liquidity_split_processed_address_state = processed_block_data
            .states
            .split_address
            .get_state(&self.split);

        if liquidity_split_processed_address_state.is_none() {
            return;
        }

        let liquidity_split_processed_address_state =
            liquidity_split_processed_address_state.unwrap();

        if needs_metadata {
            self.insert_metadata(processed_block_data);
        }

        if needs_utxos_metadata {
            self.insert_utxos_metadata(
                processed_block_data,
                liquidity_split_processed_address_state,
            );
        }

        if needs_supply {
            self.insert_supply(
                processed_block_data,
                liquidity_split_processed_address_state,
            );
        }

        if needs_realized {
            self.insert_realized_data(processed_block_data);
        }

        if needs_unrealized_data {
            self.insert_unrealized(processed_block_data);
        }

        if needs_price_paid {
            self.insert_price_paid(processed_block_data);
        }
    }

    fn to_any_height_map_vec(&self) -> Vec<&(dyn AnyHeightMap + Send + Sync)> {
        [
            self.all_dataset.to_any_height_map_vec(),
            self.illiquid_dataset.to_any_height_map_vec(),
            self.liquid_dataset.to_any_height_map_vec(),
            self.highly_liquid_dataset.to_any_height_map_vec(),
            self.metadata.to_any_height_map_vec(),
        ]
        .into_iter()
        .flatten()
        .collect_vec()
    }

    fn to_any_date_map_vec(&self) -> Vec<&(dyn AnyDateMap + Send + Sync)> {
        [
            self.all_dataset.to_any_date_map_vec(),
            self.illiquid_dataset.to_any_date_map_vec(),
            self.liquid_dataset.to_any_date_map_vec(),
            self.highly_liquid_dataset.to_any_date_map_vec(),
            self.metadata.to_any_date_map_vec(),
        ]
        .into_iter()
        .flatten()
        .collect_vec()
    }

    fn get_min_initial_first_unsafe_date(&self) -> &Option<NaiveDate> {
        &self.min_initial_first_unsafe_date
    }

    fn get_min_initial_first_unsafe_height(&self) -> &Option<usize> {
        &self.min_initial_first_unsafe_height
    }
}
