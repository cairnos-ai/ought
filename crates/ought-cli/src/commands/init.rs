pub fn run() -> anyhow::Result<()> {
    if std::path::Path::new("ought.toml").exists() {
        anyhow::bail!("ought.toml already exists in this directory");
    }

    let language = if std::path::Path::new("Cargo.toml").exists() {
        "rust"
    } else if std::path::Path::new("package.json").exists() {
        "typescript"
    } else if std::path::Path::new("pyproject.toml").exists()
        || std::path::Path::new("setup.py").exists()
    {
        "python"
    } else if std::path::Path::new("go.mod").exists() {
        "go"
    } else {
        "rust"
    };

    std::fs::create_dir_all("ought")?;

    let config_content = format!(
        r#"[project]
name = "{name}"
version = "0.1.0"

[specs]
roots = ["ought/"]

[context]
search_paths = ["src/"]
exclude = ["target/", "ought/ought-gen/"]

[generator]
provider = "anthropic"

[runner.{lang}]
test_dir = "ought/ought-gen/"
"#,
        name = std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "myproject".into()),
        lang = language,
    );
    std::fs::write("ought.toml", config_content)?;

    let example_spec = r#"# Example

context: Replace this with a description of what you're specifying.
source: src/

## Basic Behavior

- **MUST** do the most important thing correctly
- **MUST NOT** do the thing that would be bad
- **SHOULD** handle edge cases gracefully
- **MAY** support optional features
"#;
    std::fs::write("ought/example.ought.md", example_spec)?;

    eprintln!("Created ought.toml and ought/example.ought.md");
    eprintln!("Detected language: {}", language);
    Ok(())
}
