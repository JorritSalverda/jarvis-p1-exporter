use crate::model::Config;
use chrono::Utc;
use jarvis_lib::measurement_client::MeasurementClient;
use jarvis_lib::model::{Measurement, MetricType, Sample};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::time::Duration;
use tracing::{debug, info};
use uuid::Uuid;

pub struct P1ClientConfig {
    usb_device_path: String,
}

impl P1ClientConfig {
    pub fn new(usb_device_path: String) -> Result<Self, Box<dyn Error>> {
        debug!("P1ClientConfig::new(usb_device_path: {})", usb_device_path);
        Ok(Self { usb_device_path })
    }

    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        let usb_device_path =
            env::var("P1_USB_DEVICE_PATH").unwrap_or_else(|_| "/dev/ttyUSB0".to_string());

        Self::new(usb_device_path)
    }
}

pub struct P1Client {
    config: P1ClientConfig,
}

impl MeasurementClient<Config> for P1Client {
    fn get_measurements(
        &self,
        config: Config,
        last_measurements: Option<Vec<Measurement>>,
    ) -> Result<Vec<Measurement>, Box<dyn Error>> {
        info!(
            "Reading measurements from {}...",
            &self.config.usb_device_path
        );

        // open usb serial port
        let port = serialport::new(&self.config.usb_device_path, 115200)
            .timeout(Duration::from_millis(10))
            .open()
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to open usb serial port at {}",
                    &self.config.usb_device_path
                )
            });

        let mut reader = BufReader::new(port);

        let mut measurement = Measurement {
            id: Uuid::now_v7().to_string(),
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
                    info!("{} ({} chars)", &line, len);

                    for sample_config in config.sample_configs.iter() {
                        if !line.starts_with(&sample_config.prefix) {
                            continue;
                        }

                        info!("{} matches config {:?}", &line, &sample_config);

                        if len
                            < (sample_config.value_start_index + sample_config.value_length).into()
                        {
                            info!("Line with length {} is too short to extract value for reading '{}'", len, sample_config.sample_name);
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

                        value_as_float *= sample_config.value_multiplier;
                        info!("{}: {}", sample_config.sample_name, value_as_float);

                        match has_recorded_reading.get(&sample_config.prefix) {
                            Some(_) => {
                                info!(
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

        info!(
            "Collected {} readings, stop reading for more",
            measurement.samples.len()
        );

        if let Some(lm) = last_measurements {
            if !lm.is_empty() {
                measurement.samples =
                    self.sanitize_samples(measurement.samples, &lm[lm.len() - 1].samples)
            }
        }

        info!("Read measurements from {}", &self.config.usb_device_path);

        Ok(vec![measurement])
    }
}

impl P1Client {
    pub fn new(config: P1ClientConfig) -> Self {
        Self { config }
    }

    fn sanitize_samples(
        &self,
        current_samples: Vec<Sample>,
        last_samples: &[Sample],
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
                        && (current_sample.value < last_sample.value
                            || current_sample.value / last_sample.value > 1.1)
                    {
                        sanitize = true;
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
