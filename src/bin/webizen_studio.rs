//! Website and live production served by Synth itself; no script backend.
use std::path::PathBuf;
fn run() -> Result<(), String> {
    let mut args=std::env::args().skip(1);
    let mut root=None; let mut port=8802_u16;
    while let Some(arg)=args.next() {
        match arg.as_str() {
            "--website-root" => root=Some(PathBuf::from(args.next().ok_or("--website-root requires a path")?)),
            "--port" => port=args.next().ok_or("--port requires a number")?.parse().map_err(|_| "Invalid port")?,
            "--help" => { println!("webizen-studio --website-root DIR [--port 8802]"); return Ok(()); },
            _ => return Err(format!("Unknown option: {arg}")),
        }
    }
    synth::web_server::configure_website_root(&root.ok_or("Pass --website-root DIR")?)?;
    println!("Native website: http://127.0.0.1:{port}/aaa-proof.html");
    synth::web_server::serve_blocking(Some(port),false)
}
fn main(){if let Err(e)=run(){eprintln!("{e}");std::process::exit(1)}}
