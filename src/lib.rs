pub mod airthings_radon;
pub mod audit;
pub mod config;
pub mod notify;
pub mod pretty;
pub mod report;
pub mod weather;

pub struct IndoorSettings {
    pub max_humidity: u64,
    pub min_temp: f64,
    pub max_temp: f64,
}
#[derive(Clone, Debug)]
pub struct HumidityTemp {
    pub humidity: u64,
    pub temp: f64,
}
