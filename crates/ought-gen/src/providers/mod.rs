pub mod claude;
pub mod custom;
pub mod ollama;
pub mod openai;

use std::fmt::Write as _;
use std::path::PathBuf;

use ought_spec::Clause;

use crate::context::GenerationContext;
use crate::generator::{Generator, Language};

/// Create a generator from the provider name in config.
pub fn from_config(provider: &str, model: Option<&str>) -> anyhow::Result<Box<dyn Generator>> {
    match provider.to_lowercase().as_str() {
        "anthropic" | "claude" => {
            Ok(Box::new(claude::ClaudeGenerator::new(model.map(String::from))))
        }
        "openai" | "chatgpt" => {
            Ok(Box::new(openai::OpenAiGenerator::new(model.map(String::from))))
        }
        "ollama" => {
            let model = model
                .map(String::from)
                .unwrap_or_else(|| "llama3".to_string());
            Ok(Box::new(ollama::OllamaGenerator::new(model)))
        }
        other => {
            // Try as a custom executable path
            let path = PathBuf::from(other);
            Ok(Box::new(custom::CustomGenerator::new(path)))
        }
    }
}

/// Build a prompt for the LLM from a clause and its generation context.
pub fn build_prompt(clause: &Clause, context: &GenerationContext) -> String {
    let mut prompt = String::new();

    // Instructions
    let lang_name = language_name(context.target_language);
    let _ = writeln!(
        prompt,
        "You are a test generation assistant. Generate a single, self-contained {lang_name} test \
         function for the following specification clause. Output ONLY the test code with no \
         explanation, no markdown fences, and no surrounding text."
    );
    prompt.push('\n');

    // Clause details
    let _ = writeln!(prompt, "## Clause");
    let _ = writeln!(prompt, "- Keyword: {}", keyword_str(clause.keyword));
    let _ = writeln!(prompt, "- Severity: {:?}", clause.severity);
    let _ = writeln!(prompt, "- ID: {}", clause.id);
    let _ = writeln!(prompt, "- Text: {}", clause.text);

    if let Some(ref condition) = clause.condition {
        let _ = writeln!(prompt, "- GIVEN condition: {condition}");
    }

    if let Some(ref temporal) = clause.temporal {
        match temporal {
            ought_spec::Temporal::Invariant => {
                let _ = writeln!(
                    prompt,
                    "- Temporal: MUST ALWAYS (invariant). Generate property-based or fuzz-style tests."
                );
            }
            ought_spec::Temporal::Deadline(dur) => {
                let _ = writeln!(
                    prompt,
                    "- Temporal: MUST BY {:?}. Generate a test asserting the operation completes within this duration.",
                    dur
                );
            }
        }
    }

    if !clause.otherwise.is_empty() {
        let _ = writeln!(prompt, "- This clause has OTHERWISE fallbacks.");
    }

    prompt.push('\n');

    // Hints (code blocks from the spec)
    if !clause.hints.is_empty() {
        let _ = writeln!(prompt, "## Hints");
        for hint in &clause.hints {
            let _ = writeln!(prompt, "```\n{hint}\n```");
        }
        prompt.push('\n');
    }

    // Spec-level context
    if let Some(ref ctx) = context.spec_context {
        let _ = writeln!(prompt, "## Context\n{ctx}\n");
    }

    // Source code
    if !context.source_files.is_empty() {
        let _ = writeln!(prompt, "## Source Code");
        for sf in &context.source_files {
            let _ = writeln!(prompt, "### File: {}", sf.path.display());
            let _ = writeln!(prompt, "```\n{}\n```\n", sf.content);
        }
    }

    // Schema files
    if !context.schema_files.is_empty() {
        let _ = writeln!(prompt, "## Schema Files");
        for sf in &context.schema_files {
            let _ = writeln!(prompt, "### File: {}", sf.path.display());
            let _ = writeln!(prompt, "```\n{}\n```\n", sf.content);
        }
    }

    // Output instructions
    let _ = writeln!(prompt, "## Requirements");
    let _ = writeln!(
        prompt,
        "- Include the original clause text as a doc comment on the test function."
    );
    let _ = writeln!(
        prompt,
        "- The test function name should be derived from the clause ID: {}",
        clause.id
    );
    let _ = writeln!(
        prompt,
        "- The test must be self-contained with no cross-test dependencies."
    );

    if clause.keyword == ought_spec::Keyword::Wont {
        let _ = writeln!(
            prompt,
            "- This is a WONT clause: generate an absence test (verify the capability does not exist) \
             or a prevention test (verify that attempting the behavior fails gracefully)."
        );
    }

    match context.target_language {
        Language::Rust => {
            let _ = writeln!(prompt, "- Use #[test] attribute and assert! macros.");
        }
        Language::Python => {
            let _ = writeln!(prompt, "- Use def test_... function with assert statements.");
        }
        Language::TypeScript | Language::JavaScript => {
            let _ = writeln!(
                prompt,
                "- Use test() or it() with expect() assertions (Jest style)."
            );
        }
        Language::Go => {
            let _ = writeln!(
                prompt,
                "- Use func Test...(t *testing.T) with t.Error/t.Fatal."
            );
        }
    }

    prompt
}

/// Derive the output file path from a clause ID and target language.
pub fn derive_file_path(clause: &Clause, language: Language) -> PathBuf {
    let ext = match language {
        Language::Rust => "_test.rs",
        Language::Python => "_test.py",
        Language::TypeScript => ".test.ts",
        Language::JavaScript => ".test.js",
        Language::Go => "_test.go",
    };

    let path_str = clause.id.0.replace("::", "/");
    PathBuf::from(format!("{path_str}{ext}"))
}

/// Execute a CLI command with the prompt on stdin, return stdout.
pub fn exec_cli(
    command: &str,
    args: &[&str],
    prompt: &str,
) -> anyhow::Result<String> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow::anyhow!(
                    "CLI tool '{}' not found. Please install it and ensure it is on your PATH.",
                    command
                )
            } else {
                anyhow::anyhow!("failed to spawn '{}': {}", command, e)
            }
        })?;

    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(prompt.as_bytes())?;
    }
    // Drop stdin to signal EOF
    drop(child.stdin.take());

    let output = child.wait_with_output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "'{}' exited with status {}: {}",
            command,
            output.status,
            stderr.trim()
        );
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| anyhow::anyhow!("invalid UTF-8 from '{}': {}", command, e))?;

    Ok(stdout.trim().to_string())
}

fn keyword_str(kw: ought_spec::Keyword) -> &'static str {
    match kw {
        ought_spec::Keyword::Must => "MUST",
        ought_spec::Keyword::MustNot => "MUST NOT",
        ought_spec::Keyword::Should => "SHOULD",
        ought_spec::Keyword::ShouldNot => "SHOULD NOT",
        ought_spec::Keyword::May => "MAY",
        ought_spec::Keyword::Wont => "WONT",
        ought_spec::Keyword::Given => "GIVEN",
        ought_spec::Keyword::Otherwise => "OTHERWISE",
        ought_spec::Keyword::MustAlways => "MUST ALWAYS",
        ought_spec::Keyword::MustBy => "MUST BY",
    }
}

fn language_name(lang: Language) -> &'static str {
    match lang {
        Language::Rust => "Rust",
        Language::Python => "Python",
        Language::TypeScript => "TypeScript",
        Language::JavaScript => "JavaScript",
        Language::Go => "Go",
    }
}
