use crate::config::read_config_file;

pub async fn run() -> anyhow::Result<()> {
    let _config_file = read_config_file();
    std::process::exit(0);
}
