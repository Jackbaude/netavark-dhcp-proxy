use ipnet::PrefixLenError;
use netavark::error::NetavarkError;
use std::num::ParseIntError;
use std::str::FromStr;
use std::string::ToString;
use tonic::{Code, Status};

use crate::NetworkConfig;
impl FromStr for NetworkConfig {
    type Err = ParseIntError;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        // s is actually a string so if we intend to generate
        // a `NetworkConfig` object from `s` parse `s` and populate
        // instead of default empty values
        Ok(NetworkConfig {
            iface: "".to_string(),
            mac_addr: "".to_string(),
            domain_name: "".to_string(),
            host_name: "".to_string(),
            version: 0,
            ns_path: "".to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct ProxyError(String);

pub trait CustomErr {
    fn new(msg: String) -> Self;
}

impl CustomErr for ProxyError {
    fn new(msg: String) -> Self {
        ProxyError(msg)
    }
}

impl ToString for ProxyError {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl From<ProxyError> for Status {
    fn from(pe: ProxyError) -> Self {
        Status::new(Code::Unknown, pe.to_string())
    }
}

impl From<NetavarkError> for ProxyError {
    fn from(cause: NetavarkError) -> Self {
        ProxyError::new(cause.to_string())
    }
}

impl From<PrefixLenError> for ProxyError {
    fn from(cause: PrefixLenError) -> Self {
        ProxyError::new(cause.to_string())
    }
}