use serde::{Deserialize, Serialize};
use jarvis_lib::{EntityType,SampleType,MetricType};

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
    pub prefix: String,
    pub value_start_index: u16,
    pub value_length: u16,
}
