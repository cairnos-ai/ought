use ought_spec::Clause;

use crate::context::GenerationContext;
use crate::generator::{GeneratedTest, Generator};

use super::{build_prompt, derive_file_path, exec_cli_with_arg};

/// Generates tests by exec-ing the `claude` CLI.
pub struct ClaudeGenerator {
    model: Option<String>,
}

impl ClaudeGenerator {
    pub fn new(model: Option<String>) -> Self {
        Self { model }
    }
}

impl Generator for ClaudeGenerator {
    fn generate(
        &self,
        clause: &Clause,
        context: &GenerationContext,
    ) -> anyhow::Result<GeneratedTest> {
        let prompt = build_prompt(clause, context);

        let mut args: Vec<String> = vec!["-p".into()];
        if let Some(ref model) = self.model {
            args.push("--model".into());
            args.push(model.clone());
        }
        args.push(prompt);

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let code = exec_cli_with_arg("claude", &args_ref)?;
        let file_path = derive_file_path(clause, context.target_language);

        Ok(GeneratedTest {
            clause_id: clause.id.clone(),
            code,
            language: context.target_language,
            file_path,
        })
    }
}
