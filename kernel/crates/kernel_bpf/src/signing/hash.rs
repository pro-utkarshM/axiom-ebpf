//! Cryptographic hashing for BPF programs.
//!
//! Uses SHA3-256 (Keccak) for program integrity verification.

/// Length of SHA3-256 hash in bytes.
pub const SHA3_256_LEN: usize = 32;

/// SHA3-256 hash of a BPF program.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ProgramHash([u8; SHA3_256_LEN]);

impl ProgramHash {
    /// Create a hash from raw bytes.
    pub const fn from_bytes(bytes: [u8; SHA3_256_LEN]) -> Self {
        Self(bytes)
    }

    /// Create a hash from a byte slice.
    ///
    /// Returns `None` if the slice is not exactly 32 bytes.
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() != SHA3_256_LEN {
            return None;
        }
        let mut bytes = [0u8; SHA3_256_LEN];
        bytes.copy_from_slice(slice);
        Some(Self(bytes))
    }

    /// Compute the SHA3-256 hash of the given data.
    ///
    /// This is a pure Rust implementation suitable for no_std environments.
    pub fn compute(data: &[u8]) -> Self {
        Self(keccak256(data))
    }

    /// Get the hash as a byte slice.
    pub fn as_bytes(&self) -> &[u8; SHA3_256_LEN] {
        &self.0
    }

    /// Check if this hash matches another.
    pub fn matches(&self, other: &Self) -> bool {
        // Constant-time comparison to prevent timing attacks
        let mut result = 0u8;
        for (a, b) in self.0.iter().zip(other.0.iter()) {
            result |= a ^ b;
        }
        result == 0
    }
}

impl core::fmt::Debug for ProgramHash {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ProgramHash(")?;
        for byte in &self.0[..4] {
            write!(f, "{byte:02x}")?;
        }
        write!(f, "...")?;
        for byte in &self.0[28..] {
            write!(f, "{byte:02x}")?;
        }
        write!(f, ")")
    }
}

/// Keccak-256 (SHA3-256) implementation.
///
/// This is a minimal implementation suitable for no_std kernel environments.
fn keccak256(data: &[u8]) -> [u8; 32] {
    const ROUNDS: usize = 24;
    const RATE: usize = 136; // 1088 bits for SHA3-256

    // Round constants
    const RC: [u64; 24] = [
        0x0000000000000001,
        0x0000000000008082,
        0x800000000000808a,
        0x8000000080008000,
        0x000000000000808b,
        0x0000000080000001,
        0x8000000080008081,
        0x8000000000008009,
        0x000000000000008a,
        0x0000000000000088,
        0x0000000080008009,
        0x000000008000000a,
        0x000000008000808b,
        0x800000000000008b,
        0x8000000000008089,
        0x8000000000008003,
        0x8000000000008002,
        0x8000000000000080,
        0x000000000000800a,
        0x800000008000000a,
        0x8000000080008081,
        0x8000000000008080,
        0x0000000080000001,
        0x8000000080008008,
    ];

    // Rotation offsets
    const ROTC: [u32; 24] = [
        1, 3, 6, 10, 15, 21, 28, 36, 45, 55, 2, 14, 27, 41, 56, 8, 25, 43, 62, 18, 39, 61, 20, 44,
    ];

    // Position indices
    const PILN: [usize; 24] = [
        10, 7, 11, 17, 18, 3, 5, 16, 8, 21, 24, 4, 15, 23, 19, 13, 12, 2, 20, 14, 22, 9, 6, 1,
    ];

    let mut state = [0u64; 25];

    // Absorb phase
    let mut offset = 0;
    while offset + RATE <= data.len() {
        for i in 0..(RATE / 8) {
            let bytes: [u8; 8] = data[offset + i * 8..offset + i * 8 + 8].try_into().unwrap();
            state[i] ^= u64::from_le_bytes(bytes);
        }
        keccak_f(&mut state, &RC, &ROTC, &PILN, ROUNDS);
        offset += RATE;
    }

    // Padding
    let remaining = data.len() - offset;
    let mut padded = [0u8; RATE];
    padded[..remaining].copy_from_slice(&data[offset..]);
    padded[remaining] = 0x06; // SHA3 domain separator
    padded[RATE - 1] |= 0x80;

    for i in 0..(RATE / 8) {
        let bytes: [u8; 8] = padded[i * 8..i * 8 + 8].try_into().unwrap();
        state[i] ^= u64::from_le_bytes(bytes);
    }
    keccak_f(&mut state, &RC, &ROTC, &PILN, ROUNDS);

    // Squeeze phase (only need 256 bits = 32 bytes)
    let mut output = [0u8; 32];
    for i in 0..4 {
        output[i * 8..(i + 1) * 8].copy_from_slice(&state[i].to_le_bytes());
    }

    output
}

/// Keccak-f permutation.
#[inline(never)]
fn keccak_f(
    state: &mut [u64; 25],
    rc: &[u64; 24],
    rotc: &[u32; 24],
    piln: &[usize; 24],
    rounds: usize,
) {
    for round_constant in rc.iter().take(rounds) {
        // Theta
        let mut c = [0u64; 5];
        for x in 0..5 {
            c[x] = state[x] ^ state[x + 5] ^ state[x + 10] ^ state[x + 15] ^ state[x + 20];
        }
        let mut d = [0u64; 5];
        for x in 0..5 {
            d[x] = c[(x + 4) % 5] ^ c[(x + 1) % 5].rotate_left(1);
        }
        for x in 0..5 {
            for y in 0..5 {
                state[x + y * 5] ^= d[x];
            }
        }

        // Rho and Pi
        let mut t = state[1];
        for i in 0..24 {
            let j = piln[i];
            let temp = state[j];
            state[j] = t.rotate_left(rotc[i]);
            t = temp;
        }

        // Chi
        for y in 0..5 {
            let mut temp = [0u64; 5];
            for x in 0..5 {
                temp[x] = state[x + y * 5];
            }
            for x in 0..5 {
                state[x + y * 5] = temp[x] ^ ((!temp[(x + 1) % 5]) & temp[(x + 2) % 5]);
            }
        }

        // Iota
        state[0] ^= *round_constant;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_hash() {
        // SHA3-256("") = a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a
        let hash = ProgramHash::compute(b"");
        let expected: [u8; 32] = [
            0xa7, 0xff, 0xc6, 0xf8, 0xbf, 0x1e, 0xd7, 0x66, 0x51, 0xc1, 0x47, 0x56, 0xa0, 0x61,
            0xd6, 0x62, 0xf5, 0x80, 0xff, 0x4d, 0xe4, 0x3b, 0x49, 0xfa, 0x82, 0xd8, 0x0a, 0x4b,
            0x80, 0xf8, 0x43, 0x4a,
        ];
        assert_eq!(hash.as_bytes(), &expected);
    }

    #[test]
    fn known_hash() {
        // SHA3-256("abc") = 3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532
        let hash = ProgramHash::compute(b"abc");
        let expected: [u8; 32] = [
            0x3a, 0x98, 0x5d, 0xa7, 0x4f, 0xe2, 0x25, 0xb2, 0x04, 0x5c, 0x17, 0x2d, 0x6b, 0xd3,
            0x90, 0xbd, 0x85, 0x5f, 0x08, 0x6e, 0x3e, 0x9d, 0x52, 0x5b, 0x46, 0xbf, 0xe2, 0x45,
            0x11, 0x43, 0x15, 0x32,
        ];
        assert_eq!(hash.as_bytes(), &expected);
    }

    #[test]
    fn hash_matches() {
        let hash1 = ProgramHash::compute(b"test data");
        let hash2 = ProgramHash::compute(b"test data");
        let hash3 = ProgramHash::compute(b"other data");

        assert!(hash1.matches(&hash2));
        assert!(!hash1.matches(&hash3));
    }

    #[test]
    fn from_slice() {
        let bytes = [0u8; 32];
        assert!(ProgramHash::from_slice(&bytes).is_some());
        assert!(ProgramHash::from_slice(&bytes[..31]).is_none());
        assert!(ProgramHash::from_slice(&[]).is_none());
    }
}
