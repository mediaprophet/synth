//! Direct local candidate production over existing owners. Does not invent an
//! authenticated project session or claim human/domain/package acceptance.
mod asset_cli;
#[cfg(all(feature = "preview", not(target_arch = "wasm32")))]
mod native_preview;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = if args.is_empty() || args.as_slice() == ["--help"] {
        println!("Synth commands:\n  capabilities  Inspect the admitted Vibe SDK providers\n  ontology      Inspect the category-theoretic ontology taxonomy and morphisms\n  operations    List shared-library asset operations and ontological contracts\n  profiles      List available production profiles\n  produce       Generate or strictly reopen an asset HMC\n  preview       Render .10d PNGs (requires preview feature)");
        asset_cli::run(&["--help".into()])
    } else if args.as_slice() == ["capabilities"] {
        synth::vibe_runtime::inspect().and_then(|report| {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
            );
            Ok(())
        })
    } else if args.first().is_some_and(|arg| arg == "ontology") {
        let category = args.get(1).map(String::as_str);
        let payload = if category.is_some() {
            synth::production::assets::ontology::morphisms_json(category)
        } else {
            serde_json::json!({
                "categories": synth::production::assets::ontology::categories_json(),
                "morphisms": synth::production::assets::ontology::morphisms_json(None),
            })
        };
        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
        Ok(())
    } else if args.as_slice() == ["operations"] {
        println!(
            "{}",
            serde_json::to_string_pretty(&synth::production::assets::catalogue()).unwrap()
        );
        Ok(())
    } else if args.as_slice() == ["profiles"] {
        for (id, version) in synth::production::ProducerRegistry::with_builtins().profiles() {
            println!("{id}@{version}");
        }
        Ok(())
    } else if args.first().is_some_and(|arg| arg == "preview") {
        #[cfg(all(feature = "preview", not(target_arch = "wasm32")))]
        {
            native_preview::run(&args[1..])
        }
        #[cfg(any(not(feature = "preview"), target_arch = "wasm32"))]
        {
            Err("Native `.10d` preview requires a native build with --features preview".to_owned())
        }
    } else {
        asset_cli::run(&args)
    };
    if let Err(error) = result {
        eprintln!("Synth: {error}");
        std::process::exit(1);
    }
}
