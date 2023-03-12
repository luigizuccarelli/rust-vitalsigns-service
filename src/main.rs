use actix_web::{error, web, App, Error, HttpResponse, HttpServer};
use futures_util::StreamExt as _;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub name: String,
    pub device_id: String,
    pub patient_id: String,
    pub data: Vec<Daum>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Daum {
    pub hr: i64,
    pub bps: i64,
    pub bpd: i64,
    pub spo2: i64,
    pub custom: Custom,
    pub date: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Custom {
    pub tp: f64,
    pub rr: i64,
    pub etc: String,
}

const MAX_SIZE: usize = 262_144; // max payload size is 256k

/// handler - reads json in chunks - overkill for simple payload
async fn index_manual(mut payload: web::Payload) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }
    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Root>(&body)?;
    Ok(HttpResponse::Ok().json(obj)) // <- send response
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting HTTP server at http://localhost:8080");

    HttpServer::new(|| {
        App::new()
            // leave logger out for better performance
            //.wrap(middleware::Logger::default())
            .app_data(web::JsonConfig::default().limit(4096)) // <- limit size of the payload (global configuration)
            .service(web::resource("/streamdata").route(web::post().to(index_manual)))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use actix_web::{dev::Service, http, test, web, App};

    use super::*;
    use assert_json_diff::assert_json_include;
    use std::str;

    #[actix_web::test]
    async fn test_index() {
        let app = test::init_service(
            App::new().service(web::resource("/").route(web::post().to(index_manual))),
        )
        .await;

        let mut vec = Vec::new();
        let d = Daum {
            hr: 66,
            bpd: 120,
            bps: 80,
            spo2: 98,
            custom: Custom {
                tp: 34.7,
                rr: 22,
                etc: "some-extra".to_owned(),
            },
            date: "17-03-2023".to_owned(),
        };
        vec.push(d);

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&Root {
                device_id: "123".to_owned(),
                patient_id: "123".to_owned(),
                name: "test-data".to_owned(),
                data: vec,
            })
            .to_request();
        let resp = app.call(req).await.unwrap();

        // check all response objects
        assert_eq!(resp.status(), http::StatusCode::OK);
        let body_bytes = actix_web::body::to_bytes(resp.into_body()).await.unwrap();
        let body_text = str::from_utf8(&body_bytes).unwrap();
        assert_json_include!(
            actual: body_text,
            expected: r##"{"name":"test-data","deviceId":"123","patientId":"123","data":[{"hr":66,"bps":80,"bpd":120,"spo2":98,"custom":{"tp":34.7,"rr":22,"etc":"some-extra"},"date":"17-03-2023"}]}"##,
        );
    }
}
