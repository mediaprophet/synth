//! Operator-facing native project entry. Directory arguments locate public
//! configuration only; they never grant a role, adopt a key or unlock a signer.
mod create;
mod files;
mod package_admin;
mod package_grant_controls;
mod runtime;
pub mod types;
mod ui;
mod worker;
pub use runtime::Runtime;
pub use types::*;

pub fn run(args: &[String]) -> Result<(), String> {
    let (root, check) = parse(args)?;
    let runtime = Runtime::open(std::path::Path::new(root))?;
    if check {
        println!("Native project public configuration inspected; no vault opened, key adopted, source read, enrollment or window attempted");
        return Ok(());
    }
    let (worker, handle, initial) = worker::LaunchWorker::start(runtime)?;
    let attributes = dioxus_native::WindowAttributes::default()
        .with_title("Poet · Cooperative Project")
        .with_surface_size(dioxus_native::LogicalSize::new(1500, 1000));
    dioxus_native::launch_cfg_with_props(
        ui::NativeProjectShell,
        ui::NativeProjectShellProps { handle, initial },
        Vec::new(),
        vec![Box::new(attributes)],
    );
    worker.finish()
}
fn parse(args: &[String]) -> Result<(&str, bool), String> {
    match args {
        [root] if !root.is_empty() && !root.starts_with("--")=>Ok((root,false)),
        [root,flag] if !root.is_empty() && !root.starts_with("--") && flag=="--check-project"=>Ok((root,true)),
        _=>Err("Usage: poet studio --native-project EXISTING_DIRECTORY [--check-project]; project roles and keys are admitted through native controls".into()),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_entry_arguments_locate_configuration_without_granting_authority() {
        assert!(
            parse(&["C:/project".into(), "--check-project".into()])
                .unwrap()
                .1
        );
        for args in [
            vec!["--principal".into(), "owner".into()],
            vec!["C:/project".into(), "--grant".into(), "steward".into()],
        ] {
            assert!(parse(&args).is_err())
        }
    }
}

pub(crate) mod session_proof;
