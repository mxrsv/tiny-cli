use anyhow::{bail, Result};

use crate::cli::CleanOpts;

mod fs_safe;
mod process;
mod types;

pub fn run(_opts: CleanOpts) -> Result<()> {
    bail!("tiny clean: implementation pending — orchestration arrives in a follow-up commit")
}
