use regex::Regex;
use std::time::Duration;
use tracing::{error, info};

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

    pub async fn update_zp(&self, zp: f32) {
        let param1 = vec![("nZP1", format!("{zp:.02}"))];
        let param2 = vec![("cons", format!("{zp:.02}"))];
        let client = reqwest::Client::builder()
            .timeout(Duration::new(3, 0))
            .build()
            .expect("Building writting request");
        // Try both with the old URL and the new
        client
            .get(URL_SET_ZP_V1)
            .query(&param1)
            .send()
            .await
            .unwrap();
        client
            .get(URL_SET_ZP_V2)
            .query(&param2)
            .send()
            .await
            .unwrap();

        self.verify(zp).await;
    }

    async fn verify(&self, zp: f32) {
        let client = reqwest::Client::builder()
            .timeout(Duration::new(3, 0))
            .build();
        let body = client
            .expect("Building veryfy request")
            .get(URL_GET_ZP)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        let phot_zp = if let Some(result) = self.re.captures(&body) {
            result[2].trim().parse::<f32>().expect("parsing ZP")
        } else {
            0.0
        };
        if phot_zp != zp {
            error!(
                "Written ZP {:.02} does not match photometer's current ZP {:.02}",
                zp, phot_zp
            );
        } else {
            info!("Successfully written ZP {:.02} to photometer", zp);
        }
    }
}
