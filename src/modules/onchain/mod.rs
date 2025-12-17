mod errors;
mod implementation;
mod types;

pub use errors::AddressError;
pub use implementation::BitcoinAddressValidator;
pub use types::{
    AddressType, GetAddressResponse, GetAddressesResponse, Network, ValidationResult, WordCount,
};

#[cfg(test)]
mod tests;
