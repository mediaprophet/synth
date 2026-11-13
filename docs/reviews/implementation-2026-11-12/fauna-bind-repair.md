# Fauna bind repair — source assembled, execution pending

Owner: supervisor worker; independent acceptance remains with root. No Cargo or tests were run by this worker while assembling the change.

## Observed failure

Reviewed all three PNGs in `production/quality-20261112-122914-069f4c25/fauna-shot-0` using their existing receipt. Frame 1 separates the hind limbs from the pelvis; frame 2 severely deforms the torso. The complete asset also shows red muscle and white bone despite requesting zero visibility for those layers. This is not acceptance-quality animation.

The receipt selects clip 1 at times 0, 1/3, 2/3 from `wallabia_bicolor_complete.10d`. The portable scaffold places the pelvis/hips near y=0.33 m, with its authored left limbs on positive X. The finalizer previously sealed a generic MacropodSaltatorial skeleton: pelvis y=0.85 m and left hip on negative X. Inverse binds cancel at rest, so a rest-only test cannot detect the mismatch. Rotated poses expose it. The local clip keys already include their rest translations; this was not a delta-versus-absolute or accumulated-renderer-transform defect.

## Change

- Added strict optional `bind_landmarks` to portable scaffolds: exactly 21 named world-space metre positions. The quality wallaby supplies these positions; its hip/knee/ankle pivots match the authored long-bone endpoints.
- `vertebrate_authored_skin.rs` derives local rests and inverse binds from those landmarks and retargets existing archetype rotation deltas onto them. The original joint indices 0..17 are retained; right shoulder/elbow/wrist are appended at 18..20.
- Mirrored portable forelimb primitives now receive the appended right-chain weights when an authored binding exists. Generic eighteen-joint archetype callers remain supported.
- The anatomy accumulator and compiled output retain the authored binding. SKL1 seals the same 21 joints and tracks; the legacy eighteen-joint control view is retargeted consistently. Producer output reports `skeletal_joint_count` and `skin_bind_source`.
- Root separately identified the visibility cause: SRD1 alpha was paired with Opaque material declarations. The finalizer now multiplies authored alpha by layer visibility and calls root's material-opacity helper before sealing. Returned materials match the corrected sealed declarations.

## Regression requirements and source

Core unit `authored_pivots_preserve_joint_attachment_in_both_bent_poses` checks shared endpoints and bone lengths at rest/cruise/hop. It explicitly demonstrates that the old generic bind moves the species hip endpoint by more than 0.1 m.

Hypermedia integration `portable_authored_bind_seals_bilateral_weights_and_twenty_one_joints` compiles a portable mirrored forelimb fixture, reopens the actual SKL1, verifies 21 joints and distinct left/right influences, decodes the stored keys, and checks joint attachment at five sample times including interpolated poses. The malformed-landmark regression refuses incomplete/unknown maps.

These source tests do not establish visual quality. Final acceptance requires the combined targeted suites, fresh deterministic production, and inspection of the same three fauna poses plus the normal front/side/three-quarter views. Confirm opaque complete skin no longer reveals intentionally hidden layers, both hindlimbs remain attached, the torso stays continuous, and the independent right forelimb follows its chain. Do not replace these checks with changed hashes or triangle counts.

Legacy portable scaffolds without `bind_landmarks` retain the explicitly reported `archetype-default` bind; their anatomy-specific animation quality is not newly qualified by this correction. This fixture remains a stylized scaffold, not an anatomical or film-quality certification.
