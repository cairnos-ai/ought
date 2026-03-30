use ought_spec::Clause;

use crate::context::GenerationContext;
use crate::generator::{GeneratedTest, Generator};

use super::{build_prompt, derive_file_path, exec_cli};

/// Generates tests by exec-ing the `chatgpt` CLI.
pub struct OpenAiGenerator {
    model: Option<String>,
}

impl OpenAiGenerator {
    pub fn new(model: Option<String>) -> Self {
        Self { model }
    }
}

impl Generator for OpenAiGenerator {
    fn generate(
        &self,
        clause: &Clause,
        context: &GenerationContext,
    ) -> anyhow::Result<GeneratedTest> {
        let prompt = build_prompt(clause, context);

        // Try `chatgpt` CLI first, fall back to `openai`
        let result = if let Some(ref model) = self.model {
            exec_cli("chatgpt", &["-m", model.as_str()], &prompt)
                .or_else(|_| exec_cli("openai", &["chat", "-m", model.as_str()], &prompt))
        } else {
            exec_cli("chatgpt", &[], &prompt)
                .or_else(|_| exec_cli("openai", &["chat"], &prompt))
        };

        let code = result?;
        let file_path = derive_file_path(clause, context.target_language);

        Ok(GeneratedTest {
            clause_id: clause.id.clone(),
            code,
            language: context.target_language,
            file_path,
        })
    }
}
