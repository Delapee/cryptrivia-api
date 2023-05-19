use gf256::shamir::shamir;
use serde::Deserialize;
use serde_json::json;
use std::str;
use vercel_runtime::{
    run,
    Body::{self, Binary},
    Error, Request, Response, StatusCode,
};

#[derive(Debug, Deserialize)]
struct SSS {
    shares: Vec<Vec<u8>>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    if let Binary(binary_data) = req.body() {
        if binary_data.is_empty() {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "application/json")
                .body(json!({ "error": "Empty body" }).to_string().into())?);
        }

        let string_data = match str::from_utf8(&binary_data) {
            Ok(s) => s,
            Err(_) => {
                return Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("Content-Type", "application/json")
                    .body(
                        json!({ "error": "Internal server error" })
                            .to_string()
                            .into(),
                    )?);
            }
        };

        let json_result: serde_json::Result<SSS> = serde_json::from_str(string_data);

        match json_result {
            Ok(json) => {
                let reconstruction = shamir::reconstruct(&json.shares) as Vec<u8>;

                match str::from_utf8(&reconstruction) {
                    Ok(secret) => {
                        return Ok(Response::builder()
                            .status(StatusCode::OK)
                            .header("Content-Type", "application/json")
                            .body(json!({ "secret": secret }).to_string().into())?)
                    }
                    Err(_) => {
                        return Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .header("Content-Type", "application/json")
                            .body(
                                json!({ "error": "Not enough shares to decrypt the secret" })
                                    .to_string()
                                    .into(),
                            )?)
                    }
                }
            }
            Err(_) => {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header("Content-Type", "application/json")
                    .body(json!({ "error": "Wrong body format" }).to_string().into())?)
            }
        }
    }

    Ok(Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header("Content-Type", "application/json")
        .body(json!({ "error": "Missing body" }).to_string().into())?)
}
