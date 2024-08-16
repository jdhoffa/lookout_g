use crate::outlook_ics::fetch_and_parse_ics;
pub mod outlook_ics;

use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Get the ICS URL from the command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} $ICS_URL", args[0]);
        return Err("Invalid number of arguments".into());
    }

    let ics_url = &args[1];

    // Fetch and parse the ICS file
    let events = fetch_and_parse_ics(ics_url)?;

    // Output the events as JSON
    let json_output = serde_json::to_string_pretty(&events)?;
    println!("{}", json_output);

    Ok(())
}
