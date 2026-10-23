use axum::{body::Body, extract::FromRequest, http::Request};
use axum_json_schema::{Json, JsonSchemaRejection};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct Payload {
    #[schemars(range(min = 1))]
    count: u32,
}

#[tokio::test]
async fn accepts_valid_request() {
    let request = Request::builder()
        .header("content-type", "application/json")
        .body(Body::from(r#"{"count":1}"#))
        .unwrap();

    let Json(payload) = Json::<Payload>::from_request(request, &()).await.unwrap();

    assert_eq!(payload.count, 1);
}

#[tokio::test]
async fn rejects_schema_violation() {
    let request = Request::builder()
        .header("content-type", "application/json")
        .body(Body::from(r#"{"count":0}"#))
        .unwrap();

    let result = Json::<Payload>::from_request(request, &()).await;

    assert!(matches!(result, Err(JsonSchemaRejection::Schema(_))));
}

#[cfg(feature = "aide")]
#[test]
fn generates_request_and_response_schemas() {
    use aide::{
        axum::{ApiRouter, routing::post},
        openapi::OpenApi,
    };

    async fn echo(Json(payload): Json<Payload>) -> Json<Payload> {
        Json(payload)
    }

    let mut api = OpenApi::default();
    let _: axum::Router = ApiRouter::new()
        .api_route("/echo", post(echo))
        .finish_api(&mut api);
    let document = serde_json::to_value(api).unwrap();
    let operation = &document["paths"]["/echo"]["post"];

    assert_eq!(
        document["components"]["schemas"]["Payload"]["properties"]["count"]["minimum"],
        1
    );
    assert_eq!(
        operation["requestBody"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/Payload"
    );
    assert_eq!(
        operation["responses"]["200"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/Payload"
    );
}
