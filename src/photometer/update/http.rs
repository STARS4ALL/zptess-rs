use anyhow;
use regex::Regex;
use std::time::Duration;

const URL_SET_ZP_V1: &'static str = "http://192.168.4.1/SetZP";
const URL_SET_ZP_V2: &'static str = "http://192.168.4.1/setconst";
const URL_GET_ZP: &'static str = "http://192.168.4.1/config";
const ZP: &str = r"(ZP|CI.*): (\d{1,2}\.\d{1,2})";

pub struct Updater {
    re: Regex,
}

impl Updater {
    pub fn new() -> Self {
        Self {
            re: Regex::new(ZP).unwrap(),
        }
    }

    pub async fn update_zp(&self, zp: f32) -> Result<(), anyhow::Error> {
        let param1 = vec![("nZP1", format!("{zp:.02}"))];
        let param2 = vec![("cons", format!("{zp:.02}"))];
        let client = reqwest::Client::builder()
            .timeout(Duration::new(3, 0))
            .build()?;
        // Try both with the old URL and the new
        client.get(URL_SET_ZP_V1).query(&param1).send().await?;
        client.get(URL_SET_ZP_V2).query(&param2).send().await?;
        self.verify(zp).await?;
        Ok(())
    }

    async fn verify(&self, written_zp: f32) -> Result<(), anyhow::Error> {
        let client = reqwest::Client::builder()
            .timeout(Duration::new(3, 0))
            .build()?;
        let response = client.get(URL_GET_ZP).send().await?;
        let body = response.text().await?;
        let read_zp = if let Some(result) = self.re.captures(&body) {
            result[2].trim().parse::<f32>()?
        } else {
            anyhow::bail!("Parsing TESS-W HTML page");
        };
        if read_zp != written_zp {
            anyhow::bail!(
                "Read ZP ({:.02}) doesn't match written ZP ({:02})",
                read_zp,
                written_zp
            );
        }
        Ok(())
    }
}
