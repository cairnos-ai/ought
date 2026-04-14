use super::{load_config, resolve_spec_roots};
use crate::Cli;

pub fn run(cli: &Cli, port: u16, no_open: bool) -> anyhow::Result<()> {
    let (config_path, config) = load_config(&cli.config)?;
    let project_root = config_path
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();
    let spec_roots = resolve_spec_roots(&config, &config_path);
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(ought_server::serve(
        project_root,
        spec_roots,
        config.runner.clone(),
        port,
        !no_open,
    ))
}
