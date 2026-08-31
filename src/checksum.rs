//! Checksum primitives for Bitcoin address verification
//!
//! Implemented locally (no extra crates) to keep the dependency list and the
//! release binary small, in line with the rest of the project.
//!
//! Two schemes are covered:
//! - **Base58Check** for legacy `1...` (P2PKH) and `3...` (P2SH) addresses
//! - **Bech32 / Bech32m** (BIP-173 / BIP-350) for `bc1...` SegWit addresses

/// Bitcoin Base58 alphabet (`0`, `O`, `I` and `l` are excluded on purpose)
pub const BASE58_ALPHABET: &[u8; 58] =
    b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

/// Bech32 data-part alphabet (BIP-173)
pub const BECH32_ALPHABET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

/// Version byte of a mainnet P2PKH address (`1...`)
pub const P2PKH_VERSION: u8 = 0x00;

/// Version byte of a mainnet P2SH address (`3...`)
pub const P2SH_VERSION: u8 = 0x05;

const BECH32M_CONST: u32 = 0x2bc8_30a3;

/// SHA-256 round constants
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// Compute the SHA-256 digest of `data`
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    // Pad: 0x80, zeroes, then the message length in bits (big endian)
    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut msg = Vec::with_capacity(data.len() + 72);
    msg.extend_from_slice(data);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for (i, word) in w.iter_mut().take(16).enumerate() {
            *word = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut out = [0u8; 32];
    for (i, word) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

/// Check whether every character belongs to the Base58 alphabet
pub fn is_base58(input: &str) -> bool {
    !input.is_empty() && input.bytes().all(|b| BASE58_ALPHABET.contains(&b))
}

/// Decode a Base58 string into its byte representation
///
/// Returns `None` if the input contains a character outside the alphabet.
pub fn base58_decode(input: &str) -> Option<Vec<u8>> {
    let mut bytes: Vec<u8> = Vec::with_capacity(input.len());

    for ch in input.bytes() {
        let mut carry = BASE58_ALPHABET.iter().position(|&c| c == ch)? as u32;

        for byte in bytes.iter_mut().rev() {
            let value = (*byte as u32) * 58 + carry;
            *byte = (value & 0xff) as u8;
            carry = value >> 8;
        }
        while carry > 0 {
            bytes.insert(0, (carry & 0xff) as u8);
            carry >>= 8;
        }
    }

    // Each leading base58 '1' digit encodes one leading zero byte
    let leading_zeros = input.bytes().take_while(|&b| b == b'1').count();
    let mut decoded = vec![0u8; leading_zeros];
    decoded.extend_from_slice(&bytes);

    Some(decoded)
}

/// Verify the Base58Check checksum of an address
///
/// On success the version byte is returned ([`P2PKH_VERSION`] or [`P2SH_VERSION`]
/// for mainnet Bitcoin addresses).
pub fn base58check_verify(address: &str) -> Option<u8> {
    let decoded = base58_decode(address)?;

    // 1 version byte + 20 byte hash + 4 byte checksum
    if decoded.len() != 25 {
        return None;
    }

    let (payload, checksum) = decoded.split_at(21);
    if sha256(&sha256(payload))[..4] == *checksum {
        Some(payload[0])
    } else {
        None
    }
}

/// Bech32 checksum variant
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bech32Variant {
    /// BIP-173, used by witness version 0
    Bech32,
    /// BIP-350, used by witness version 1 and above
    Bech32m,
}

/// A decoded and verified SegWit address
#[derive(Debug, Clone)]
pub struct SegwitAddress {
    /// Human readable part (`bc` for Bitcoin mainnet)
    pub hrp: String,
    /// Witness version (0 for P2WPKH/P2WSH, 1 for P2TR)
    pub witness_version: u8,
    /// Length in bytes of the witness program
    pub program_len: usize,
    /// Checksum variant the address was encoded with
    pub variant: Bech32Variant,
}

fn bech32_polymod(values: &[u8]) -> u32 {
    const GEN: [u32; 5] = [0x3b6a57b2, 0x26508e6d, 0x1ea119fa, 0x3d4233dd, 0x2a1462b3];
    let mut chk: u32 = 1;

    for value in values {
        let top = chk >> 25;
        chk = ((chk & 0x1ff_ffff) << 5) ^ (*value as u32);
        for (i, gen) in GEN.iter().enumerate() {
            if (top >> i) & 1 == 1 {
                chk ^= gen;
            }
        }
    }
    chk
}

fn bech32_hrp_expand(hrp: &str) -> Vec<u8> {
    let mut expanded: Vec<u8> = hrp.bytes().map(|b| b >> 5).collect();
    expanded.push(0);
    expanded.extend(hrp.bytes().map(|b| b & 31));
    expanded
}

/// Regroup `data` from `from`-bit groups into `to`-bit groups
fn convert_bits(data: &[u8], from: u32, to: u32, pad: bool) -> Option<Vec<u8>> {
    let mut acc: u32 = 0;
    let mut bits: u32 = 0;
    let maxv = (1u32 << to) - 1;
    let mut out = Vec::with_capacity(data.len() * from as usize / to as usize + 1);

    for value in data {
        let value = *value as u32;
        if (value >> from) != 0 {
            return None;
        }
        acc = (acc << from) | value;
        bits += from;
        while bits >= to {
            bits -= to;
            out.push(((acc >> bits) & maxv) as u8);
        }
    }

    if pad {
        if bits > 0 {
            out.push(((acc << (to - bits)) & maxv) as u8);
        }
    } else if bits >= from || ((acc << (to - bits)) & maxv) != 0 {
        // Anything left over must be zero padding only
        return None;
    }

    Some(out)
}

/// Verify a Bech32/Bech32m encoded SegWit address (BIP-173 / BIP-350)
///
/// Mixed-case input is rejected, as required by the specification. The human
/// readable part is *not* restricted here; callers decide whether they accept
/// `bc` (mainnet) only.
pub fn bech32_verify(address: &str) -> Option<SegwitAddress> {
    if !(8..=90).contains(&address.len()) || !address.is_ascii() {
        return None;
    }

    let has_lower = address.bytes().any(|b| b.is_ascii_lowercase());
    let has_upper = address.bytes().any(|b| b.is_ascii_uppercase());
    if has_lower && has_upper {
        return None;
    }

    let address = address.to_ascii_lowercase();
    let separator = address.rfind('1')?;
    let hrp = &address[..separator];
    let data_part = &address[separator + 1..];
    if hrp.is_empty() || data_part.len() < 6 {
        return None;
    }
    if hrp.bytes().any(|b| !(33..=126).contains(&b)) {
        return None;
    }

    let mut data = Vec::with_capacity(data_part.len());
    for ch in data_part.bytes() {
        data.push(BECH32_ALPHABET.iter().position(|&c| c == ch)? as u8);
    }

    let mut values = bech32_hrp_expand(hrp);
    values.extend_from_slice(&data);
    let variant = match bech32_polymod(&values) {
        1 => Bech32Variant::Bech32,
        BECH32M_CONST => Bech32Variant::Bech32m,
        _ => return None,
    };

    // SegWit specific rules (BIP-141 / BIP-350)
    let witness_version = data[0];
    if witness_version > 16 {
        return None;
    }

    let program = convert_bits(&data[1..data.len() - 6], 5, 8, false)?;
    if !(2..=40).contains(&program.len()) {
        return None;
    }
    if witness_version == 0 {
        if program.len() != 20 && program.len() != 32 {
            return None;
        }
        if variant != Bech32Variant::Bech32 {
            return None;
        }
    } else if variant != Bech32Variant::Bech32m {
        return None;
    }

    Some(SegwitAddress {
        hrp: hrp.to_string(),
        witness_version,
        program_len: program.len(),
        variant,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    #[test]
    fn test_sha256_known_vectors() {
        assert_eq!(
            hex(&sha256(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hex(&sha256(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            hex(&sha256(b"The quick brown fox jumps over the lazy dog")),
            "d7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592"
        );
    }

    #[test]
    fn test_sha256_multi_block() {
        // 1000 bytes forces several compression rounds
        let data = vec![b'a'; 1000];
        assert_eq!(
            hex(&sha256(&data)),
            "41edece42d63e8d9bf515a9ba6932e1c20cbc9f5a5d134645adb5db1b9737ea3"
        );
    }

    #[test]
    fn test_base58_decode_leading_zeros() {
        assert_eq!(base58_decode("1").unwrap(), vec![0u8]);
        assert_eq!(base58_decode("11").unwrap(), vec![0u8, 0u8]);
        assert!(base58_decode("0OIl").is_none());
    }

    #[test]
    fn test_base58check_p2pkh() {
        // Genesis block coinbase address
        assert_eq!(
            base58check_verify("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"),
            Some(P2PKH_VERSION)
        );
    }

    #[test]
    fn test_base58check_p2sh() {
        assert_eq!(
            base58check_verify("3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy"),
            Some(P2SH_VERSION)
        );
    }

    #[test]
    fn test_base58check_rejects_typo() {
        // Same address with the last character changed - checksum must fail
        assert_eq!(
            base58check_verify("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNb"),
            None
        );
        assert_eq!(base58check_verify("not-an-address"), None);
    }

    #[test]
    fn test_bech32_v0_p2wpkh() {
        let decoded = bech32_verify("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq").unwrap();
        assert_eq!(decoded.hrp, "bc");
        assert_eq!(decoded.witness_version, 0);
        assert_eq!(decoded.program_len, 20);
        assert_eq!(decoded.variant, Bech32Variant::Bech32);
    }

    #[test]
    fn test_bech32m_v1_taproot() {
        // BIP-350 test vector
        let decoded =
            bech32_verify("bc1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vqzk5jj0")
                .unwrap();
        assert_eq!(decoded.witness_version, 1);
        assert_eq!(decoded.program_len, 32);
        assert_eq!(decoded.variant, Bech32Variant::Bech32m);
    }

    #[test]
    fn test_bech32_rejects_invalid() {
        // Mixed case
        assert!(bech32_verify("bc1QAR0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq").is_none());
        // Bad checksum (last character altered)
        assert!(bech32_verify("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdp").is_none());
        // Witness v0 encoded with bech32m (BIP-350 invalid vector)
        assert!(bech32_verify("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kemeawh").is_none());
    }
}
