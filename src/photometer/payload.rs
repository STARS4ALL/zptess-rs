pub mod json {

    // JSON parsing stuff
    use serde::Deserialize;
    use serde_json;

    #[derive(Deserialize, Debug)]
    #[allow(non_snake_case)]
    pub struct TESSPayload {
        udp: u32,
        rev: i8,
        name: String,
        freq: f32,
        mag: f32,
        tamb: f32,
        tsky: f32,
        wdBm: i16,
        ain: i16,
        ZP: f32,
    }

    impl TESSPayload {
        pub fn new(line: &str) -> Self {
            serde_json::from_str(line).expect("Decoding JSON")
        }
    }
}
