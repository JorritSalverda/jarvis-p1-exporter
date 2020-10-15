package p1

import (
	"bufio"
	"context"
	"strconv"
	"strings"
	"time"

	contractsv1 "github.com/JorritSalverda/jarvis-contracts-golang/contracts/v1"
	apiv1 "github.com/JorritSalverda/jarvis-p1-exporter/api/v1"
	"github.com/google/uuid"
	"github.com/rs/zerolog/log"
	"github.com/tarm/serial"
)

// Client is the interface for connecting to a modbus device via ethernet
type Client interface {
	GetMeasurement(ctx context.Context, config apiv1.Config, lastMeasurement *contractsv1.Measurement) (measurement contractsv1.Measurement, err error)
}

// NewClient returns new modbus.Client
func NewClient(ctx context.Context, p1USBDevicePath string) (Client, error) {

	return &client{
		p1USBDevicePath: p1USBDevicePath,
	}, nil
}

type client struct {
	p1USBDevicePath string
}

func (c *client) GetMeasurement(ctx context.Context, config apiv1.Config, lastMeasurement *contractsv1.Measurement) (measurement contractsv1.Measurement, err error) {

	serialConfig := &serial.Config{Name: c.p1USBDevicePath, Baud: 115200}
	usb, err := serial.OpenPort(serialConfig)
	if err != nil {
		log.Fatal().Err(err).Msgf("Failed opening usb port at %v", c.p1USBDevicePath)
	}
	defer usb.Close()

	measurement = contractsv1.Measurement{
		ID:             uuid.New().String(),
		Source:         "jarvis-p1-exporter",
		Location:       config.Location,
		Samples:        []*contractsv1.Sample{},
		MeasuredAtTime: time.Now().UTC(),
	}

	hasRecordedReading := map[string]bool{}

	reader := bufio.NewReader(usb)
	for {
		if len(hasRecordedReading) >= len(config.SampleConfigs) {
			log.Info().Msgf("Collected %v readings, stop reading for more", len(measurement.Samples))
			break
		}

		// read from usb port
		rawLine, err := reader.ReadBytes('\x0a')
		if err != nil {
			log.Fatal().Err(err).Msgf("Failed reading from usb port at %v", c.p1USBDevicePath)
		}

		line := string(rawLine[:])
		log.Debug().Msg(line)

		for _, sc := range config.SampleConfigs {
			if !strings.HasPrefix(line, sc.Prefix) {
				continue
			}

			if len(line) < sc.ValueStartIndex+sc.ValueLength {
				log.Warn().Msgf("Line with length %v is too short to extract value for reading '%v'", len(line), sc.SampleName)
				break
			}

			valueAsString := line[sc.ValueStartIndex : sc.ValueStartIndex+sc.ValueLength]
			valueAsFloat64, err := strconv.ParseFloat(valueAsString, 64)
			if err != nil {
				log.Warn().Err(err).Msgf("Failed parsing float '%v' for reading '%v'", valueAsString, sc.SampleName)
				break
			}

			valueAsFloat64 = valueAsFloat64 * sc.ValueMultiplier
			log.Info().Msgf("%v: %v", sc.SampleName, valueAsFloat64)

			if _, ok := hasRecordedReading[sc.Prefix]; !ok {
				// init sample from config
				sample := contractsv1.Sample{
					EntityType: sc.EntityType,
					EntityName: sc.EntityName,
					SampleType: sc.SampleType,
					SampleName: sc.SampleName,
					MetricType: sc.MetricType,
				}

				sample.Value = valueAsFloat64

				hasRecordedReading[sc.Prefix] = true

				measurement.Samples = append(measurement.Samples, &sample)

			} else {
				log.Warn().Msgf("A reading for %v has already been recorded", sc.SampleName)
			}

			break
		}
	}

	if lastMeasurement != nil {
		measurement.Samples = c.sanitizeSamples(measurement.Samples, lastMeasurement.Samples)
	}

	return
}

func (c *client) sanitizeSamples(currentSamples, lastSamples []*contractsv1.Sample) (sanitizeSamples []*contractsv1.Sample) {

	sanitizeSamples = []*contractsv1.Sample{}
	for _, cs := range currentSamples {
		// check if there's a corresponding sample in lastSamples and see if the difference with it's value isn't too large
		sanitize := false
		for _, ls := range lastSamples {
			if cs.EntityType == ls.EntityType &&
				cs.EntityName == ls.EntityName &&
				cs.SampleType == ls.SampleType &&
				cs.SampleName == ls.SampleName &&
				cs.MetricType == cs.MetricType {
				if cs.MetricType == contractsv1.MetricType_METRIC_TYPE_COUNTER && cs.Value < ls.Value {
					sanitize = true
					log.Warn().Msgf("Value for %v is less than the last sampled value %v, keeping previous value instead", cs, ls.Value)
					sanitizeSamples = append(sanitizeSamples, ls)
				} else if cs.MetricType == contractsv1.MetricType_METRIC_TYPE_COUNTER && cs.Value/ls.Value > 1.1 {
					sanitize = true
					log.Warn().Msgf("Value for %v is more than 10 percent larger than the last sampled value %v, keeping previous value instead", cs, ls.Value)
					sanitizeSamples = append(sanitizeSamples, ls)
				}

				break
			}
		}
		if !sanitize {
			sanitizeSamples = append(sanitizeSamples, cs)
		}
	}

	return
}
