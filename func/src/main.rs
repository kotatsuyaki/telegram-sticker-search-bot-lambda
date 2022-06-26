use anyhow::{Context, Result};
use lambda_http::{IntoResponse, Request, RequestExt};
use lambda_runtime::{service_fn, Error as LambdaError};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
struct FuncRequest {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FuncResponse {
    message: String,
}

async fn func(req: Request) -> Result<impl IntoResponse, LambdaError> {
    info!("Got request {:?}", req);

    let FuncRequest { name } = req
        .payload()
        .context("Payload deserialization failed")?
        .context("Payload is None")?;

    Ok(FuncResponse {
        message: format!("Goodnight, {name}!"),
    }
    .to_json())
}

fn main() -> Result<(), LambdaError> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .init();

    runtime.block_on(async {
        info!("Starting lambda runtime");

        let func = service_fn(func);
        lambda_http::run(func).await?;
        Ok(())
    })
}

trait ToJsonValue {
    fn to_json(self) -> serde_json::Value;
}

impl<T> ToJsonValue for T
where
    T: Serialize,
{
    fn to_json(self) -> serde_json::Value {
        serde_json::to_value(self).expect("to_json failed")
    }
}

#[cfg(test)]
mod test {
    use super::{func, FuncResponse};
    use http::StatusCode;
    use lambda_http::{service_fn, tower::ServiceExt, IntoResponse, Service};

    #[test]
    fn test_func() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let req_text = include_str!("../../testdata/func.json");

        runtime.block_on(async {
            let req = lambda_http::request::from_str(req_text).unwrap();
            let mut service = service_fn(func);
            let res = service
                .ready()
                .await
                .unwrap()
                .call(req)
                .await
                .unwrap()
                .into_response();

            let (parts, body) = res.into_parts();
            let func_res = serde_json::from_slice::<FuncResponse>(&body.to_vec())
                .expect("Response shape is wrong");

            assert_eq!(parts.status, StatusCode::OK);
            assert_eq!(func_res.message, "Goodnight, Taki!");
        });
    }
}
