use res_def::vosk_model_url;

#[tokio::test]
async fn verify_remote_model() -> Result<(), Box<dyn std::error::Error>> {
    use reqwest::{Client, StatusCode};

    let client = Client::new();

    let response = client.head(vosk_model_url()).send().await?;

    assert_eq!(StatusCode::OK, response.status());
    Ok(())
}
