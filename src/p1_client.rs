use crate::model::{Config, ConfigSample, Measurement, MetricType, Sample};
use std::io::{self, Write};
use byteorder::{BigEndian, ByteOrder};
use chrono::Utc;
use conv::*;
use std::collections::HashMap;
use std::env;
use std::error::Error;
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
        // open usb serial port
        let port = serialport::new(&self.config.usb_device_path, 115200)
            .timeout(Duration::from_millis(10))
            .open();

        match port {
            Ok(mut port) => {
                let mut measurement = Measurement {
                    id: Uuid::new_v4().to_string(),
                    source: String::from("jarvis-p1-exporter"),
                    location: config.location.clone(),
                    samples: Vec::new(),
                    measured_at_time: Utc::now(),
                };

                let has_recorded_reading: HashMap<String, bool> = HashMap::new();

                while has_recorded_reading.len() < config.sample_configs.len() {

                    let mut serial_buf: Vec<u8> = vec![0; 1000];
                    match port.read(serial_buf.as_mut_slice()) {
                        Ok(t) => {

                          // write to stdout
                          io::stdout().write_all(&serial_buf[..t]).unwrap();

                          let line = std::str::from_utf8(&serial_buf[..t])?;

                          for sample_config in config.sample_configs.iter() {
                            if !line.starts_with(&sample_config.prefix) {
                              continue
                            }

                            if line.len() < sample_config.value_start_index + sample_config.value_length {
                              println!("Line with length {} is too short to extract value for reading '{}'", line.len(), sample_config.sample_name);
                              break
                            }

                            let value_as_string = line[sample_config.value_start_index..sample_config.value_start_index+sample_config.value_length];
                            let value_as_float: f64 = match value_as_string.parse() {
                              Ok(f) => f,
                              Err(e) => {
                                eprintln!("Failed parsing float '{}' for reading '{}': {}", value_as_string, sample_config.sample_name, e);
                                break
                              }
                            };

                            value_as_float = value_as_float * sample_config.value_multiplier;
                            println!("{}: {}", sample_config.sample_name, value_as_float);


                            // 			if _, ok := hasRecordedReading[sc.Prefix]; !ok {
                            // 				// init sample from config
                            // 				sample := contractsv1.Sample{
                            // 					EntityType: sc.EntityType,
                            // 					EntityName: sc.EntityName,
                            // 					SampleType: sc.SampleType,
                            // 					SampleName: sc.SampleName,
                            // 					MetricType: sc.MetricType,
                            // 				}

                            // 				sample.Value = valueAsFloat64

                            // 				hasRecordedReading[sc.Prefix] = true

                            // 				measurement.Samples = append(measurement.Samples, &sample)

                            // measurement.samples.push(sample);

                            // 			} else {
                            // 				log.Warn().Msgf("A reading for %v has already been recorded", sc.SampleName)
                            // 			}

                            // 			break

                          }
                        },
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
                    "Retrieved measurement via p1 from device {}",
                    &self.config.usb_device_path
                );

                Ok(measurement)
            }
            Err(e) => return Err(Box::new(e)),
        }
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

        // 	sanitizeSamples = []*contractsv1.Sample{}
        // 	for _, cs := range currentSamples {
        // 		// check if there's a corresponding sample in lastSamples and see if the difference with it's value isn't too large
        // 		sanitize := false
        // 		for _, ls := range lastSamples {
        // 			if cs.EntityType == ls.EntityType &&
        // 				cs.EntityName == ls.EntityName &&
        // 				cs.SampleType == ls.SampleType &&
        // 				cs.SampleName == ls.SampleName &&
        // 				cs.MetricType == cs.MetricType {
        // 				if cs.MetricType == contractsv1.MetricType_METRIC_TYPE_COUNTER && cs.Value < ls.Value {
        // 					sanitize = true
        // 					log.Warn().Msgf("Value for %v is less than the last sampled value %v, keeping previous value instead", cs, ls.Value)
        // 					sanitizeSamples = append(sanitizeSamples, ls)
        // 				} else if cs.MetricType == contractsv1.MetricType_METRIC_TYPE_COUNTER && cs.Value/ls.Value > 1.1 {
        // 					sanitize = true
        // 					log.Warn().Msgf("Value for %v is more than 10 percent larger than the last sampled value %v, keeping previous value instead", cs, ls.Value)
        // 					sanitizeSamples = append(sanitizeSamples, ls)
        // 				}

        // 				break
        // 			}
        // 		}
        // 		if !sanitize {
        // 			sanitizeSamples = append(sanitizeSamples, cs)
        // 		}
        // 	}

        // 	return
    }
}
