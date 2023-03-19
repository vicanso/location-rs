use crate::ip_data;
use serde::{Deserialize, Serialize};
use std::net::{AddrParseError, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    ip: String,
    country: String,
    province: String,
    city: String,
}
fn get_country(index: usize) -> String {
    if let Some(value) = ip_data::COUNTRY_LIST.get(index) {
        return value.to_string();
    }
    "".to_string()
}
fn get_province(index: usize) -> String {
    if let Some(value) = ip_data::PROVINCE_LIST.get(index) {
        return value.to_string();
    }
    "".to_string()
}
fn get_city(index: usize) -> String {
    if let Some(value) = ip_data::CITY_LIST.get(index) {
        return value.to_string();
    }
    "".to_string()
}

fn get_location_info(data: Option<&[usize; 3]>) -> Location {
    if let Some(value) = data {
        return Location {
            country: get_country(value[0]),
            province: get_province(value[1]),
            city: get_city(value[2]),
            ..Default::default()
        };
    }
    Location::default()
}

pub fn get_location(ip: &str) -> Result<Location, AddrParseError> {
    let mut result = if ip.contains(':') {
        let addr = Ipv6Addr::from_str(ip)?;
        let value: u128 = addr.into();
        let index = ip_data::IPV6_LIST
            .binary_search(&value)
            .unwrap_or_else(|index| index);
        get_location_info(ip_data::IPV6_LOCATION_LIST.get(index))
    } else {
        let addr = Ipv4Addr::from_str(ip)?;
        let value: u32 = addr.into();
        let index = ip_data::IPV4_LIST
            .binary_search(&value)
            .unwrap_or_else(|index| index);
        get_location_info(ip_data::IPV4_LOCATION_LIST.get(index))
    };
    result.ip = ip.to_string();
    Ok(result)
}
