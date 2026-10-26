//! Package grants are explicit intents; only the native steward signer can admit them.
use super::{types::*, worker::LaunchHandle};
use dioxus::prelude::*;

#[component]
pub(super) fn PackageGrantControls(
    handle: LaunchHandle,
    view: Signal<LaunchView>,
    status: Signal<String>,
    busy: Signal<bool>,
) -> Element {
    let mut filename = use_signal(|| "working.hmc".to_string());
    let mut package_id = use_signal(String::new);
    let mut digest = use_signal(String::new);
    let mut subject = use_signal(String::new);
    let mut grant_id = use_signal(String::new);
    let mut evidence = use_signal(String::new);
    let mut seconds = use_signal(|| "86400".to_string());
    let grant = {
        let h = handle.clone();
        move |_| {
            let mut msg = status;
            let seconds = match seconds().parse::<u32>() {
                Ok(v) => v,
                Err(_) => {
                    msg.set("Grant duration requires whole seconds".into());
                    return;
                }
            };
            let intent = PackageGrantIntent {
                filename: filename(),
                package_id: package_id(),
                package_digest: digest(),
                subject: subject(),
                grant_id: grant_id(),
                evidence: evidence(),
                seconds,
            };
            if let Err(error) = intent.validate() {
                msg.set(error);
                return;
            }
            super::ui::dispatch(
                h.clone(),
                LaunchAction::GrantPackageReview(intent),
                view,
                status,
                busy,
            );
        }
    };
    let revoke = {
        let h = handle.clone();
        move |_| {
            super::ui::dispatch(
                h.clone(),
                LaunchAction::RevokePackageReview {
                    grant_id: grant_id(),
                    evidence: evidence(),
                },
                view,
                status,
                busy,
            )
        }
    };
    let grants = view().package_grants;
    rsx! {
        details{style:"padding:14px;border-bottom:1px solid #465463;",
            summary{"Steward: exact saved-package review grants"}
            p{"Use the release identifier and saved HMC digest from the package workflow. Source-review grants do not grant this permission. Grant/revoke signs new admission and locks the session."}
            div{style:"display:flex;gap:8px;flex-wrap:wrap;",
                input{value:"{filename}",aria_label:"Review-grant HMC filename",disabled:busy(),oninput:move|e|filename.set(e.value())}
                input{value:"{package_id}",aria_label:"Review-grant package release identifier",placeholder:"Project-scoped release identifier",disabled:busy(),oninput:move|e|package_id.set(e.value())}
                input{value:"{digest}",aria_label:"Exact HMC digest for review grant",placeholder:"Full saved HMC SHA256",disabled:busy(),oninput:move|e|digest.set(e.value())}
                input{value:"{subject}",aria_label:"Admitted package reviewer subject",placeholder:"Admitted reviewer subject",disabled:busy(),oninput:move|e|subject.set(e.value())}
                input{value:"{grant_id}",aria_label:"Package review grant identifier",placeholder:"Explicit grant identifier",disabled:busy(),oninput:move|e|grant_id.set(e.value())}
                input{value:"{evidence}",aria_label:"Package grant decision evidence",placeholder:"Decision evidence reference",disabled:busy(),oninput:move|e|evidence.set(e.value())}
                input{value:"{seconds}",aria_label:"Requested package review grant duration in seconds",disabled:busy(),oninput:move|e|seconds.set(e.value())}
                button{disabled:busy(),onclick:grant,"Grant review of this exact package"}
                button{disabled:busy()||grant_id().trim().is_empty()||evidence().trim().is_empty(),onclick:revoke,"Revoke this package-review grant"}
            }
            for grant in grants {
                p{style:"overflow-wrap:anywhere;","Grant {grant.id} · {grant.reviewer_subject} · {grant.target.filename} · SHA256 {crate::graphical::package::hex_digest(&grant.target.package_digest)} · valid {grant.valid_from_unix}..{grant.valid_until_unix} UNIX seconds · revoked {grant.revoked}"}
            }
            p{"The displayed duration is an upper bound; expiry also respects current project and reviewer validity. Native stewardship and the protected authority signer are checked at execution."}
        }
    }
}
