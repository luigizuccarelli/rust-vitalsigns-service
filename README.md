# Rust based Vital Signs microservice

## Usage

### server

```sh
cd rust-vitalsigns-service
cargo run
# Started http server: 127.0.0.1:8080
```

### curl

Create a test file and add the contents

```sh
vi payload.json

{
  "name":"iot-paas",
  "deviceId":"id123444",
  "patientId":"pid3423423",
  "data":[
     {
        "hr":80,
        "bps":120,
        "bpd":80,
        "spo2":96,
        "custom":{
          "tp":34.7,
          "rr":20,
          "etc":"123"
        },
        "date":"17-03-2023"
     }
  ]
}
```

Curl the endpoint /streamdata

```sh
curl -d'@payload.json' http://127.0.0.1:8080/streamdata | jq
{
  "name":"iot-paas",
  "deviceId":"id123444",
  "patientId":"pid3423423",
  "data":[
    {
      "hr":80,
      "bps":120,
      "bpd":80,
      "spo2":96,
      "custom":{
        "tp":34.7,
        "rr":20,
        "etc":"123"
      },
      "date":"17-03-2023"
    }
  ]
}
```

### testing

```sh
cargo test
```
