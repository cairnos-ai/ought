use ought_spec::Clause;

use crate::context::GenerationContext;
use crate::generator::{ClauseGroup, GeneratedTest, Generator};

use super::{
    build_batch_prompt, build_prompt, derive_file_path, exec_cli_verbose, exec_cli_with_arg,
    parse_batch_response,
};

/// Generates tests by exec-ing the `claude` CLI.
pub struct ClaudeGenerator {
    model: Option<String>,
}

impl ClaudeGenerator {
    pub fn new(model: Option<String>) -> Self {
        Self { model }
    }

    fn build_args(&self, prompt: String) -> Vec<String> {
        let mut args: Vec<String> = vec!["-p".into()];
        if let Some(ref model) = self.model {
            args.push("--model".into());
            args.push(model.clone());
        }
        args.push(prompt);
        args
    }

    fn exec(&self, prompt: String, verbose: bool) -> anyhow::Result<String> {
        let args = self.build_args(prompt);
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        if verbose {
            exec_cli_verbose("claude", &args_ref, None)
        } else {
            exec_cli_with_arg("claude", &args_ref)
        }
    }
}

impl Generator for ClaudeGenerator {
    fn generate(
        &self,
        clause: &Clause,
        context: &GenerationContext,
    ) -> anyhow::Result<GeneratedTest> {
        let prompt = build_prompt(clause, context);
        let code = self.exec(prompt, context.verbose)?;
        let file_path = derive_file_path(clause, context.target_language);

        Ok(GeneratedTest {
            clause_id: clause.id.clone(),
            code,
            language: context.target_language,
            file_path,
        })
    }

    fn generate_batch(
        &self,
        group: &ClauseGroup<'_>,
        context: &GenerationContext,
    ) -> anyhow::Result<Vec<GeneratedTest>> {
        if group.clauses.is_empty() {
            return Ok(vec![]);
        }
        if group.clauses.len() == 1 {
            return Ok(vec![self.generate(group.clauses[0], context)?]);
        }

        let prompt = build_batch_prompt(group, context);
        let response = self.exec(prompt, context.verbose)?;
        Ok(parse_batch_response(&response, group, context.target_language))
    }
}
