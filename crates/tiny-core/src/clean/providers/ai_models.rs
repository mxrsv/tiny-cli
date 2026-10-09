use std::path::Path;

use crate::error::Result;

use super::{dir_roots_as_items, CleanProvider, ReportOnly};
use crate::clean::fs_safe::list_children;
use crate::clean::scan_context::ScanContext;
use crate::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};

const ID: &str = "ai-models";
const LABEL: &str = "AI models";

/// Blobs are shared between Ollama models, so the folder is one item.
const OLLAMA: &str = ".ollama/models";
/// One item per child (`models--org--name`, publisher folder).
const PER_CHILD: &[&str] = &[".cache/huggingface/hub", ".lmstudio/models"];
const HOW_TO_REMOVE: &str =
    "AI models are measured only; remove them with `ollama rm`, LM Studio or `huggingface-cli delete-cache`";

pub struct AiModels;

impl AiModels {
    pub(crate) fn discover_in(&self, ctx: &ScanContext<'_>, home: &Path) -> Vec<CleanItem> {
        let mut roots = vec![home.join(OLLAMA)];
        for dir in PER_CHILD {
            roots.extend(list_children(&home.join(dir), ctx));
        }
        dir_roots_as_items(ctx, &roots, ID, LABEL, RiskLevel::Review)
    }
}

impl CleanProvider for AiModels {
    fn id(&self) -> &'static str {
        ID
    }
    fn label(&self) -> &'static str {
        LABEL
    }
    fn inclusion_reason(&self) -> String {
        "Local models downloaded by Ollama, LM Studio and Hugging Face; shown for their size only"
            .into()
    }
    fn risk(&self) -> RiskLevel {
        RiskLevel::Review
    }
    fn desktop_report_only_reason(&self) -> ReportOnly {
        ReportOnly::SizeOnly
    }
    fn discover(&self, ctx: &ScanContext<'_>) -> Result<Vec<CleanItem>> {
        Ok(super::home()
            .map(|h| self.discover_in(ctx, &h))
            .unwrap_or_default())
    }
    /// Size only: nothing is moved or deleted, from the CLI either.
    fn execute(&self, items: &[CleanItem], _action: ExecAction) -> Result<ExecReport> {
        Ok(ExecReport {
            failed: items
                .iter()
                .map(|item| (item.path.clone(), HOW_TO_REMOVE.to_string()))
                .collect(),
            ..ExecReport::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clean::providers::test_home::TestHome;

    #[test]
    fn measures_each_model_store() {
        let home = TestHome::new("ai-models");
        home.file(".ollama/models/blobs/sha256-abc", 8);
        home.file(".ollama/id_ed25519", 1);
        home.file(".cache/huggingface/hub/models--org--tiny/blobs/x", 4);
        home.file(".cache/huggingface/token", 1);
        home.dir(".lmstudio/models/lmstudio-community/Qwen");
        let items = AiModels.discover_in(&ScanContext::unchecked(), home.path());
        assert_eq!(
            home.relative(&items),
            [
                ".ollama/models",
                ".cache/huggingface/hub/models--org--tiny",
                ".lmstudio/models/lmstudio-community"
            ]
        );
        assert_eq!(items[0].size, 8);
    }

    #[test]
    fn is_report_only_and_never_removes_anything() {
        let home = TestHome::new("ai-models-exec");
        home.file(".ollama/models/blobs/sha256-abc", 8);
        let items = AiModels.discover_in(&ScanContext::unchecked(), home.path());
        assert_eq!(
            super::super::desktop_report_only(&AiModels),
            Some(ReportOnly::SizeOnly)
        );
        for action in [ExecAction::Trash, ExecAction::HardDelete] {
            let report = AiModels.execute(&items, action).unwrap();
            assert!(report.removed_paths.is_empty());
            assert_eq!(report.failed.len(), 1);
        }
        assert!(home.path().join(".ollama/models/blobs/sha256-abc").exists());
    }
}
