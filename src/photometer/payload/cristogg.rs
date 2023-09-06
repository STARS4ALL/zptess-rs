// Cristobal Garcia's old way to deliver readings

use regex::Regex;

// <fH 00430><tA +2945><tO +2439><mZ -0000>
const PATTERN1: &str = r"^<fH([ +]\d{5})><tA ([+-]\d{4})><tO ([+-]\d{4})><mZ ([+-]\d{4})>";
const PATTERN2: &str = r"^<fm([ +]\d{5})><tA ([+-]\d{4})><tO ([+-]\d{4})><mZ ([+-]\d{4})>";

//let re2: Regex = Regex::new(PATTERN1).expect("Failed pattern");
//let re1: Regex = Regex::new(PATTERN1).expect("Failed pattern");

#[derive(Debug)]
struct TESSPayload {
    freq: f32,
    tbox: f32,
    tsky: f32,
    zp: f32,
}
impl TESSPayload {
    pub fn new(re: &Regex, line: &str) {
        let re: Regex = Regex::new(PATTERN1).expect("Failed pattern");
        if let Some(result) = re.captures(line) {
            tracing::info!("{:?}", result);
            let p = TESSPayload {
                freq: result[1].trim().parse::<f32>().expect("Frequency") / 1000.0,
                zp: result[4].trim().parse::<f32>().expect("ZP"),
                tbox: result[2].trim().parse::<f32>().expect("Temp Box") / 100.0,
                tsky: result[3].trim().parse::<f32>().expect("Temp Sky") / 100.0,
            };
            tracing::info!("{:?}", p);
        }
    }
}
