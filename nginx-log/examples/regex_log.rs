use std::{net::IpAddr, time::Duration};

use chrono::{DateTime, NaiveDateTime, Utc};
use regex::Regex;

const DATA: &str = "[GIN] 2024/08/28 - 19:00:05 | 200 |      70.893µs |  47.100.221.202 | POST     /tool_api/get_fallguys_log_url";
const REGEX_EXP: &str =
    r#"\[([A-z]+)\]\s(.{21})\s\|\s([0-9]+)\s\|\s([0-9,.]+µs)\s\|\s([0-9,.]+)\s\|\s([A-Z]+)\s(.+)"#;

#[derive(Debug)]
enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Connect,
    Trace,
    Patch,
}

#[derive(Debug)]
#[allow(unused)]
struct GinLog {
    time: DateTime<Utc>,
    status: u16,
    duration: Duration,
    remote_addr: IpAddr,
    method: HttpMethod,
    url: String,
}

impl From<&str> for HttpMethod {
    fn from(value: &str) -> Self {
        match value.to_uppercase().as_str() {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            "HEAD" => HttpMethod::Head,
            "OPTIONS" => HttpMethod::Options,
            "CONNECT" => HttpMethod::Connect,
            "TRACE" => HttpMethod::Trace,
            "PATCH" => HttpMethod::Patch,
            _ => unreachable!(),
        }
    }
}

impl From<[&str; 7]> for GinLog {
    fn from(value: [&str; 7]) -> Self {
        let naive_datetime =
            NaiveDateTime::parse_from_str(value[1], "%Y/%m/%d - %H:%M:%S").unwrap();
        let datetime_utc: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive_datetime, Utc);

        GinLog {
            time: datetime_utc,
            status: value[2].parse().unwrap(),
            duration: parse_duration(value[3]).unwrap(),
            remote_addr: value[4].parse().unwrap(),
            method: HttpMethod::from(value[5]),
            url: value[6].to_string(),
        }
    }
}

fn parse_duration(input: &str) -> Result<Duration, &'static str> {
    // 判断输入的单位并进行相应的转换
    if input.ends_with("ns") {
        // 纳秒
        let value: u64 = input
            .trim_end_matches("ns")
            .parse()
            .map_err(|_| "Invalid format")?;
        Ok(Duration::from_nanos(value))
    } else if input.ends_with("µs") {
        // 微秒
        let value: f64 = input
            .trim_end_matches("µs")
            .parse()
            .map_err(|_| "Invalid format")?;
        let nanos = (value * 1_000.0).round() as u64;
        Ok(Duration::from_nanos(nanos))
    } else if input.ends_with("ms") {
        // 毫秒
        let value: f64 = input
            .trim_end_matches("ms")
            .parse()
            .map_err(|_| "Invalid format")?;
        let nanos = (value * 1_000_000.0).round() as u64;
        Ok(Duration::from_nanos(nanos))
    } else if input.ends_with("s") {
        // 秒
        let value: f64 = input
            .trim_end_matches("s")
            .parse()
            .map_err(|_| "Invalid format")?;
        let nanos = (value * 1_000_000_000.0).round() as u64;
        Ok(Duration::from_nanos(nanos))
    } else if input.ends_with("m") {
        // 分钟
        let value: f64 = input
            .trim_end_matches("m")
            .parse()
            .map_err(|_| "Invalid format")?;
        let seconds = (value * 60.0).round() as u64;
        Ok(Duration::from_secs(seconds))
    } else if input.ends_with("h") {
        // 小时
        let value: f64 = input
            .trim_end_matches("h")
            .parse()
            .map_err(|_| "Invalid format")?;
        let seconds = (value * 3600.0).round() as u64;
        Ok(Duration::from_secs(seconds))
    } else {
        Err("Unsupported unit")
    }
}

fn main() -> anyhow::Result<()> {
    let re = Regex::new(r"\s+").unwrap(); // 匹配一个或多个空白字符
    let data = re.replace_all(DATA, " ").into_owned();
    println!("{data}");
    let re = Regex::new(REGEX_EXP)?;
    let res: (&str, [&str; 7]) = re.captures(&data).unwrap().extract();
    println!("{:#?}", GinLog::from(res.1));
    println!("{:?}", GinLog::from(res.1).duration);
    Ok(())
}
