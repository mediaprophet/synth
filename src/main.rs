//! Direct local candidate production over existing owners. Does not invent an
//! authenticated project session or claim human/domain/package acceptance.
#[cfg(not(target_arch = "wasm32"))]
mod native_preview;
mod produce;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = if args.first().is_some_and(|arg| arg == "preview") {
        #[cfg(not(target_arch = "wasm32"))]
        { native_preview::run(&args[1..]) }
        #[cfg(target_arch = "wasm32")]
        { Err("Native `.10d` preview is unavailable on wasm32".to_owned()) }
    } else {
        produce::run(&args)
    };
    if let Err(error) = result {
        eprintln!("Synth: {error}");
        std::process::exit(1);
    }
}
