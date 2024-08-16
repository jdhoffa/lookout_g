use ical::IcalParser;
use reqwest::blocking::get;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::Cursor;

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
