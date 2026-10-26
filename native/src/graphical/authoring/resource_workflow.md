# Native surface resource workflow
Texture inputs enter the existing authorized import library at the native project's
`imports` directory. Use existing package import controls with canonical texture
key and explicit MIME kind, then bind the inspected HMC SHA256. No source path or
network URL grants access. Existing-key changed compiled data needs a new asset
key. Bind immutable resources and explicitly rebuild after reopening textured
source; rebinding never restores a session or changes source authority.

The revolution builder requires native host opt-in
`POET_NATIVE_REVOLVE_PROFILE=1`, plus authored requests and actual target grants.
The legacy host profile stays unchanged without this explicit setting.
