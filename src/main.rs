/**
Query the host IP address by DoH(DNS over HTTPs)

DoH Server:
- https://dns.alidns.com/dns-query
- https://1.1.1.1/dns-query
- https://9.9.9.9/dns-query
- https://rubyfish.cn/dns-query

参考:
https://help.aliyun.com/document_detail/171666.html
 */
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use std::time::Duration;

use clap::Parser;
use futures::future;
use rand::random;
use serde::{Deserialize, Serialize};
use surge_ping::{Client, Config, PingIdentifier, PingSequence};
use tabled::object::{Columns, Rows};
use tabled::{Alignment, Modify, Style, Table, Tabled};
use tokio::time;

#[derive(Debug, Clone, Parser)]
#[command(name = "doh")]
#[command(about = "Query the host IP address by DoH(DNS over HTTPs)", long_about = None)]
struct Cli {
    /// Query hostname
    #[arg(long)]
    host: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Answer {
    /// The name of the record.
    pub name: String,
    /// The type associated with each record. To convert to a string representation use
    /// https://www.iana.org/assignments/dns-parameters/dns-parameters.xhtml#dns-parameters-4
    pub r#type: u32,
    /// The time to live in seconds for this record.
    pub TTL: u32,
    /// The data associated with the record.
    pub data: String,
}

pub struct Row {
    pub answer: Answer,
    pub mean: i32,
    pub lost: f32,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
struct DnsResponse {
    Status: u32,
    Answer: Option<Vec<Answer>>,
    Comment: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Tabled)]
pub struct Record {
    pub DoH: String,
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Type")]
    pub r#type: u32,
    pub TTL: u32,
    #[tabled(rename = "Address")]
    pub data: String,
    #[tabled(rename = "Avg")]
    pub mean: String,
    #[tabled(rename = "Lost")]
    pub lost: String,
}

impl Record {
    pub fn new(tag: &str, r: Row) -> Self {
        Self {
            DoH: String::from(tag),
            name: r.answer.name,
            r#type: r.answer.r#type,
            TTL: r.answer.TTL,
            data: r.answer.data,
            mean: if r.mean == -1 {
                String::from("/")
            } else {
                format!("{}ms", r.mean)
            },
            lost: format!("{}%", (r.lost * 100.0)),
        }
    }
}

async fn get_ip(hostname: &str, server: &str) -> Result<Vec<Row>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = format!("{}?name={}&type={}", server, hostname, "A");
    let res = client
        .get(url)
        .header("Accept", "application/dns-json")
        .send()
        .await?;

    let mut data: Vec<Answer> = vec![];
    let body = res.json::<DnsResponse>().await?;
    if let Some(answer) = body.Answer {
        data.extend(answer);
    }

    let mut items = vec![];
    let client = Client::new(&Config::default())?;
    for item in data.clone() {
        let ip_v4 = Ipv4Addr::from_str(item.data.as_str())?;
        let (mean, lost) = ping(client.clone(), IpAddr::V4(ip_v4)).await;
        items.push(Row {
            answer: item,
            mean,
            lost,
        });
    }
    Ok(items)
}

const MAX_PING: u16 = 10;

fn mean(data: &[i32]) -> Option<f32> {
    let sum = data.iter().sum::<i32>() as f32;
    let count = data.len();

    match count {
        positive if positive > 0 => Some(sum / count as f32),
        _ => None,
    }
}

async fn ping(client: Client, addr: IpAddr) -> (i32, f32) {
    let payload = [0; 56];
    let mut pinger = client.pinger(addr, PingIdentifier(random())).await;
    pinger.timeout(Duration::from_secs(1));
    let mut interval = time::interval(Duration::from_secs(1));
    let mut times = vec![];
    let mut lost = 0.0;
    for idx in 0..MAX_PING {
        interval.tick().await;
        let res = pinger.ping(PingSequence(idx), &payload).await;
        if let Ok((_, dur)) = res {
            times.push(dur.as_millis() as i32);
        } else {
            lost += 1.0;
        }
    }
    (
        mean(&times[..]).map_or(-1, |i| i as i32),
        lost / MAX_PING as f32,
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let servers = HashMap::from([
        ("1.1.1.1", "https://1.1.1.1/dns-query"),
        ("9.9.9.9", "https://9.9.9.9:5053/dns-query"),
        ("aliyun", "https://dns.alidns.com/resolve"),
    ]);

    let args = Cli::parse();
    let hostname = args.host.as_str();
    // get_ip(&hostname, "server");
    let bodies = future::join_all(
        servers
            .into_iter()
            .map(|(tag, server)| async move { (tag, get_ip(hostname, server).await) }),
    )
    .await;

    let mut data = vec![];
    for (tag, items) in bodies {
        match items {
            Ok(items) => {
                for item in items {
                    data.push(Record::new(tag, item));
                }
            }
            Err(e) => eprintln!("{} error: {}", tag, e),
        }
    }

    let table = Table::new(data)
        .with(Style::modern())
        .with(Modify::new(Columns::single(6)).with(Alignment::right()))
        .with(Modify::new(Columns::single(5)).with(Alignment::right()))
        .with(Modify::new(Rows::first()).with(Alignment::center()))
        .to_string();
    println!("{}", table);
    Ok(())
}
