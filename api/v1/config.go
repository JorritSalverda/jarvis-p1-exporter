package api

import (
	contractsv1 "github.com/JorritSalverda/jarvis-contracts-golang/contracts/v1"
)

type Config struct {
	Location      string         `yaml:"location"`
	SampleConfigs []ConfigSample `yaml:"sampleConfigs"`
}

type ConfigSample struct {
	// default jarvis config for sample
	EntityType contractsv1.EntityType `yaml:"entityType"`
	EntityName string                 `yaml:"entityName"`
	SampleType contractsv1.SampleType `yaml:"sampleType"`
	SampleName string                 `yaml:"sampleName"`
	MetricType contractsv1.MetricType `yaml:"metricType"`

	// modbus specific config for sample
	ValueMultiplier float64 `yaml:"valueMultiplier"`
	Prefix          string  `yaml:"prefix"`
	ValueStartIndex int     `yaml:"valueStartIndex"`
	ValueLength     int     `yaml:"valueLength"`
}
