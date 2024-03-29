use alloy_primitives::U256;
use ethers::signers::{coins_bip39::English, LocalWallet, MnemonicBuilder, Signer, Wallet};
use ethers_core::k256::ecdsa::SigningKey;
use ethers_core::types::H160;
use hdwallet::{ChainPath, DefaultKeyChain, KeyChain};
use std::{
    collections::{HashMap, HashSet},
    fmt,
    fs::File,
    io::{Read, Write},
    iter, str,
};

use crate::errors::Error;

/// Build Wallet from Private key or Mnemonic
pub fn build_wallet(value: &str) -> Result<Wallet<SigningKey>, Error> {
    value
        .parse::<LocalWallet>()
        .or(MnemonicBuilder::<English>::default()
            .phrase(value)
            .build()
            .map_err(Error::WalletError))
}

/// Get wallet public address to String
pub fn wallet_address(wallet: &Wallet<SigningKey>) -> String {
    format!("{:?}", wallet.address())
}

/// Validate that private key as an Eth wallet
pub fn parse_key(value: &str) -> Result<String, Error> {
    let wallet = build_wallet(value)?;
    let address = wallet_address(&wallet);
    tracing::trace!(address, "Resolved wallet address");
    Ok(String::from(value))
}

/// Validate that private key as an Eth wallet
pub fn public_key(value: &str) -> Result<String, Error> {
    let wallet = build_wallet(value)?;
    let addr = wallet_address(&wallet);
    tracing::trace!(address = addr, "Resolved wallet address");
    Ok(addr)
}

// Given a HD wallet, utilize the additional paths to generate child wallet,
// return child wallet public key, private key, wallet address
pub fn derive_key_pair(
    key_chain: &DefaultKeyChain,
    epoch: u64,
    qm_hash: &str,
    index: u64,
) -> Result<(String, H160), Error> {
    let path = format!(
        "m/{}",
        std::iter::once(epoch.to_string())
            .chain(qm_hash.as_bytes().iter().map(|b| b.to_string()))
            .chain(std::iter::once(index.to_string()))
            .collect::<Vec<_>>()
            .join("/")
    );
    let chain_path = ChainPath::from(path);

    let (derived_key, _) = key_chain
        .derive_private_key(chain_path)
        .map_err(|e| Error::ContractError(e.to_string()))?;
    let private_key = derived_key.private_key.display_secret().to_string();
    let wallet = build_wallet(&private_key)?;

    Ok((private_key, wallet.address()))
}

/* Token unit and formatting */
const ONE_18: u128 = 1_000_000_000_000_000_000;

/// GRT with 18 fractional digits
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct GRT(pub UDecimal18);

/// Represents a positive decimal value with 18 fractional digits precision. Using U256 as storage.
#[derive(Copy, Clone, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct UDecimal18(U256);

impl From<U256> for UDecimal18 {
    fn from(value: U256) -> Self {
        Self(U256::from(value) * U256::from(ONE_18))
    }
}

impl From<u128> for UDecimal18 {
    fn from(value: u128) -> Self {
        Self::from(U256::from(value))
    }
}

impl TryFrom<f64> for UDecimal18 {
    type Error = <U256 as TryFrom<f64>>::Error;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        U256::try_from(value * 1e18).map(Self)
    }
}

impl From<UDecimal18> for f64 {
    fn from(value: UDecimal18) -> Self {
        f64::from(value.0) * 1e-18
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct InvalidDecimalString;

impl fmt::Display for InvalidDecimalString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid decimal string")
    }
}

impl str::FromStr for UDecimal18 {
    type Err = InvalidDecimalString;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // We require at least one ASCII digit. Otherwise `U256::from_str_radix` will return 0 for
        // some inputs we consider invalid.
        if !s.chars().any(|c: char| -> bool { c.is_ascii_digit() }) {
            return Err(InvalidDecimalString);
        }
        let (int, frac) = s.split_at(s.chars().position(|c| c == '.').unwrap_or(s.len()));
        let p = 18;
        let digits = int
            .chars()
            // append fractional digits (after decimal point)
            .chain(frac.chars().skip(1).chain(iter::repeat('0')).take(p))
            .collect::<String>();
        Ok(UDecimal18(
            U256::from_str_radix(&digits, 10).map_err(|_| InvalidDecimalString)?,
        ))
    }
}

impl fmt::Display for UDecimal18 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 == U256::from(0) {
            return write!(f, "0");
        }
        let p = 18;
        let digits = self.0.to_string().into_bytes();
        let ctz = digits
            .iter()
            .rev()
            .take_while(|&&b| b == b'0')
            .count()
            .min(p);
        if digits.len() < p {
            let fill = "0".repeat(p - digits.len());
            let frac = &digits[0..digits.len() - ctz];
            write!(f, "0.{}{}", fill, unsafe { str::from_utf8_unchecked(frac) })
        } else {
            let (mut int, mut frac) = digits.split_at(digits.len() - p);
            frac = &frac[0..frac.len() - ctz];
            if int.is_empty() {
                int = &[b'0'];
            }
            if ctz == p {
                write!(f, "{}", unsafe { str::from_utf8_unchecked(int) })
            } else {
                write!(
                    f,
                    "{}.{}",
                    unsafe { str::from_utf8_unchecked(int) },
                    unsafe { str::from_utf8_unchecked(frac) }
                )
            }
        }
    }
}

impl fmt::Debug for UDecimal18 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl UDecimal18 {
    /// This will use the value of the given U256 directly, without scaling by 1e18.
    pub fn from_raw_u256(value: U256) -> Self {
        Self(value)
    }

    pub fn raw_u256(&self) -> &U256 {
        &self.0
    }

    pub fn as_u128(&self) -> Option<u128> {
        if self.0 % U256::from(ONE_18) > U256::ZERO {
            return None;
        }
        let inner = self.0 / U256::from(ONE_18);
        inner.try_into().ok()
    }

    pub fn saturating_add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }
}

impl std::ops::Add for UDecimal18 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.add(rhs.0))
    }
}

impl std::ops::Mul for UDecimal18 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self((self.0 * rhs.0) / U256::from(ONE_18))
    }
}

impl std::ops::Div for UDecimal18 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self((self.0 * U256::from(ONE_18)) / rhs.0)
    }
}

impl std::iter::Sum for UDecimal18 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self(iter.map(|u| u.0).sum())
    }
}

// Takes a HashMap<String, Vec<u64>> and writes it to a specified file in JSON format.
pub fn store_map_as_json(
    map: &HashMap<String, HashSet<u64>>,
    file_path: &str,
) -> Result<(), Error> {
    let serialized = serde_json::to_string(map).map_err(Error::JsonError)?;

    let mut file = File::create(file_path).map_err(Error::FileIOError)?;

    file.write_all(serialized.as_bytes())
        .map_err(Error::FileIOError)?;

    Ok(())
}

// Reads a JSON file and converts it into a HashMap<String, HashSet<u64>>.
pub fn read_json_to_map(file_path: &str) -> Result<HashMap<String, HashSet<u64>>, Error> {
    match File::open(file_path) {
        Ok(mut file) => {
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .map_err(Error::FileIOError)?;

            let temp_map: HashMap<String, Vec<u64>> =
                serde_json::from_str(&contents).map_err(Error::JsonError)?;

            let map: HashMap<String, HashSet<u64>> = temp_map
                .into_iter()
                .map(|(key, vec)| (key, vec.into_iter().collect::<HashSet<u64>>()))
                .collect();

            Ok(map)
        }
        Err(e) => {
            tracing::info!("Failed to open file at '{}'. Error: {}.", file_path, e);
            Ok(HashMap::new())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn u256_from_str() {
        assert_eq!("100".parse::<U256>().unwrap(), U256::from(100));
        assert_eq!("0x100".parse::<U256>().unwrap(), U256::from(256));
    }

    #[test]
    fn udecimal18_from_str() {
        let tests: &[(&str, Option<(&str, u128)>)] = &[
            ("", None),
            ("?", None),
            (".", None),
            ("1.1.1", None),
            ("10.10?1", None),
            ("1?0.01", None),
            ("0", Some(("0", 0))),
            ("0.0", Some(("0", 0))),
            (".0", Some(("0", 0))),
            ("0.", Some(("0", 0))),
            ("00.00", Some(("0", 0))),
            ("1", Some(("1", ONE_18))),
            ("1.0", Some(("1", ONE_18))),
            ("1.", Some(("1", ONE_18))),
            ("0.1", Some(("0.1", ONE_18 / 10))),
            (".1", Some(("0.1", ONE_18 / 10))),
            ("0.0000000000000000012", Some(("0.000000000000000001", 1))),
            ("0.001001", Some(("0.001001", 1_001_000_000_000_000))),
            ("0.001", Some(("0.001", ONE_18 / 1_000))),
            ("100.001", Some(("100.001", 100_001_000_000_000_000_000))),
            ("100.000", Some(("100", 100 * ONE_18))),
            ("123.0", Some(("123", 123 * ONE_18))),
            ("123", Some(("123", 123 * ONE_18))),
            (
                "123456789123456789.123456789123456789123456789",
                Some((
                    "123456789123456789.123456789123456789",
                    123_456_789_123_456_789_123_456_789_123_456_789,
                )),
            ),
        ];
        for (input, expected) in tests {
            let output = input.parse::<UDecimal18>();
            println!("\"{input}\" => {output:?}");
            match expected {
                &Some((repr, internal)) => {
                    assert_eq!(output.as_ref().map(|d| d.0), Ok(U256::from(internal)));
                    assert_eq!(output.as_ref().map(ToString::to_string), Ok(repr.into()));
                }
                None => assert_eq!(output, Err(InvalidDecimalString)),
            }
        }
    }

    #[test]
    fn udecimal_from_f64() {
        let tests = [
            0.0,
            0.5,
            0.01,
            0.0042,
            1.0,
            123.456,
            1e14,
            1e17,
            1e18,
            1e19,
            2.0f64.powi(128) - 1.0,
            2.0f64.powi(128),
            1e26,
            1e27,
            1e28,
            1e29,
            1e30,
            1e31,
            1e32,
        ];
        for test in tests {
            let expected = (test * 1e18_f64).floor();
            let decimal = UDecimal18::try_from(test).unwrap();
            let output = decimal.0.to_string().parse::<f64>().unwrap();
            let error = (expected - output).abs() / expected.max(1e-30);
            println!(
                "expected: {}\n decimal: {}\n   error: {:.3}%\n---",
                expected / 1e18,
                decimal,
                error * 100.0
            );
            assert!(error < 0.005);
        }
    }
}
