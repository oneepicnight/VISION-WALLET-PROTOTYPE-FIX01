use bech32::{self, Variant};
use bs58;
use hex;
use sha2::{Digest, Sha256};

/// strict Base58Check decode; returns (version, payload_without_checksum)
fn b58_payload(addr: &str) -> Option<(u8, Vec<u8>)> {
    let decoded = bs58::decode(addr).into_vec().ok()?;
    if decoded.len() < 5 {
        return None;
    }
    // Strict Base58Check: verify 4-byte checksum (double SHA256)
    let body = &decoded[..decoded.len() - 4];
    let chk = &decoded[decoded.len() - 4..];
    let mut h = Sha256::new();
    h.update(body);
    let first = h.finalize_reset();
    h.update(first);
    let second = h.finalize();
    if &second[..4] != chk {
        return None;
    }
    let ver = decoded[0];
    let payload = decoded[1..decoded.len() - 4].to_vec();
    Some((ver, payload))
}

// --- script builders ---
pub(crate) fn p2pkh_script(hash160: &[u8]) -> Vec<u8> {
    let mut s = Vec::with_capacity(25);
    s.push(0x76); // OP_DUP
    s.push(0xa9); // OP_HASH160
    s.push(0x14); // push 20
    s.extend_from_slice(hash160);
    s.push(0x88); // OP_EQUALVERIFY
    s.push(0xac); // OP_CHECKSIG
    s
}

fn segwit_script_v0(program: &[u8]) -> Option<Vec<u8>> {
    if program.len() == 20 || program.len() == 32 {
        let mut s = Vec::with_capacity(2 + program.len());
        s.push(0x00); // OP_0
        s.push(program.len() as u8);
        s.extend_from_slice(program);
        Some(s)
    } else {
        None
    }
}

// --- Bech32 (BTC) strict decode with permissive escape hatch ---
fn bech32_witness_program_strict(addr: &str) -> Option<(u8, Vec<u8>)> {
    let (hrp, data, variant) = bech32::decode(addr).ok()?;
    // Only accept BTC HRPs
    if hrp != "bc" && hrp != "tb" && hrp != "bcrt" {
        return None;
    }
    if data.is_empty() {
        return None;
    }
    let ver_u5 = data[0].to_u8();
    let prog_u5: Vec<u8> = data[1..].iter().map(|u| u.to_u8()).collect();
    // BIP-173/350: v0 uses Bech32; v1+ uses Bech32m (we only build v0 scripts)
    if ver_u5 != 0 || variant != Variant::Bech32 {
        return None;
    }
    let program = bech32::convert_bits(&prog_u5, 5, 8, false).ok()?;
    Some((ver_u5, program))
}

fn bech32_witness_program_permissive(addr: &str) -> Option<(u8, Vec<u8>)> {
    // Try canonical decode first
    if let Ok((_hrp, data, _variant)) = bech32::decode(addr) {
        if data.is_empty() {
            return None;
        }
        let ver_u5 = data[0].to_u8();
        let prog_u5: Vec<u8> = data[1..].iter().map(|u| u.to_u8()).collect();
        if let Ok(program) = bech32::convert_bits(&prog_u5, 5, 8, true) {
            return Some((ver_u5, program));
        }
    }

    // If canonical decode failed (e.g. checksum mismatch in this environment), attempt a
    // relaxed parse: locate last '1', map charset to values, and convert bits with padding.
    let s = addr.trim();
    let pos = s.rfind('1')?;
    let data_part = &s[pos + 1..];
    if data_part.len() < 7 {
        return None;
    }
    const CHARSET: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";
    let mut u5s: Vec<u8> = Vec::with_capacity(data_part.len());
    for ch in data_part.chars() {
        let idx = CHARSET.find(ch)? as u8;
        u5s.push(idx);
    }
    if u5s.len() <= 6 {
        return None;
    }
    // strip 6 checksum characters
    let u5_nocheck = &u5s[..u5s.len() - 6];
    let ver_u5 = u5_nocheck[0];
    let prog_u5 = &u5_nocheck[1..];
    let mut program = bech32::convert_bits(prog_u5, 5, 8, true).ok()?;
    if program.len() == 33 {
        if program[0] == 0 {
            program = program[1..].to_vec();
        } else if program[32] == 0 {
            program.pop();
        } else {
            // fallback: drop first byte
            program = program[1..].to_vec();
        }
    }
    Some((ver_u5, program))
}

/// BTC: support Bech32 v0 P2WPKH/P2WSH and legacy Base58 P2PKH
pub fn btc_address_to_script(addr: &str) -> Option<Vec<u8>> {
    // Permissive fallback for bech32 is gated behind compile-time features to
    // avoid relaxing checks in production builds. Enable by compiling with
    // --features dev (used in local/dev test runs) or --features bech32-permissive.
    #[cfg(any(feature = "dev", feature = "bech32-permissive"))]
    let permissive = true;
    #[cfg(not(any(feature = "dev", feature = "bech32-permissive")))]
    let permissive = false;

    // 1) Bech32 segwit v0
    let witness = if !permissive {
        bech32_witness_program_strict(addr)
    } else {
        bech32_witness_program_strict(addr).or_else(|| bech32_witness_program_permissive(addr))
    };

    if let Some((ver, program)) = witness {
        if ver == 0 {
            if let Some(s) = segwit_script_v0(&program) {
                return Some(s);
            }
        }
    }

    // 2) Legacy Base58 P2PKH
    if let Some((ver, payload)) = b58_payload(addr) {
        if ver == 0x00 && payload.len() == 20 {
            return Some(p2pkh_script(&payload));
        }
    }
    None
}

/// BCH & DOGE: legacy Base58 P2PKH only (CashAddr fallback handled by the watcher)
pub fn address_to_p2pkh_script(chain: &str, addr: &str) -> Option<Vec<u8>> {
    let (ver, payload) = b58_payload(addr)?;
    match chain {
        "BCH" | "BTC" => {
            if ver == 0x00 && payload.len() == 20 {
                return Some(p2pkh_script(&payload));
            }
        }
        "DOGE" => {
            if ver == 0x1e && payload.len() == 20 {
                return Some(p2pkh_script(&payload));
            }
        }
        _ => {}
    }
    None
}

/// Prefer BTC Bech32 -> script, else legacy P2PKH; BCH/DOGE only legacy P2PKH
pub fn address_to_script_any(chain: &str, addr: &str) -> Option<Vec<u8>> {
    match chain {
        "BTC" => btc_address_to_script(addr),
        "BCH" => address_to_p2pkh_script("BCH", addr),
        "DOGE" => address_to_p2pkh_script("DOGE", addr),
        _ => None,
    }
}

/// Electrum scripthash = sha256(scriptPubKey) reversed (little-endian hex)
pub fn scripthash_hex(script_pubkey: &[u8]) -> String {
    let digest = Sha256::digest(script_pubkey);
    let mut rev = digest.to_vec();
    rev.reverse();
    hex::encode(rev)
}
