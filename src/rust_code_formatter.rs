use std::path::Path;
use std::process::Command;

use crate::Result;

/// Tries to format the source code of a given file.
#[allow(dead_code)]
pub(crate) fn try_format<T>(path_to_file: T) -> Result<()>
where
    T: AsRef<Path>,
{
    Command::new("rustfmt")
        .args([path_to_file.as_ref()])
        .status()?;
    Ok(())
}
