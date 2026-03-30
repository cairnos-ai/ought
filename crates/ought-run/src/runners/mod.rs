pub mod go;
pub mod python;
pub mod rust;
pub mod typescript;

use crate::runner::Runner;

/// Create a runner from the language name in config.
pub fn from_name(name: &str) -> anyhow::Result<Box<dyn Runner>> {
    match name.to_lowercase().as_str() {
        "rust" => Ok(Box::new(rust::RustRunner)),
        "python" => Ok(Box::new(python::PythonRunner)),
        "typescript" | "ts" => Ok(Box::new(typescript::TypeScriptRunner)),
        "go" => Ok(Box::new(go::GoRunner)),
        other => anyhow::bail!("unknown runner language: {other:?}"),
    }
}
