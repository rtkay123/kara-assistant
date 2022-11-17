#[tokio::test]
async fn verify_remote_model() -> Result<(), Box<dyn std::error::Error>> {
    use reqwest::{Client, StatusCode};

    let client = Client::new();

    let response = client
        .head("https://alphacephei.com/vosk/models/vosk-model-small-en-us-0.15.zip")
        .send()
        .await?;

    assert_eq!(StatusCode::OK, response.status());
    Ok(())
}
