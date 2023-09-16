extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Indoor {
    battery: i32,
    pub humidity: f64,
    radonShortTermAvg: f64,
    pub temp: f64,
    time: i64,
    relayDeviceType: String,
}
pub fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    celsius * 9.0 / 5.0 + 32.0
}

#[test]
fn test_celsius_to_fahrenheit() {
    let input = 0.0; // freezing point of water in Celsius
    let expected_output = 32.0; // freezing point of water in Fahrenheit
    let result = celsius_to_fahrenheit(input);
    assert_eq!(expected_output, result);
}

#[test]
fn test() {
    let data = r#"
    {
        "battery": 99,
        "humidity": 61.0,
        "radonShortTermAvg": 35.0,
        "temp": 21.5,
        "time": 1694880270,
        "relayDeviceType": "app"
    }
    "#;

    let indoor: Indoor = serde_json::from_str(data).unwrap();
    println!("{:?}", indoor);

    let serialized_data = serde_json::to_string(&indoor).unwrap();
    println!("{}", serialized_data);
}
