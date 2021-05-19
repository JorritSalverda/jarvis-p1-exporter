use crate::model::Config;
use serde_yaml;
use std::env;
use std::error::Error;
use std::fs;

pub struct ConfigClientConfig {
    config_path: String,
}

impl ConfigClientConfig {
    pub fn new(config_path: String) -> Result<Self, Box<dyn Error>> {
        println!("ConfigClientConfig::new(config_path: {})", config_path);
        Ok(Self { config_path })
    }

    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        let config_path = env::var("CONFIG_PATH").unwrap_or("/configs/config.yaml".to_string());

        Self::new(config_path)
    }
}

pub struct ConfigClient {
    config: ConfigClientConfig,
}

impl ConfigClient {
    pub fn new(config: ConfigClientConfig) -> Self {
        Self { config }
    }

    pub fn read_config_from_file(&self) -> Result<Config, Box<dyn Error>> {

        println!("Loading config from {}...", &self.config.config_path);

        let config_file_contents = fs::read_to_string(&self.config.config_path)?;
        let config: Config = serde_yaml::from_str(&config_file_contents)?;

        println!("Loaded config from {}: {:?}", &self.config.config_path, &config);

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jarvis_lib::{EntityType,SampleType,MetricType};

    #[test]
    fn read_config_from_file_returns_deserialized_test_file() {
        let config_client =
            ConfigClient::new(ConfigClientConfig::new("test-config.yaml".to_string()).unwrap());

        let config = config_client.read_config_from_file().unwrap();

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
