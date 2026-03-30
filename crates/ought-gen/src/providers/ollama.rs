use ought_spec::Clause;

use crate::context::GenerationContext;
use crate::generator::{GeneratedTest, Generator};

use super::{build_prompt, derive_file_path, exec_cli};

/// Generates tests by exec-ing the `ollama` CLI for local models.
pub struct OllamaGenerator {
    model: String,
}

impl OllamaGenerator {
    pub fn new(model: String) -> Self {
        Self { model }
    }
}

impl Generator for OllamaGenerator {
    fn generate(
        &self,
        clause: &Clause,
        context: &GenerationContext,
    ) -> anyhow::Result<GeneratedTest> {
        let prompt = build_prompt(clause, context);

        let code = exec_cli("ollama", &["run", &self.model], &prompt)?;
        let file_path = derive_file_path(clause, context.target_language);

        Ok(GeneratedTest {
            clause_id: clause.id.clone(),
            code,
            language: context.target_language,
            file_path,
        })
    }
}
