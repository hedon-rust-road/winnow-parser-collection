#![allow(unused)]
use std::net::IpAddr;

use anyhow::anyhow;
use chrono::{format::Pad, DateTime, Utc};
use winnow::PResult;

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
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
fn main() -> anyhow::Result<()> {
    let s = r#"93.180.71.3 - - [17/May/2015:08:05:32 +0000] "GET /downloads/product_1 HTTP/1.1" 304 0 "-" "Debian APT-HTTP/1.3 (0.8.16~exp12ubuntu10.21)""#;
    let log = parse_nginx_log(s).map_err(|e| anyhow!("Failed to parse log: {:?}", e))?;

    println!("{:?}", log);
    Ok(())
}

fn parse_nginx_log(s: &str) -> PResult<NginxLog> {
    let input = &mut (&*s);
    let ip = parse_ip(input)?;
    parse_ignored(input)?;
    parse_ignored(input)?;
    let datetime = parse_datetime(input)?;
    println!("datetime: {:?}", datetime);
    let (method, url, protocol) = parse_http(input)?;
    let status = parse_http_status(input)?;
    println!("status: {:?}", status);
    let body_bytes = parse_http_body_bytes(input)?;
    let referer = parse_http_referer(input)?;
    let user_agent = parse_http_user_agent(input)?;
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

fn parse_ip(_s: &mut &str) -> PResult<IpAddr> {
    todo!()
}

fn parse_ignored(_s: &mut &str) -> PResult<()> {
    todo!()
}

fn parse_datetime(_s: &mut &str) -> PResult<DateTime<Utc>> {
    todo!()
}

fn parse_http(_s: &mut &str) -> PResult<(HttpMethod, String, HttpProto)> {
    todo!()
}

fn parse_http_method(_s: &mut &str) -> PResult<HttpMethod> {
    todo!()
}

fn parse_http_url(_s: &mut &str) -> PResult<String> {
    todo!()
}

fn parse_http_proto(_s: &mut &str) -> PResult<HttpProto> {
    todo!()
}

fn parse_http_status(_s: &mut &str) -> PResult<u16> {
    todo!()
}

fn parse_http_body_bytes(_s: &mut &str) -> PResult<u64> {
    todo!()
}

fn parse_http_referer(_s: &mut &str) -> PResult<String> {
    todo!()
}

fn parse_http_user_agent(_s: &mut &str) -> PResult<String> {
    todo!()
}
