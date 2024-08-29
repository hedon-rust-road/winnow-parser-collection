#![allow(unused)]
use std::{
    fs::File,
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
    sync::Arc,
};

use anyhow::anyhow;
use arrow::{
    array::{Array, Int64Array, RecordBatch, StringArray, UInt16Array, UInt64Array},
    datatypes::{DataType, Field, Schema},
};
use chrono::{format::Pad, DateTime, Utc};
use parquet::{
    arrow::ArrowWriter,
    column::writer::ColumnWriter,
    data_type::ByteArray,
    file::{properties::WriterProperties, writer::SerializedFileWriter},
    schema::parser::parse_message_type,
};
use strum_macros::Display;
use winnow::{
    ascii::{digit1, space0},
    combinator::{alt, delimited, separated, terminated},
    token::take_until,
    PResult, Parser,
};

#[derive(Debug, PartialEq, Eq, Display)]
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

#[derive(Debug, PartialEq, Eq, Display)]
enum HttpProto {
    HTTP1_0,
    HTTP1_1,
    HTTP2_0,
    HTTP3_0,
}

#[allow(unused)]
#[derive(Debug)]
struct NginxLog {
    addr: IpAddr,
    datetime: DateTime<Utc>,
    method: HttpMethod,
    url: String,
    protocol: HttpProto,
    status: u16,
    body_bytes: u64,
    referer: String,
    user_agent: String,
}

// we need to parse:
// 93.180.71.3 - - [17/May/2015:08:05:32 +0000] "GET /downloads/product_1 HTTP/1.1" 304 0 "-" "Debian APT-HTTP/1.3 (0.8.16~exp12ubuntu10.21)"
// with winnow parser combinator
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("{:?}", parse_one_nginx_log().await?);

    let logs = parse_nginx_logs().await?;
    println!("{:?}", logs[1]);

    let filename = write_logs_to_parquet(logs)?;
    println!("{}", filename);
    Ok(())
}

fn write_logs_to_parquet(logs: Vec<NginxLog>) -> anyhow::Result<String> {
    let schema = Schema::new(vec![
        Field::new("addr", DataType::Utf8, false),
        Field::new("datetime", DataType::Int64, false),
        Field::new("method", DataType::Utf8, false),
        Field::new("url", DataType::Utf8, false),
        Field::new("protocol", DataType::Utf8, false),
        Field::new("status", DataType::UInt16, false),
        Field::new("body_bytes", DataType::UInt64, true),
        Field::new("referer", DataType::Utf8, true),
        Field::new("user_agent", DataType::Utf8, true),
    ]);

    let filename = "nginx_logs.parquet";
    let file = File::create(filename)?;
    let mut writer = ArrowWriter::try_new(file, Arc::new(schema), None)?;

    let addrs = logs
        .iter()
        .map(|v| v.addr.to_string())
        .collect::<Vec<String>>();

    let datetimes = logs
        .iter()
        .map(|v| v.datetime.timestamp())
        .collect::<Vec<i64>>();

    let methods = logs
        .iter()
        .map(|v| v.method.to_string())
        .collect::<Vec<String>>();

    let urls = logs
        .iter()
        .map(|v| v.url.to_string())
        .collect::<Vec<String>>();

    let protocols = logs
        .iter()
        .map(|v| v.protocol.to_string())
        .collect::<Vec<String>>();

    let body_bytes = logs.iter().map(|v| v.body_bytes).collect::<Vec<u64>>();

    let status = logs.iter().map(|v| v.status).collect::<Vec<u16>>();

    let referer = logs
        .iter()
        .map(|v| v.referer.to_string())
        .collect::<Vec<String>>();

    let user_agents = logs
        .iter()
        .map(|v| v.user_agent.to_string())
        .collect::<Vec<String>>();

    let batch = RecordBatch::try_from_iter(vec![
        ("addr", Arc::new(StringArray::from(addrs)) as Arc<dyn Array>),
        (
            "datetime",
            Arc::new(Int64Array::from(datetimes)) as Arc<dyn Array>,
        ),
        (
            "method",
            Arc::new(StringArray::from(methods)) as Arc<dyn Array>,
        ),
        ("url", Arc::new(StringArray::from(urls)) as Arc<dyn Array>),
        (
            "protocol",
            Arc::new(StringArray::from(protocols)) as Arc<dyn Array>,
        ),
        (
            "status",
            Arc::new(UInt16Array::from(status)) as Arc<dyn Array>,
        ),
        (
            "body_bytes",
            Arc::new(UInt64Array::from(body_bytes)) as Arc<dyn Array>,
        ),
        (
            "referer",
            Arc::new(StringArray::from(referer)) as Arc<dyn Array>,
        ),
        (
            "user_agent",
            Arc::new(StringArray::from(user_agents)) as Arc<dyn Array>,
        ),
    ])?;

    writer.write(&batch)?;
    writer.close()?;

    Ok(filename.to_string())
}

async fn parse_nginx_logs() -> anyhow::Result<Vec<NginxLog>> {
    let nginx_log_url = "https://raw.githubusercontent.com/elastic/examples/master/Common Data Formats/nginx_logs/nginx_logs";
    let nginx_log = reqwest::get(nginx_log_url).await?.text().await?;
    let logs = nginx_log
        .lines()
        .filter_map(|v| parse_nginx_log(v).ok())
        .collect::<Vec<_>>();
    Ok(logs)
}

async fn parse_one_nginx_log() -> anyhow::Result<NginxLog> {
    let s = r#"93.180.71.3 - - [17/May/2015:08:05:32 +0000] "GET /downloads/product_1 HTTP/1.1" 304 0 "-" "Debian APT-HTTP/1.3 (0.8.16~exp12ubuntu10.21)""#;
    let log = parse_nginx_log(s).map_err(|e| anyhow!("Failed to parse log: {:?}", e))?;
    Ok(log)
}

fn parse_nginx_log(s: &str) -> PResult<NginxLog> {
    let input = &mut (&*s);
    let ip = parse_ip(input)?;
    parse_ignored(input)?;
    parse_ignored(input)?;
    let datetime = parse_datetime(input)?;
    let (method, url, protocol) = parse_http(input)?;
    let status = parse_http_status(input)?;
    let body_bytes = parse_http_body_bytes(input)?;
    let referer = parse_quoted_string(input)?;
    let user_agent = parse_quoted_string(input)?;
    Ok(NginxLog {
        addr: ip,
        datetime,
        method,
        url,
        protocol,
        status,
        body_bytes,
        referer,
        user_agent,
    })
}

fn parse_ip(s: &mut &str) -> PResult<IpAddr> {
    let res: Vec<u8> = separated(4, digit1.parse_to::<u8>(), ".").parse_next(s)?;
    space0(s)?;
    Ok(IpAddr::V4(Ipv4Addr::new(res[0], res[1], res[2], res[3])))
}

fn parse_ignored(s: &mut &str) -> PResult<()> {
    "- ".parse_next(s)?;
    Ok(())
}

fn parse_datetime(s: &mut &str) -> PResult<DateTime<Utc>> {
    let ret = delimited('[', take_until(1.., ']'), ']').parse_next(s)?;
    space0(s)?;
    Ok(DateTime::parse_from_str(ret, "%d/%b/%Y:%H:%M:%S %z")
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap())
}

fn parse_http(s: &mut &str) -> PResult<(HttpMethod, String, HttpProto)> {
    let parser = (parse_http_method, parse_http_url, parse_http_proto);
    let ret = delimited('"', parser, '"').parse_next(s)?;
    space0(s)?;
    Ok(ret)
}

fn parse_http_method(s: &mut &str) -> PResult<HttpMethod> {
    let ret = alt((
        "GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS", "CONNECT", "TRACE", "PATCH",
    ))
    .parse_to()
    .parse_next(s)?;
    space0(s)?;
    Ok(ret)
}

fn parse_http_url(s: &mut &str) -> PResult<String> {
    let ret = take_until(1.., ' ').parse_next(s)?;
    space0(s)?;
    Ok(ret.to_string())
}

fn parse_http_proto(s: &mut &str) -> PResult<HttpProto> {
    let ret = alt(("HTTP/1.0", "HTTP/1.1", "HTTP/2.0", "HTTP/3.0"))
        .parse_to()
        .parse_next(s)?;
    space0(s)?;
    Ok(ret)
}

fn parse_http_status(s: &mut &str) -> PResult<u16> {
    let ret = digit1.parse_to().parse_next(s)?;
    space0(s)?;
    Ok(ret)
}

fn parse_http_body_bytes(s: &mut &str) -> PResult<u64> {
    let ret = digit1.parse_to().parse_next(s)?;
    space0(s)?;
    Ok(ret)
}

fn parse_quoted_string(s: &mut &str) -> PResult<String> {
    let ret = delimited('"', take_until(1.., '"'), '"').parse_next(s)?;
    space0(s)?;
    Ok(ret.to_string())
}

impl FromStr for HttpProto {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HTTP/1.0" => Ok(HttpProto::HTTP1_0),
            "HTTP/1.1" => Ok(HttpProto::HTTP1_1),
            "HTTP/2.0" => Ok(HttpProto::HTTP2_0),
            "HTTP/3.0" => Ok(HttpProto::HTTP3_0),
            _ => Err(anyhow::anyhow!("Unknown HTTP protocol: {}", s)),
        }
    }
}

impl FromStr for HttpMethod {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "DELETE" => Ok(HttpMethod::Delete),
            "HEAD" => Ok(HttpMethod::Head),
            "OPTIONS" => Ok(HttpMethod::Options),
            "CONNECT" => Ok(HttpMethod::Connect),
            "TRACE" => Ok(HttpMethod::Trace),
            "PATCH" => Ok(HttpMethod::Patch),
            _ => Err(anyhow::anyhow!("Unknown HTTP method: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn parse_ip_should_work() -> Result<()> {
        let mut s = "1.1.1.1";
        let ip = parse_ip(&mut s).unwrap();
        assert_eq!(s, "");
        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)));
        Ok(())
    }

    #[test]
    fn parse_datetime_should_work() -> Result<()> {
        let mut s = "[17/May/2015:08:05:32 +0000]";
        let dt = parse_datetime(&mut s).unwrap();
        assert_eq!(s, "");
        assert_eq!(dt, Utc.with_ymd_and_hms(2015, 5, 17, 8, 5, 32).unwrap());
        Ok(())
    }

    #[test]
    fn parse_http_should_work() -> Result<()> {
        let mut s = "\"GET /downloads/product_1 HTTP/1.1\"";
        let (method, url, protocol) = parse_http(&mut s).unwrap();
        assert_eq!(s, "");
        assert_eq!(method, HttpMethod::Get);
        assert_eq!(url, "/downloads/product_1");
        assert_eq!(protocol, HttpProto::HTTP1_1);
        Ok(())
    }
}
