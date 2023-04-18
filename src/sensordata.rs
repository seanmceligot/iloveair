extern crate serde_json;
use serde::{Deserialize, Serialize};

// example SensorData for serde_json
//
// {
//   "date": {
//     "val": "2023-04-17 20:28",
//     "unit": "%Y-%m-%d %H:%M"
//   },
//   "humidity": {
//     "val": 47.0,
//     "unit": "%rH"
//   },
//   "radon_st_avg": {
//     "val": 68,
//     "unit": "Bq/m3"
//   },
//   "radon_lt_avg": {
//     "val": 81,
//     "unit": "Bq/m3"
//   },
//   "temperature": {
//     "val": 70.016,
//     "unit": "degF"
//   },
//   "pressure": {
//     "val": 994.7,
//     "unit": "hPa"
//   },
//   "co2": {
//     "val": 521.0,
//     "unit": "ppm"
//   },
//   "voc": {
//     "val": 80.0,
//     "unit": "ppb"
//   }
// }
#[derive(Serialize, Deserialize)]
pub struct SensorData {
    pub date: DateData,
    pub humidity: HumidityData,
    pub radon_st_avg: RadonData,
    pub radon_lt_avg: RadonData,
    pub temperature: TemperatureData,
    pub pressure: PressureData,
    pub co2: CO2Data,
    pub voc: VOCData,
}

#[derive(Serialize, Deserialize)]
pub struct DateData {
    pub val: String,
    pub unit: String,
}

#[derive(Serialize, Deserialize)]
pub struct HumidityData {
    pub val: f32,
    pub unit: String,
}

#[derive(Serialize, Deserialize)]
pub struct RadonData {
    pub val: i32,
    pub unit: String,
}

#[derive(Serialize, Deserialize)]
pub struct TemperatureData {
    pub val: f32,
    pub unit: String,
}

#[derive(Serialize, Deserialize)]
pub struct PressureData {
    pub val: f32,
    pub unit: String,
}

#[derive(Serialize, Deserialize)]
pub struct CO2Data {
    pub val: f32,
    pub unit: String,
}

#[derive(Serialize, Deserialize)]
pub struct VOCData {
    pub val: f32,
    pub unit: String,
}
pub fn pretty_print_sensor_data(data: &SensorData) {
    println!("Date: {} {}", data.date.val, data.date.unit);
    println!("Humidity: {} {}", data.humidity.val, data.humidity.unit);
    println!(
        "Radon Short-Term Average: {} {}",
        data.radon_st_avg.val, data.radon_st_avg.unit
    );
    println!(
        "Radon Long-Term Average: {} {}",
        data.radon_lt_avg.val, data.radon_lt_avg.unit
    );
    println!(
        "Temperature: {} {}",
        data.temperature.val, data.temperature.unit
    );
    println!("Pressure: {} {}", data.pressure.val, data.pressure.unit);
    println!("CO2: {} {}", data.co2.val, data.co2.unit);
    println!("VOC: {} {}", data.voc.val, data.voc.unit);
}
