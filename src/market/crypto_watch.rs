use sha2::{Digest, Sha256};
use tokio::time::Duration;
use hex;
use serde_json::json;
use anyhow::Result;
use reqwest;
use sled;
use std::sync::Arc;
use crate::market::land::LandListing as LandListing;

pub struct ChainConfig {
    pub name: &'static str,
    pub rpc_url: String,
    pub confirmations: u64,
}

pub fn load_chain_configs() -> Vec<ChainConfig> {
    vec![
        ChainConfig { name: "BTC", rpc_url: std::env::var("VISION_ELECTRUM_BTC").unwrap_or("tcp://electrum.blockstream.info:50001".into()), confirmations: 2 },
        ChainConfig { name: "BCH", rpc_url: std::env::var("VISION_ELECTRUM_BCH").unwrap_or("tcp://bch.imaginary.cash:50001".into()), confirmations: 2 },
        ChainConfig { name: "DOGE", rpc_url: std::env::var("VISION_ELECTRUM_DOGE").unwrap_or("tcp://electrum.dogecoin.org:50001".into()), confirmations: 40 },
    ]
}

pub fn generate_invoice_address(chain: &str, listing_id: &str) -> String {
    let seed = std::env::var("VISION_INVOICE_SEED").unwrap_or("VisionSeedDefault".into());
    let hash = Sha256::digest(format!("{}:{}:{}", chain, listing_id, seed).as_bytes());
    format!("{}_{}", chain.to_lowercase(), &hex::encode(hash)[..16])
}

async fn electrum_rpc_call(url: &str, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
    let body = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });
    let client = reqwest::Client::new();
    let endpoint = url.replace("tcp://", "https://");
    let resp = client.post(&endpoint)
        .json(&body)
        .send()
        .await?;
    let v: serde_json::Value = resp.json().await?;
    Ok(v["result"].clone())
}

/// Watcher encapsulates the polling loop and (optionally) a test notify hook.
pub struct Watcher {
    pub chain: String,
    pub rpc_url: String,
    pub confirmations: u64,
    pub db: Arc<sled::Db>,
    pub poll_interval: Duration,
    #[cfg(feature = "testhooks")]
    pub test_notify: Option<std::sync::Arc<tokio::sync::Notify>>,
}

impl Watcher {
    pub fn new(chain: String, rpc_url: String, confirmations: u64, db: Arc<sled::Db>, poll_secs: u64) -> Self {
        Self {
            chain,
            rpc_url,
            confirmations,
            db,
            poll_interval: Duration::from_secs(poll_secs),
            #[cfg(feature = "testhooks")]
            test_notify: None,
        }
    }

    #[cfg(feature = "testhooks")]
    #[allow(dead_code)]
    pub fn with_test_notify(mut self, notify: std::sync::Arc<tokio::sync::Notify>) -> Self {
        self.test_notify = Some(notify);
        self
    }

    pub async fn run(self) {
        let interval = self.poll_interval;
        loop {
            let _ = self.tick_once().await;
            #[cfg(feature = "testhooks")]
            {
                use tokio::time::{timeout, Duration as TokioDuration};
                if let Some(n) = &self.test_notify {
                    let _ = timeout(TokioDuration::from_millis(250), n.notified()).await;
                } else {
                    tokio::time::sleep(interval).await;
                }
            }
            #[cfg(not(feature = "testhooks"))]
            {
                tokio::time::sleep(interval).await;
            }
        }
    }

    pub async fn tick_once(&self) -> Result<()> {
        run_watch_cycle(&self.chain, &self.rpc_url, self.confirmations, self.db.clone()).await
    }
}

#[allow(unused_variables)]
pub async fn spawn_crypto_watchers(db: Arc<sled::Db>, test_notify: Option<std::sync::Arc<tokio::sync::Notify>>) {
    let configs = load_chain_configs();
    let initial_secs: u64 = std::env::var("VISION_WATCH_POLL_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(30);
    for cfg in configs {
        let w = Watcher::new(cfg.name.to_string(), cfg.rpc_url.to_string(), cfg.confirmations, db.clone(), initial_secs);
        #[cfg(feature = "testhooks")]
        if let Some(_nt) = &test_notify {
            // if testhooks are enabled and a notify is supplied, the with_test_notify call
            // happens inside spawn_crypto_watchers caller path via mod wiring. Keep here for clarity.
        }
        tokio::spawn(async move { w.run().await });
    }
}

async fn run_watch_cycle(chain: &str, rpc_url: &str, _conf_req: u64, db: Arc<sled::Db>) -> Result<()> {

    // open the listings tree and iterate entries (land listings are stored by listing_id)
    let tree = db.open_tree("market_land_listings")?;
    for kv in tree.iter() {
        let (_k, val) = kv?;
        let listing: LandListing = serde_json::from_slice(&val)?;
        if listing.price_chain.to_uppercase() != chain.to_uppercase() {
            continue;
        }

        // Query history for the pay_to address
        let hist = electrum_rpc_call(rpc_url, "blockchain.address.get_history", json!([listing.pay_to])).await;
        match hist {
            Ok(h) => {
                if let Some(arr) = h.as_array() {
                    for tx in arr.iter() {
                        let height = tx["height"].as_i64().unwrap_or(0);
                        let tx_hash = tx["tx_hash"].as_str().unwrap_or("");
                        if height > 0 {
                            // For now, treat height > 0 as confirmed; a real implementation would use confirmations
                            println!("{} confirmed for {} tx {}", chain, listing.listing_id, tx_hash);
                            // POST to internal confirm endpoint
                            // Post confirm to the configured server URL (use VISION_SERVER_URL or VISION_PORT)
                            let server_base = std::env::var("VISION_SERVER_URL").unwrap_or_else(|_| {
                                let port = std::env::var("VISION_PORT").unwrap_or_else(|_| "8080".into());
                                format!("http://127.0.0.1:{}", port)
                            });
                            let confirm_url = format!("{}/_market/land/confirm", server_base);
                            let _ = reqwest::Client::new()
                                .post(&confirm_url)
                                .json(&json!({ "listing_id": listing.listing_id, "observed_txid": tx_hash, "chain": chain }))
                                .send()
                                .await;
                            break;
                        } else {
                            println!("{}: listing {} waiting for confirmations", chain, listing.listing_id);
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("electrum rpc error for {}: {:?}", chain, err);
                // Fallback: could call block explorer API here
            }
        }
    }

    Ok(())
}
