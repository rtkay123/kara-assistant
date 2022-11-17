#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;

    #[test]
    fn valid_remote_url() -> Result<(), Box<dyn std::error::Error>> {
        let url = vosk_model_url();
        assert!(Url::parse(&url).is_ok());
        let url = Url::parse(&url)?;
        assert!(Url::has_host(&url));
        Ok(())
    }
}

pub fn vosk_model_url() -> String {
    "https://alphacephei.com/vosk/models/vosk-model-small-en-us-0.15.zip".to_owned()
}
