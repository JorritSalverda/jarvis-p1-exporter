## Installation

To install this application using Helm run the following commands: 

```bash
helm repo add jorritsalverda https://helm.jorritsalverda.com
kubectl create namespace jarvis-p1-exporter

helm upgrade \
  jarvis-p1-exporter \
  jorritsalverda/jarvis-p1-exporter \
  --install \
  --namespace jarvis-p1-exporter \
  --set secret.gcpServiceAccountKeyfile='{abc: blabla}' \
  --wait
```

# Configuration

See https://www.netbeheernederland.nl/_upload/Files/Slimme_meter_15_a727fce1f1.pdf for a list of Smart Meter codes.