use super::Info;
use anyhow::Result;
use regex::Regex;
use reqwest;
use std::time::Duration;
use tracing::info;

const NAME: &str = r"(stars\d+)";
const MAC: &str = r"MAC: ([0-9A-Fa-f]{1,2}:[0-9A-Fa-f]{1,2}:[0-9A-Fa-f]{1,2}:[0-9A-Fa-f]{1,2}:[0-9A-Fa-f]{1,2}:[0-9A-Fa-f]{1,2})";
const ZP: &str = r"(ZP|CI.*): (\d{1,2}\.\d{1,2})";
const FIRMWARE: &str = r"Compiled: (.+?)<br>";
const FREQ_OFF: &str = r"Offset Hz: (\d{1,3}\.\d{1,3})<br>";
const URL_GET_INFO: &str = "http://192.168.4.1/config";

/*
 let mut owned_string: String = "hello ".to_owned();
    let another_owned_string: String = "world".to_owned();

    owned_string.push_str(&another_owned_string);
    println!("{}", owned_string);
*/

#[derive(Debug)]
pub struct Discoverer {
    re: Vec<Regex>,
}

impl Discoverer {
    pub fn new() -> Self {
        Self {
            re: vec![
                Regex::new(NAME).unwrap(),
                Regex::new(MAC).unwrap(),
                Regex::new(FIRMWARE).unwrap(),
                Regex::new(ZP).unwrap(),
                Regex::new(FREQ_OFF).unwrap(),
            ],
        }
    }

    fn decode(&self, body: &str) -> Result<Info> {
        let mut info = Info::new();
        for (i, re) in self.re.iter().enumerate() {
            if let Some(result) = re.captures(body) {
                match i {
                    0 => info.name = result[1].to_string(),
                    1 => info.mac = result[1].to_string(),
                    2 => info.firmware = result[1].to_string(),
                    3 => info.zp = result[2].trim().parse::<f32>()?,
                    4 => info.freq_offset = result[1].trim().parse::<f32>()?,
                    _ => unimplemented!(),
                }
            }
        }
        info!("From http: {:#?}", info);
        Ok(info)
    }

    async fn fetch(&self) -> Result<String> {
        let client = reqwest::Client::builder()
            .timeout(Duration::new(3, 0))
            .build()?;
        let body = client.get(URL_GET_INFO).send().await?.text().await?;
        Ok(body)
    }

    pub async fn discover(&self) -> Result<Info> {
        let body = self.fetch().await?;
        self.decode(&body)
    }
}
