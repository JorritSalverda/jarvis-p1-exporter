use crate::model::Config;
use jarvis_lib::{Measurement, MetricType, Sample};
use chrono::Utc;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::time::Duration;
use uuid::Uuid;

pub struct P1ClientConfig {
    usb_device_path: String,
}

impl P1ClientConfig {
    pub fn new(usb_device_path: String) -> Result<Self, Box<dyn Error>> {
        println!("P1ClientConfig::new(usb_device_path: {})", usb_device_path);
        Ok(Self { usb_device_path })
    }

    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        let usb_device_path = env::var("P1_USB_DEVICE_PATH").unwrap_or("/dev/ttyUSB0".to_string());

        Self::new(usb_device_path)
    }
}

pub struct P1Client {
    config: P1ClientConfig,
}

impl P1Client {
    pub fn new(config: P1ClientConfig) -> Self {
        Self { config }
    }

    pub fn get_measurement(
        &self,
        config: Config,
        last_measurement: Option<Measurement>,
    ) -> Result<Measurement, Box<dyn Error>> {

        println!("Reading measurement from {}...", &self.config.usb_device_path);

        // open usb serial port
        let port = serialport::new(&self.config.usb_device_path, 115200)
            .timeout(Duration::from_millis(10))
            .open()?;

        let mut reader = BufReader::new(port);

        let mut measurement = Measurement {
            id: Uuid::new_v4().to_string(),
            source: String::from("jarvis-p1-exporter"),
            location: config.location.clone(),
            samples: Vec::new(),
            measured_at_time: Utc::now(),
        };

        let mut has_recorded_reading: HashMap<String, bool> = HashMap::new();

        while has_recorded_reading.len() < config.sample_configs.len() {

            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(len) => {
                    println!("{} ({} chars)", &line, len);

                    for sample_config in config.sample_configs.iter() {
                        if !line.starts_with(&sample_config.prefix) {
                            continue;
                        }

                        println!("{} matches config {:?}", &line, &sample_config);

                        if len
                            < (sample_config.value_start_index + sample_config.value_length).into()
                        {
                            println!("Line with length {} is too short to extract value for reading '{}'", len, sample_config.sample_name);
                            break;
                        }

                        let value_as_string = &line[sample_config.value_start_index.into()
                            ..(sample_config.value_start_index + sample_config.value_length)
                                .into()];
                        let mut value_as_float: f64 = match value_as_string.parse() {
                            Ok(f) => f,
                            Err(e) => {
                                eprintln!(
                                    "Failed parsing float '{}' for reading '{}': {}",
                                    value_as_string, sample_config.sample_name, e
                                );
                                break;
                            }
                        };

                        value_as_float = value_as_float * sample_config.value_multiplier;
                        println!("{}: {}", sample_config.sample_name, value_as_float);

                        match has_recorded_reading.get(&sample_config.prefix) {
                            Some(_) => {
                                println!(
                                    "A reading for {} has already been recorded",
                                    sample_config.sample_name
                                )
                            }
                            None => {
                                measurement.samples.push(Sample {
                                    entity_type: sample_config.entity_type,
                                    entity_name: sample_config.entity_name.clone(),
                                    sample_type: sample_config.sample_type,
                                    sample_name: sample_config.sample_name.clone(),
                                    metric_type: sample_config.metric_type,
                                    value: value_as_float,
                                });

                                has_recorded_reading.insert(sample_config.prefix.clone(), true);
                            }
                        }

                        break;
                    }
                }
                // if timeout just read again
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                Err(e) => return Err(Box::new(e)),
            }
        }

        println!(
            "Collected {} readings, stop reading for more",
            measurement.samples.len()
        );

        match last_measurement {
            Some(lm) => {
                measurement.samples = self.sanitize_samples(measurement.samples, lm.samples)
            }
            None => {}
        }

        println!(
            "Read measurement from {}",
            &self.config.usb_device_path
        );

        Ok(measurement)
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
