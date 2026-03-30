use ought_spec::Clause;

use crate::context::GenerationContext;
use crate::generator::{ClauseGroup, GeneratedTest, Generator};

use super::{
    build_batch_prompt, build_prompt, derive_file_path, exec_cli, exec_cli_verbose,
    parse_batch_response,
};

/// Generates tests by exec-ing the `chatgpt` CLI.
pub struct OpenAiGenerator {
    model: Option<String>,
}

impl OpenAiGenerator {
    pub fn new(model: Option<String>) -> Self {
        Self { model }
    }

    fn exec_prompt(&self, prompt: &str, verbose: bool) -> anyhow::Result<String> {
        if verbose {
            if let Some(ref model) = self.model {
                exec_cli_verbose("chatgpt", &["-m", model.as_str()], Some(prompt))
                    .or_else(|_| {
                        exec_cli_verbose("openai", &["chat", "-m", model.as_str()], Some(prompt))
                    })
            } else {
                exec_cli_verbose("chatgpt", &[], Some(prompt))
                    .or_else(|_| exec_cli_verbose("openai", &["chat"], Some(prompt)))
            }
        } else if let Some(ref model) = self.model {
            exec_cli("chatgpt", &["-m", model.as_str()], prompt)
                .or_else(|_| exec_cli("openai", &["chat", "-m", model.as_str()], prompt))
        } else {
            exec_cli("chatgpt", &[], prompt)
                .or_else(|_| exec_cli("openai", &["chat"], prompt))
        }
    }
}

impl Generator for OpenAiGenerator {
    fn generate(
        &self,
        clause: &Clause,
        context: &GenerationContext,
    ) -> anyhow::Result<GeneratedTest> {
        let prompt = build_prompt(clause, context);
        let code = self.exec_prompt(&prompt, context.verbose)?;
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
        let response = self.exec_prompt(&prompt, context.verbose)?;
        Ok(parse_batch_response(&response, group, context.target_language))
    }
}
