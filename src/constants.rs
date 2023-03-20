use ethers::{
    abi::AbiEncode,
    types::{Bytes, H160},
};
use lazy_static::lazy_static;

use crate::ClaimCall;

lazy_static! {
    pub static ref ARB_ADDRESS: H160 = "0x912ce59144191c1204e64559fe8253a0e49e6548"
        .parse()
        .unwrap();
    pub static ref DISTRIBUTOR_ADDRESS: H160 = "0x67a24ce4321ab3af51c2d0a4801c3e111d88c9d9"
        .parse()
        .unwrap();
    pub static ref CLAIM_DATA: Bytes = AbiEncode::encode(ClaimCall).into();
}
