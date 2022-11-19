use dirs::data_dir;
use std::path::PathBuf;

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

pub fn model_path() -> PathBuf {
    let mut data_dir = data_dir().unwrap_or_default();
    data_dir.push("kara/asr");
    data_dir
}
