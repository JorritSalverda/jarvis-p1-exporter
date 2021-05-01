use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum EntityType {
    #[serde(rename = "")]
    Invalid,
    #[serde(rename = "ENTITY_TYPE_TARIFF")]
    Tariff,
    #[serde(rename = "ENTITY_TYPE_ZONE")]
    Zone,
    #[serde(rename = "ENTITY_TYPE_DEVICE")]
    Device,
}

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum MetricType {
    #[serde(rename = "")]
    Invalid,
    #[serde(rename = "METRIC_TYPE_COUNTER")]
    Counter,
    #[serde(rename = "METRIC_TYPE_GAUGE")]
    Gauge,
}

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum SampleType {
    #[serde(rename = "")]
    Invalid,
    #[serde(rename = "SAMPLE_TYPE_ELECTRICITY_CONSUMPTION")]
    ElectricityConsumption,
    #[serde(rename = "SAMPLE_TYPE_ELECTRICITY_PRODUCTION")]
    ElectricityProduction,
    #[serde(rename = "SAMPLE_TYPE_GAS_CONSUMPTION")]
    GasConsumption,
    #[serde(rename = "SAMPLE_TYPE_TEMPERATURE")]
    Temperature,
    #[serde(rename = "SAMPLE_TYPE_PRESSURE")]
    Pressure,
    #[serde(rename = "SAMPLE_TYPE_FLOW")]
    Flow,
    #[serde(rename = "SAMPLE_TYPE_HUMIDITY")]
    Humidity,
    Time,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Measurement {
    pub id: String,
    pub source: String,
    pub location: String,
    pub samples: Vec<Sample>,
    pub measured_at_time: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Sample {
    pub entity_type: EntityType,
    pub entity_name: String,
    pub sample_type: SampleType,
    pub sample_name: String,
    pub metric_type: MetricType,
    pub value: f64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub location: String,
    pub sample_configs: Vec<ConfigSample>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConfigSample {
    // default jarvis config for sample
    pub entity_type: EntityType,
    pub entity_name: String,
    pub sample_type: SampleType,
    pub sample_name: String,
    pub metric_type: MetricType,

    // modbus specific config for sample
    pub value_multiplier: f64,
    pub register_type: RegisterType,
    pub register_address: u16,
    pub register_quantity: u16,
}

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum RegisterType {
    #[serde(rename = "input")]
    Input,
    #[serde(rename = "holding")]
    Holding,
}
