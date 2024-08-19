use ical::IcalParser;
use reqwest::blocking::get;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::Cursor;

pub struct Config {
    pub ics_url: String,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 2 {
            return Err("not enough arguments");
        }

        let ics_url = args[1].clone();

        Ok(Config { ics_url })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    pub summary: String,
    pub location: Option<String>,
    pub description: Option<String>,
    pub start: EventDateTime,
    pub end: EventDateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventDateTime {
    pub date_time: String,
    pub time_zone: String,
}

pub fn fetch_and_parse_ics(ics_url: &str) -> Result<Vec<Event>, Box<dyn Error>> {
    let response = get(ics_url)?;
    let ics_data = response.bytes()?;

    let mut events = Vec::new();
    let reader = Cursor::new(ics_data);
    let parser = IcalParser::new(reader);

    for calendar in parser {
        match calendar {
            Ok(calendar) => {
                for ical_event in calendar.events {
                    let mut event = Event {
                        summary: String::new(),
                        location: None,
                        description: None,
                        start: EventDateTime {
                            date_time: String::new(),
                            time_zone: "UTC".to_string(),
                        },
                        end: EventDateTime {
                            date_time: String::new(),
                            time_zone: "UTC".to_string(),
                        },
                    };
                    for property in ical_event.properties {
                        match property.name.as_str() {
                            "SUMMARY" => event.summary = property.value.unwrap_or_default(),
                            "LOCATION" => event.location = property.value,
                            "DESCRIPTION" => event.description = property.value,
                            "DTSTART" => event.start.date_time = property.value.unwrap_or_default(),
                            "DTEND" => event.end.date_time = property.value.unwrap_or_default(),
                            _ => {}
                        }
                    }
                    events.push(event);
                }
            }
            Err(e) => eprintln!("Error parsing calendar: {:?}", e),
        }
    }

    Ok(events)
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let events = fetch_and_parse_ics(&config.ics_url)?;

    let json_output = serde_json::to_string_pretty(&events)?;

    println!("{json_output}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_fetch_and_parse_ics_success() -> Result<(), Box<dyn Error>> {
        // Define a mock ICS data
        let ics_data = "BEGIN:VCALENDAR
BEGIN:VEVENT
SUMMARY:Test Event
LOCATION:Test Location
DESCRIPTION:Test Description
DTSTART:20230801T090000Z
DTEND:20230801T100000Z
END:VEVENT
END:VCALENDAR";

        // Create a mock server that returns the ICS data
        // Request a new server from the pool
        let mut server = mockito::Server::new();

        // Create a mock
        let mock = server
            .mock("GET", "/test.ics")
            .with_status(200)
            .with_body(ics_data)
            .create();

        // Use URL configure your client
        let url = server.url();
        let ics_url = format!("{}/test.ics", url);
        let events = fetch_and_parse_ics(&ics_url)?;

        mock.assert();

        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.summary, "Test Event");
        assert_eq!(event.location.as_deref(), Some("Test Location"));
        assert_eq!(event.description.as_deref(), Some("Test Description"));
        assert_eq!(event.start.date_time, "20230801T090000Z");
        assert_eq!(event.end.date_time, "20230801T100000Z");

        Ok(())
    }

    #[test]
    fn test_fetch_and_parse_ics_invalid_url() {
        let invalid_url = "http://invalid-url";
        let result = fetch_and_parse_ics(invalid_url);
        assert!(result.is_err());
    }
}
