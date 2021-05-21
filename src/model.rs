use jarvis_lib::config_client::SetDefaults;
use jarvis_lib::model::{EntityType, MetricType, SampleType};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub location: String,
    pub sample_configs: Vec<ConfigSample>,
}

impl SetDefaults for Config {
    fn set_defaults(&mut self) {}
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

#[cfg(test)]
mod tests {
    use super::*;
    use jarvis_lib::config_client::{ConfigClient, ConfigClientConfig};
    use jarvis_lib::model::{EntityType, MetricType, SampleType};

    #[test]
    fn read_config_from_file_returns_deserialized_test_file() {
        let config_client =
            ConfigClient::new(ConfigClientConfig::new("test-config.yaml".to_string()).unwrap());

        let config: Config = config_client.read_config_from_file().unwrap();

        assert_eq!(config.location, "My Home".to_string());
        assert_eq!(config.sample_configs.len(), 4);

        assert_eq!(config.sample_configs[0].entity_type, EntityType::Tariff);
        assert_eq!(
            config.sample_configs[0].entity_name,
            "Sagemcom XS210".to_string()
        );
        assert_eq!(
            config.sample_configs[0].sample_type,
            SampleType::ElectricityConsumption
        );
        assert_eq!(
            config.sample_configs[0].sample_name,
            "Levering dal".to_string()
        );
        assert_eq!(config.sample_configs[0].metric_type, MetricType::Counter);
        assert_eq!(config.sample_configs[0].value_multiplier, 3600000f64);
        assert_eq!(config.sample_configs[0].prefix, "1-0:1.8.1".to_string());
        assert_eq!(config.sample_configs[0].value_start_index, 10u16);
        assert_eq!(config.sample_configs[0].value_length, 10u16);

        assert_eq!(config.sample_configs[1].entity_type, EntityType::Tariff);
        assert_eq!(
            config.sample_configs[1].entity_name,
            "Sagemcom XS210".to_string()
        );
        assert_eq!(
            config.sample_configs[1].sample_type,
            SampleType::ElectricityConsumption
        );
        assert_eq!(
            config.sample_configs[1].sample_name,
            "Levering normaal".to_string()
        );
        assert_eq!(config.sample_configs[1].metric_type, MetricType::Counter);
        assert_eq!(config.sample_configs[1].value_multiplier, 3600000f64);
        assert_eq!(config.sample_configs[1].prefix, "1-0:1.8.2".to_string());
        assert_eq!(config.sample_configs[1].value_start_index, 10u16);
        assert_eq!(config.sample_configs[1].value_length, 10u16);

        assert_eq!(config.sample_configs[2].entity_type, EntityType::Tariff);
        assert_eq!(
            config.sample_configs[2].entity_name,
            "Sagemcom XS210".to_string()
        );
        assert_eq!(
            config.sample_configs[2].sample_type,
            SampleType::ElectricityProduction
        );
        assert_eq!(
            config.sample_configs[2].sample_name,
            "Teruglevering dal".to_string()
        );
        assert_eq!(config.sample_configs[2].metric_type, MetricType::Counter);
        assert_eq!(config.sample_configs[2].value_multiplier, 3600000f64);
        assert_eq!(config.sample_configs[2].prefix, "1-0:2.8.1".to_string());
        assert_eq!(config.sample_configs[2].value_start_index, 10u16);
        assert_eq!(config.sample_configs[2].value_length, 10u16);

        assert_eq!(config.sample_configs[3].entity_type, EntityType::Tariff);
        assert_eq!(
            config.sample_configs[3].entity_name,
            "Sagemcom XS210".to_string()
        );
        assert_eq!(
            config.sample_configs[3].sample_type,
            SampleType::ElectricityProduction
        );
        assert_eq!(
            config.sample_configs[3].sample_name,
            "Teruglevering normaal".to_string()
        );
        assert_eq!(config.sample_configs[3].metric_type, MetricType::Counter);
        assert_eq!(config.sample_configs[3].value_multiplier, 3600000f64);
        assert_eq!(config.sample_configs[3].prefix, "1-0:2.8.2".to_string());
        assert_eq!(config.sample_configs[3].value_start_index, 10u16);
        assert_eq!(config.sample_configs[3].value_length, 10u16);
    }
}
