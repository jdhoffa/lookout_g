use chrono::{DateTime, NaiveDateTime, Utc};
use chrono_tz::Tz;
use ical::IcalParser;
use reqwest::blocking::get;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::Cursor;

pub struct Config {
    pub ics_url: String,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();

        let ics_url = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get an ICS URL string"),
        };

        Ok(Config { ics_url })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OutlookEvent {
    pub id: Option<String>,
    pub summary: String,
    pub location: Option<String>,
    pub description: Option<String>,
    pub start: OutlookEventDateTime,
    pub end: OutlookEventDateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OutlookEventDateTime {
    pub date_time: String,
    pub params: Option<Vec<(String, Vec<String>)>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GoogleEvent {
    pub id: Option<String>,
    pub summary: Option<String>,
    pub location: Option<String>,
    pub description: Option<String>,
    pub start: GoogleEventDateTime,
    pub end: GoogleEventDateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GoogleEventDateTime {
    pub date: Option<String>,
    pub date_time: Option<String>,
    pub time_zone: Option<String>,
}

pub fn outlook_to_google(outlook_event: OutlookEvent) -> GoogleEvent {
    GoogleEvent {
        id: outlook_event.id,
        summary: Some(outlook_event.summary),
        location: outlook_event.location,
        description: outlook_event.description,
        start: convert_event_datetime(outlook_event.start),
        end: convert_event_datetime(outlook_event.end),
    }
}

pub fn convert_event_datetime(event_datetime: OutlookEventDateTime) -> GoogleEventDateTime {
    fn date_time_to_google_date(date_str: &str) -> Option<String> {
        if date_str.len() >= 8 {
            let year = &date_str[0..4];
            let month = &date_str[4..6];
            let day = &date_str[6..8];
            Some(format!("{}-{}-{}", year, month, day))
        } else {
            None
        }
    }

    fn date_time_to_rfc3339(date_str: &str) -> Option<String> {
        if date_str.len() >= 15 {
            let naive_date_time = NaiveDateTime::parse_from_str(date_str, "%Y%m%dT%H%M%S").ok()?;
            let date_time: DateTime<Utc> =
                DateTime::from_naive_utc_and_offset(naive_date_time, Utc);
            Some(date_time.to_rfc3339())
        } else {
            None
        }
    }

    fn map_timezone_name(tz_name: &str) -> Option<Tz> {
        match tz_name {
            "Central America Standard Time" => Some(Tz::America__Guatemala),
            "Central Europe Standard Time" => Some(Tz::Europe__Berlin), // Central European Time (CET)
            "Central Standard Time" => Some(Tz::America__Chicago), // Central Standard Time (CST) in the US
            "Eastern Standard Time" => Some(Tz::America__New_York), // Eastern Standard Time (EST) in the US
            "GMT Standard Time" => Some(Tz::Europe__London), // Greenwich Mean Time, used in the UK (not during daylight saving time)
            "Greenwich Standard Time" => Some(Tz::Etc__GMT), // Pure GMT without daylight saving adjustments
            "Mountain Standard Time" => Some(Tz::America__Denver), // Mountain Standard Time (MST) in the US
            "Pacific Standard Time" => Some(Tz::America__Los_Angeles), // Pacific Standard Time (PST) in the US
            "Romance Standard Time" => Some(Tz::Europe__Paris), // Romance Standard Time, used in Western Europe (e.g., France, Belgium)
            "SA Pacific Standard Time" => Some(Tz::America__Bogota), // South America Pacific Standard Time (e.g., Colombia)
            "US Mountain Standard Time" => Some(Tz::America__Phoenix), // MST without daylight saving in the US (e.g., Arizona)
            "UTC" => Some(Tz::UTC),                                    // Coordinated Universal Time
            "W. Europe Standard Time" => Some(Tz::Europe__Berlin), // Western Europe Standard Time (e.g., Germany, Netherlands)
            _ => None,
        }
    }

    // Extract timeZone from params if available
    let time_zone = event_datetime.params.and_then(|params| {
        params.iter().find_map(|(key, values)| {
            if key == "TZID" {
                values.get(0).cloned()
            } else {
                None
            }
        })
    });

    GoogleEventDateTime {
        date: date_time_to_google_date(&event_datetime.date_time),
        date_time: date_time_to_rfc3339(&event_datetime.date_time),
        time_zone: time_zone
            .and_then(|tz_name| map_timezone_name(&tz_name).map(|tz| tz.name().to_string())),
    }
}

pub fn fetch_and_parse_ics(ics_url: &str) -> Result<Vec<GoogleEvent>, Box<dyn Error>> {
    let response = get(ics_url)?;
    let ics_data = response.bytes()?;

    let mut events = Vec::new();
    let reader = Cursor::new(ics_data);
    let parser = IcalParser::new(reader);

    for calendar in parser {
        match calendar {
            Ok(calendar) => {
                for ical_event in calendar.events {
                    let init_params: Option<Vec<(String, Vec<String>)>> = Some(Vec::new());

                    let mut event = OutlookEvent {
                        id: None,
                        summary: String::new(),
                        location: None,
                        description: None,
                        start: OutlookEventDateTime {
                            date_time: String::new(),
                            params: init_params.clone(),
                        },
                        end: OutlookEventDateTime {
                            date_time: String::new(),
                            params: init_params.clone(),
                        },
                    };
                    for property in ical_event.properties {
                        match property.name.as_str() {
                            "UID" => event.id = property.value,
                            "SUMMARY" => event.summary = property.value.unwrap_or_default(),
                            "LOCATION" => event.location = property.value,
                            "DESCRIPTION" => event.description = property.value,
                            "DTSTART" => {
                                event.start.date_time = property.value.unwrap_or_default();
                                event.start.params = property.params.clone();
                            }
                            "DTEND" => {
                                event.end.date_time = property.value.unwrap_or_default();
                                event.end.params = property.params.clone();
                            }
                            _ => {}
                        }
                    }
                    events.push(outlook_to_google(event));
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
DTSTART;TZID=Romance Standard Time:20230801T090000
DTEND;TZID=Romance Standard Time:20230801T100000
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
        assert_eq!(event.summary, Some(String::from("Test Event")));
        assert_eq!(event.location.as_deref(), Some("Test Location"));
        assert_eq!(event.description.as_deref(), Some("Test Description"));
        assert_eq!(
            event.start.date_time,
            Some(String::from("2023-08-01T09:00:00+00:00"))
        );
        assert_eq!(
            event.end.date_time,
            Some(String::from("2023-08-01T10:00:00+00:00"))
        );
        assert_eq!(event.start.time_zone, Some(String::from("Europe/Paris")));
        assert_eq!(event.end.time_zone, Some(String::from("Europe/Paris")));

        Ok(())
    }

    #[test]
    fn test_fetch_and_parse_ics_invalid_url() {
        let invalid_url = "http://invalid-url";
        let result = fetch_and_parse_ics(invalid_url);
        assert!(result.is_err());
    }
}
