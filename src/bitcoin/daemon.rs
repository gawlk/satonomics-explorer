#![allow(unused)]
use std::{process::Command, thread::sleep, time::Duration};

use serde_json::Value;

pub const BITCOIN_DATADIR_RAW_PATH: &str = "/Users/k/Developer/bitcoin";

struct BlockchainInfo {
    pub headers: u64,
    pub blocks: u64,
}

pub struct Daemon;

impl Daemon {
    pub fn start() {
        sleep(Duration::from_secs(1));

        println!("Starting node...");

        // bitcoind -datadir=/Users/k/Developer/bitcoin -blocksonly -txindex=1 -daemon -v2transport
        let output = Command::new("bitcoind")
            .arg(format!("-datadir={BITCOIN_DATADIR_RAW_PATH}"))
            .arg("-blocksonly")
            .arg("-txindex=1")
            .arg("-daemon")
            .arg("-v2transport")
            .output()
            .expect("bitcoind to be able to properly start");

        sleep(Duration::from_secs(15));

        println!("Node started successfully !");
    }

    pub fn stop() {
        // bitcoin-cli -datadir=/Users/k/Developer/bitcoin stop
        let status = Command::new("bitcoin-cli")
            .arg(format!("-datadir={BITCOIN_DATADIR_RAW_PATH}"))
            .arg("stop")
            .output()
            .unwrap()
            .status;

        if status.success() {
            println!("Stopping node...");
            sleep(Duration::from_secs(15));
            println!("bitcoind stopped successfully !");
        }
    }

    pub fn wait_sync() -> color_eyre::Result<()> {
        while !Self::check_if_fully_synced()? {
            sleep(Duration::from_secs(5))
        }

        Ok(())
    }

    pub fn wait_for_new_block(last_block_height: usize) -> color_eyre::Result<()> {
        println!("Waiting for new block...");

        while Self::get_blockchain_info()?.headers as usize == last_block_height {
            sleep(Duration::from_secs(5))
        }

        Ok(())
    }

    pub fn check_if_fully_synced() -> color_eyre::Result<bool> {
        let BlockchainInfo { blocks, headers } = Self::get_blockchain_info()?;

        let synced = blocks == headers;

        if synced {
            println!("Synced ! ({blocks} blocks)");
        } else {
            println!("Syncing... ({} remaining)", headers - blocks)
        }

        Ok(synced)
    }

    fn get_blockchain_info() -> color_eyre::Result<BlockchainInfo> {
        // bitcoin-cli -datadir=/Users/k/Developer/bitcoin getblockchaininfo
        let output = Command::new("bitcoin-cli")
            .arg(format!("-datadir={BITCOIN_DATADIR_RAW_PATH}"))
            .arg("getblockchaininfo")
            .output()
            .unwrap();

        let json: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))?;
        let json = json.as_object().unwrap();

        let blocks = json.get("blocks").unwrap().as_u64().unwrap();
        let headers = json.get("headers").unwrap().as_u64().unwrap();

        Ok(BlockchainInfo { headers, blocks })
    }
}
