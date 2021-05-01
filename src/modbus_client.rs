use crate::model::{Config, ConfigSample, Measurement, MetricType, RegisterType, Sample};
use byteorder::{BigEndian, ByteOrder};
use chrono::Utc;
use conv::*;
use modbus::tcp;
use modbus::Client;
use std::env;
use std::error::Error;
use std::time::Duration;
use uuid::Uuid;

pub struct ModbusClientConfig {
    host: String,
    port: u16,
    unit_id: u8,
}

impl ModbusClientConfig {
    pub fn new(host: String, port: u16, unit_id: u8) -> Result<Self, Box<dyn Error>> {
        println!(
            "ModbusClientConfig::new(host: {}, port: {}, unit_id: {})",
            host, port, unit_id
        );

        if host == "" {
            return Err(Box::<dyn Error>::from(
                "Please set the ip address of your modbus device on your local network",
            ));
        }
        if port != 502 && (port < 49152) {
            return Err(Box::<dyn Error>::from("Please set the modbus port of your modbus device on your local network to its default 502, or anywhere between 49152 and 65535 if changed in the installer menu"));
        }

        Ok(Self {
            host,
            port,
            unit_id,
        })
    }

    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        let host = env::var("MODBUS_HOST_IP").unwrap_or("127.0.0.1".to_string());
        let port: u16 = env::var("MODBUS_HOST_PORT")
            .unwrap_or("502".to_string())
            .parse()
            .unwrap_or(502);
        let unit_id: u8 = env::var("MODBUS_UNIT_ID")
            .unwrap_or("3".to_string())
            .parse()
            .unwrap_or(3);

        Self::new(host, port, unit_id)
    }
}

pub struct ModbusClient {
    config: ModbusClientConfig,
}

impl ModbusClient {
    pub fn new(config: ModbusClientConfig) -> ModbusClient {
        ModbusClient { config }
    }

    pub fn get_measurement(
        &self,
        config: Config,
        last_measurement: Option<Measurement>,
    ) -> Result<Measurement, Box<dyn Error>> {
        let mut modbus_client = self.init_modbus_client()?;

        let mut measurement = Measurement {
            id: Uuid::new_v4().to_string(),
            source: String::from("jarvis-modbus-exporter"),
            location: config.location.clone(),
            samples: Vec::new(),
            measured_at_time: Utc::now(),
        };

        for sample_config in config.sample_configs.iter() {
            match self.get_sample(sample_config, &mut modbus_client) {
                Ok(sample) => {
                    measurement.samples.push(sample);
                }
                Err(error) => return Err(error),
            };
        }

        match last_measurement {
            Some(lm) => {
                measurement.samples = self.sanitize_samples(measurement.samples, lm.samples)
            }
            None => {}
        }

        println!(
            "Retrieved measurement via modbus from device {}",
            &self.config.host
        );

        Ok(measurement)
    }

    fn init_modbus_cfg(&self) -> modbus::Config {
        let mut cfg = tcp::Config::default();
        cfg.tcp_port = self.config.port;
        cfg.modbus_uid = self.config.unit_id;
        cfg.tcp_connect_timeout = Some(Duration::new(20, 0));

        cfg
    }

    fn init_modbus_client(&self) -> std::io::Result<modbus::Transport> {
        let cfg = self.init_modbus_cfg();

        tcp::Transport::new_with_cfg(&self.config.host, cfg)
    }

    fn get_sample(
        &self,
        sample_config: &ConfigSample,
        modbus_client: &mut modbus::Transport,
    ) -> Result<Sample, Box<dyn Error>> {
        let sample_registers = match sample_config.register_type {
            RegisterType::Input => modbus_client.read_input_registers(
                sample_config.register_address,
                sample_config.register_quantity,
            ),
            RegisterType::Holding => modbus_client.read_holding_registers(
                sample_config.register_address,
                sample_config.register_quantity,
            ),
        }?;

        let mut sample_bytes: Vec<u8> = vec![0; sample_registers.len() * 2];
        BigEndian::write_u16_into(&sample_registers, &mut sample_bytes);

        let mut sample_value: Vec<u64> = vec![0; sample_registers.len() / 4];
        BigEndian::read_u64_into(&sample_bytes, &mut sample_value);

        // init sample from config
        Ok(Sample {
            entity_type: sample_config.entity_type,
            entity_name: sample_config.entity_name.clone(),
            sample_type: sample_config.sample_type,
            sample_name: sample_config.sample_name.clone(),
            metric_type: sample_config.metric_type,
            value: f64::approx_from(sample_value[0]).unwrap() * sample_config.value_multiplier,
        })
    }

    fn sanitize_samples(
        &self,
        current_samples: Vec<Sample>,
        last_samples: Vec<Sample>,
    ) -> Vec<Sample> {
        let mut sanitized_samples: Vec<Sample> = Vec::new();

        for current_sample in current_samples.into_iter() {
            // check if there's a corresponding sample in lastSamples and see if the difference with it's value isn't too large
            let mut sanitize = false;
            for last_sample in last_samples.iter() {
                if current_sample.entity_type == last_sample.entity_type
                    && current_sample.entity_name == last_sample.entity_name
                    && current_sample.sample_type == last_sample.sample_type
                    && current_sample.sample_name == last_sample.sample_name
                    && current_sample.metric_type == last_sample.metric_type
                {
                    if current_sample.metric_type == MetricType::Counter
                        && current_sample.value < last_sample.value
                    {
                        sanitize = true;
                        // log.Warn().Msgf("Value for %v is less than the last sampled value %v, keeping previous value instead", cs, ls.Value)
                        sanitized_samples.push(last_sample.clone());
                    } else if current_sample.metric_type == MetricType::Counter
                        && current_sample.value / last_sample.value > 1.1
                    {
                        sanitize = true;
                        // log.Warn().Msgf("Value for %v is more than 10 percent larger than the last sampled value %v, keeping previous value instead", cs, ls.Value)
                        sanitized_samples.push(last_sample.clone());
                    }

                    break;
                }
            }

            if !sanitize {
                sanitized_samples.push(current_sample);
            }
        }

        sanitized_samples
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        Config, ConfigSample, EntityType, Measurement, MetricType, RegisterType, SampleType,
    };

    #[test]
    #[ignore]
    fn get_measurement_returns_total_power_produced_for_sma_convertor() {
        let modbus_client = ModbusClient::new(
            ModbusClientConfig::new("192.168.195.3".to_string(), 502, 3).unwrap(),
        );

        let config = Config {
            location: "My Home".to_string(),
            sample_configs: vec![ConfigSample {
                entity_type: EntityType::Device,
                entity_name: "Sunny TriPower 8.0".to_string(),
                sample_type: SampleType::ElectricityProduction,
                sample_name: "Totaal opgewekt".to_string(),
                metric_type: MetricType::Counter,
                value_multiplier: 3600f64,
                register_type: RegisterType::Input,
                register_address: 30513u16,
                register_quantity: 4u16,
            }],
        };

        let last_measurement: Option<Measurement> = Option::None;

        let measurement = modbus_client
            .get_measurement(config, last_measurement)
            .unwrap();

        assert_eq!(measurement.location, "My Home".to_string());
        assert_eq!(measurement.samples.len(), 1);
        assert_eq!(measurement.samples[0].entity_type, EntityType::Device);
        assert_eq!(
            measurement.samples[0].entity_name,
            "Sunny TriPower 8.0".to_string()
        );
        assert_eq!(
            measurement.samples[0].sample_type,
            SampleType::ElectricityProduction
        );
        assert_eq!(
            measurement.samples[0].sample_name,
            "Totaal opgewekt".to_string()
        );
        assert_eq!(measurement.samples[0].metric_type, MetricType::Counter);
        assert!(measurement.samples[0].value > 9253360800.0f64);
    }
}
