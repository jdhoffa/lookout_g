use lookout_g::Config;
use std::env;
use std::process;

#[tokio::main]
async fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    if let Err(e) = lookout_g::run(config).await {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
