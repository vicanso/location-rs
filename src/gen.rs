use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

#[derive(Default, Debug, Clone, PartialEq)]
enum LocationCategory {
    #[default]
    IPV4,
    IPV6,
}

#[derive(Default, Debug, Clone, PartialEq)]
struct Location {
    category: LocationCategory,
    begin: u128,
    end: u128,
    country: String,
    province: String,
    city: String,
}

fn parse_record(record: csv::StringRecord) -> Location {
    let mut category = LocationCategory::IPV4;
    let begin: u128;
    let end: u128;
    // IP开始 IP结束 国家 省 空列 市
    let ip = record.get(0).unwrap();
    // ipv6
    if ip.contains(':') {
        category = LocationCategory::IPV6;
        begin = Ipv6Addr::from_str(ip).unwrap().into();
        end = Ipv6Addr::from_str(record.get(1).unwrap()).unwrap().into();
    } else {
        let begin_value: u32 = Ipv4Addr::from_str(ip).unwrap().into();
        begin = begin_value as u128;
        let end_value: u32 = Ipv4Addr::from_str(record.get(1).unwrap()).unwrap().into();
        end = end_value as u128;
    }

    let country = record.get(2).unwrap().to_string();
    let province = record.get(3).unwrap().to_string();
    let city = record.get(5).unwrap().to_string();

    Location {
        category,
        begin,
        end,
        country,
        province,
        city,
    }
}

fn parse_record_from_file(file: &str, category: LocationCategory, max: i64) -> Vec<Location> {
    let mut result = vec![];
    let file = fs::File::open(file).unwrap();
    let reader = BufReader::new(file);

    let mut archive = zip::ZipArchive::new(reader).unwrap();

    let file = archive.by_index(0).unwrap();
    let mut rdr = csv::Reader::from_reader(file);
    let mut prev_end: u128 = 0;
    let value = parse_record(rdr.headers().unwrap().clone());
    // 上一次如果与本次的偏差大于1，则表示中间有部分IP无对应
    if prev_end < value.begin - 1 {
        result.push(Location {
            category: category.clone(),
            begin: prev_end + 1,
            end: value.begin - 1,
            ..Default::default()
        });
    }
    prev_end = value.end;
    result.push(value);
    for (index, item) in rdr.records().enumerate() {
        if max > 0 && index >= max as usize {
            break;
        }
        let record = item.unwrap();
        let value = parse_record(record);
        // 上一次如果与本次的偏差大于1，则表示中间有部分IP无对应
        if prev_end < value.begin - 1 {
            result.push(Location {
                category: category.clone(),
                begin: prev_end + 1,
                end: value.begin - 1,
                ..Default::default()
            });
        }
        prev_end = value.end;
        result.push(value);
    }
    result.sort_by(|a, b| a.end.cmp(&b.end));
    result
}

pub fn generate_ip_data(max: i64) {
    let mut country_list = Vec::new();
    let mut country_index_map = HashMap::new();
    // 第一个值为空值
    country_list.push("".to_string());

    let mut province_list = Vec::new();
    let mut province_index_map = HashMap::new();
    province_list.push("".to_string());

    let mut city_list = Vec::new();
    let mut city_index_map = HashMap::new();
    city_list.push("".to_string());

    let append_not_exists =
        |values: &mut Vec<String>, map: &mut HashMap<String, usize>, value: &String| -> usize {
            if let Some(index) = map.get(value) {
                return index.to_owned();
            }
            values.push(value.clone());
            let index = values.len() - 1;
            map.insert(value.to_string(), index);
            index
        };

    let mut records = parse_record_from_file(
        "./assets/geolite2-city-ipv4.csv.zip",
        LocationCategory::IPV4,
        max,
    );
    let ipv6_records = parse_record_from_file(
        "./assets/geolite2-city-ipv6.csv.zip",
        LocationCategory::IPV6,
        max,
    );

    println!("ipv4 total: {}", records.len());
    println!("ipv6 total: {}", ipv6_records.len());
    for item in ipv6_records {
        records.push(item);
    }

    let mut ipv4_data: Vec<u32> = vec![];
    let mut ipv4_location_data: Vec<Vec<usize>> = vec![];
    let mut ipv6_data: Vec<u128> = vec![];
    let mut ipv6_location_data: Vec<Vec<usize>> = vec![];
    for item in records.iter() {
        let country_index =
            append_not_exists(&mut country_list, &mut country_index_map, &item.country);
        let province_index =
            append_not_exists(&mut province_list, &mut province_index_map, &item.province);
        let city_index = append_not_exists(&mut city_list, &mut city_index_map, &item.city);

        let location_data = vec![country_index, province_index, city_index];
        if item.category == LocationCategory::IPV4 {
            ipv4_data.push(item.end as u32);
            ipv4_location_data.push(location_data);
        } else {
            ipv6_data.push(item.end);
            ipv6_location_data.push(location_data);
        }
    }
    println!("country total: {}", country_list.len());
    println!("province total: {}", province_list.len());
    println!("city total: {}", city_list.len());

    let country_data = serde_json::to_string(&country_list).unwrap();
    let country_code = format!(
        "pub static COUNTRY_LIST: [&str; {}] = {country_data};",
        country_list.len()
    );

    let province_data = serde_json::to_string(&province_list).unwrap();
    let province_code = format!(
        "pub static PROVINCE_LIST: [&str; {}] = {province_data};",
        province_list.len()
    );

    let city_data = serde_json::to_string(&city_list).unwrap();
    let city_code = format!(
        "pub static CITY_LIST: [&str; {}] = {city_data};",
        city_list.len()
    );

    let ipv4_code = format!(
        "pub static IPV4_LIST: [u32; {}] = {};",
        ipv4_data.len(),
        serde_json::to_string(&ipv4_data).unwrap()
    );

    let ipv4_location_code = format!(
        "pub static IPV4_LOCATION_LIST: [[usize; 3]; {}] = {};",
        ipv4_location_data.len(),
        serde_json::to_string(&ipv4_location_data).unwrap()
    );

    let ipv6_code = format!(
        "pub static IPV6_LIST: [u128; {}] = {};",
        ipv6_data.len(),
        serde_json::to_string(&ipv6_data).unwrap()
    );

    let ipv6_location_code = format!(
        "pub static IPV6_LOCATION_LIST: [[usize; 3]; {}] = {};",
        ipv6_location_data.len(),
        serde_json::to_string(&ipv6_location_data).unwrap()
    );

    let filename = "./src/ip_data.rs";
    let _ = fs::remove_file(filename);

    let mut file = File::create(filename).unwrap();

    let data = vec![
        country_code,
        province_code,
        city_code,
        ipv4_code,
        ipv4_location_code,
        ipv6_code,
        ipv6_location_code,
    ]
    .join("\n\n\n");
    file.write_all(data.as_bytes()).unwrap();
}
