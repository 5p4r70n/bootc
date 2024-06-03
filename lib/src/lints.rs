//! # Implementation of container build lints
//!
//! This module implements `bootc container lint`.

use anyhow::Result;
use cap_std::fs::Dir;
use cap_std_ext::cap_std;
use cap_std_ext::dirext::CapStdExtDirExt as _;
use fn_error_context::context;

/// check for the existence of the /var/run directory
/// if it exists we need to check that it links to /run if not error
/// if it does not exist error.
#[context("Linting")]
pub(crate) fn lint(root: &Dir) -> Result<()> {
    let lints = [check_var_run, check_kernel];
    for lint in lints {
        lint(&root)?;
    }
    println!("Checks passed: {}", lints.len());
    Ok(())
}

fn check_var_run(root: &Dir) -> Result<()> {
    if let Some(meta) = root.symlink_metadata_optional("var/run")? {
        if !meta.is_symlink() {
            anyhow::bail!("Not a symlink: var/run");
        }
    }
    Ok(())
}

fn check_kernel(root: &Dir) -> Result<()> {
    let result = ostree_ext::bootabletree::find_kernel_dir_fs(&root)?;
    tracing::debug!("Found kernel: {:?}", result);
    Ok(())
}

#[cfg(test)]
fn fixture() -> Result<cap_std_ext::cap_tempfile::TempDir> {
    let tempdir = cap_std_ext::cap_tempfile::tempdir(cap_std::ambient_authority())?;
    Ok(tempdir)
}

#[test]
fn test_var_run() -> Result<()> {
    let root = &fixture()?;
    // This one should pass
    check_var_run(root).unwrap();
    root.create_dir_all("var/run/foo")?;
    assert!(check_var_run(root).is_err());
    root.remove_dir_all("var/run")?;
    // Now we should pass again
    check_var_run(root).unwrap();
    Ok(())
}

#[test]
fn test_kernel_lint() -> Result<()> {
    let root = &fixture()?;
    // This one should pass
    check_kernel(root).unwrap();
    root.create_dir_all("usr/lib/modules/5.7.2")?;
    root.write("usr/lib/modules/5.7.2/vmlinuz", "old vmlinuz")?;
    root.create_dir_all("usr/lib/modules/6.3.1")?;
    root.write("usr/lib/modules/6.3.1/vmlinuz", "new vmlinuz")?;
    assert!(check_kernel(root).is_err());
    root.remove_dir_all("usr/lib/modules/5.7.2")?;
    // Now we should pass again
    check_kernel(root).unwrap();
    Ok(())
}
