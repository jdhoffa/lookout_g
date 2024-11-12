use chrono::{DateTime, NaiveDateTime, Utc};
use chrono_tz::Tz;
use google_calendar3::api::{Event, EventDateTime};
use google_calendar3::{hyper, hyper_rustls, oauth2, CalendarHub};
use hyper::client::HttpConnector;
use hyper_rustls::HttpsConnector;
use ical::IcalParser;
use reqwest::Client;
use std::error::Error;
use std::io::Cursor;

pub struct Config {
    pub ics_url: String,
    pub google_calendar_name: String,
    pub google_credentials_path: String,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();

        let ics_url = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get an ICS URL string"),
        };

        let google_calendar_name = match args.next() {
            Some(arg) => arg,
            None => String::from("Test"),
        };

        let google_credentials_path = match args.next() {
            Some(arg) => arg,
            None => String::from("credentials.json"),
        };

        Ok(Config {
            ics_url,
            google_calendar_name,
            google_credentials_path,
        })
    }
}

pub struct OutlookEvent {
    pub id: Option<String>,
    pub summary: String,
    pub location: Option<String>,
    pub description: Option<String>,
    pub start: OutlookEventDateTime,
    pub end: OutlookEventDateTime,
}

pub struct OutlookEventDateTime {
    pub date_time: String,
    pub params: Option<Vec<(String, Vec<String>)>>,
}

pub fn outlook_to_google(outlook_event: OutlookEvent) -> Event {
    Event {
        //id: outlook_event.id,
        summary: Some(outlook_event.summary),
        location: outlook_event.location,
        description: outlook_event.description,
        start: Some(convert_event_datetime(outlook_event.start)),
        end: Some(convert_event_datetime(outlook_event.end)),
        ..Default::default()
    }
}

pub fn convert_event_datetime(event_datetime: OutlookEventDateTime) -> EventDateTime {
    fn date_time_to_naive(date_str: &str) -> Option<DateTime<Utc>> {
        if date_str.len() >= 15 {
            let naive_date_time = NaiveDateTime::parse_from_str(date_str, "%Y%m%dT%H%M%S").ok()?;
            let date_time: DateTime<Utc> =
                DateTime::from_naive_utc_and_offset(naive_date_time, Utc);
            Some(date_time)
        } else {
            None
        }
    }

    pub fn map_timezone_name(tz_name: &str) -> Option<Tz> {
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

    EventDateTime {
        date_time: date_time_to_naive(&event_datetime.date_time),
        time_zone: time_zone
            .and_then(|tz_name| map_timezone_name(&tz_name).map(|tz| tz.name().to_string())),
        ..Default::default()
    }
}

async fn fetch_and_parse_ics(ics_url: &str) -> Result<Vec<Event>, Box<dyn Error>> {
    let client = Client::new();
    let response = client
        .get(ics_url)
        .send()
        .await
        .expect("Failed to send request");

    let ics_data = response.bytes().await?;

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

                    let google_event = outlook_to_google(event);

                    if let Some(this_date_time) = google_event.start.clone().unwrap().date_time {
                        let this_date = this_date_time.date_naive();
                        if this_date >= Utc::now().date_naive() {
                            events.push(google_event);
                        }
                    } else {
                        println!("Event does not have a start date");
                        continue;
                    }
                }
            }
            Err(e) => eprintln!("Error parsing calendar: {:?}", e),
        }
    }

    Ok(events)
}

async fn create_hub() -> CalendarHub<HttpsConnector<HttpConnector>> {
    // Load OAuth2 credentials from the `credentials.json` file
    let secret = oauth2::read_application_secret("credentials.json")
        .await
        .expect("Failed to read client secret");

    // Create an authenticator
    let auth = oauth2::InstalledFlowAuthenticator::builder(
        secret,
        oauth2::InstalledFlowReturnMethod::HTTPRedirect,
    )
    .persist_tokens_to_disk("tokencache.json")
    .build()
    .await
    .expect("Failed to create authenticator");

    // Set up the Google Calendar API hub
    let hub = CalendarHub::new(
        hyper::Client::builder().build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .unwrap()
                .https_or_http()
                .enable_http1()
                .build(),
        ),
        auth,
    );

    hub
}

pub async fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let events = fetch_and_parse_ics(&config.ics_url).await?;
    let hub = create_hub().await;

    // Retrieve the list of calendars
    let calendars = hub
        .calendar_list()
        .list()
        .doit()
        .await
        .expect("Failed to list calendars");

    // Find the calendar with the summary "Test"
    let test_calendar_id = calendars
        .1
        .items
        .unwrap_or_default()
        .into_iter()
        .find(|cal| cal.summary.as_deref() == Some("Test"))
        .expect("Test calendar not found")
        .id
        .expect("Test calendar has no ID");

    // Insert the event into the calendar
    let result = hub
        .events()
        .insert(events[1].clone(), &test_calendar_id)
        .doit()
        .await;

    match result {
        Ok((_, event)) => {
            println!("Event created: {:?}", event.html_link);
        }
        Err(e) => {
            println!("Error creating event: {:?}", e);
        }
    }

    let json_output = serde_json::to_string_pretty(&events[1])?;

    println!("{json_output}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*; // Replace `your_crate_name` with the name of your crate.
    use mockito;
    use tokio;

    #[test]
    fn test_config_build_success() {
        let args = vec![
            "program_name".to_string(),
            "https://example.com/calendar.ics".to_string(),
            "MyCalendar".to_string(),
            "path/to/credentials.json".to_string(),
        ];
        let config = Config::build(args.into_iter()).unwrap();
        assert_eq!(config.ics_url, "https://example.com/calendar.ics");
        assert_eq!(config.google_calendar_name, "MyCalendar");
        assert_eq!(config.google_credentials_path, "path/to/credentials.json");
    }

    #[test]
    fn test_config_build_missing_args() {
        let args = vec![
            "program_name".to_string(),
            "https://example.com/calendar.ics".to_string(),
        ];
        let result = Config::build(args.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().google_calendar_name, "Test");
    }

    #[test]
    fn test_outlook_to_google_conversion() {
        let outlook_event = OutlookEvent {
            id: Some("event123".to_string()),
            summary: "Meeting".to_string(),
            location: Some("Boardroom".to_string()),
            description: Some("Discuss project updates".to_string()),
            start: OutlookEventDateTime {
                date_time: "20231111T100000".to_string(),
                params: None,
            },
            end: OutlookEventDateTime {
                date_time: "20231111T110000".to_string(),
                params: None,
            },
        };

        let google_event = outlook_to_google(outlook_event);
        assert_eq!(google_event.summary.unwrap(), "Meeting");
        assert_eq!(google_event.location.unwrap(), "Boardroom");
        assert_eq!(google_event.description.unwrap(), "Discuss project updates");
        assert!(google_event.start.is_some());
        assert!(google_event.end.is_some());
    }

    #[test]
    fn test_convert_event_datetime_valid() {
        let outlook_dt = OutlookEventDateTime {
            date_time: "20231111T100000".to_string(),
            params: Some(vec![("TZID".to_string(), vec!["UTC".to_string()])]),
        };

        let event_dt = convert_event_datetime(outlook_dt);
        assert!(event_dt.date_time.is_some());
        assert_eq!(event_dt.time_zone.unwrap(), "UTC");
    }

    #[test]
    fn test_convert_event_datetime_invalid_date() {
        let outlook_dt = OutlookEventDateTime {
            date_time: "invalid_date".to_string(),
            params: Some(vec![("TZID".to_string(), vec!["UTC".to_string()])]),
        };

        let event_dt = convert_event_datetime(outlook_dt);
        assert!(event_dt.date_time.is_none());
    }

    #[tokio::test]
    async fn test_fetch_and_parse_ics() {
        let start_time = chrono::Local::now();
        let end_time = chrono::Local::now() + chrono::Duration::hours(1);

        // Define the mock ICS data
        let ics_data = format!(
            "BEGIN:VCALENDAR
BEGIN:VEVENT
SUMMARY:Test Event
LOCATION:Test Location
DESCRIPTION:Test Description
DTSTART;TZID=Romance Standard Time:{}
DTEND;TZID=Romance Standard Time:{}
END:VEVENT
END:VCALENDAR",
            start_time.format("%Y%m%dT%H%M%S"),
            end_time.format("%Y%m%dT%H%M%S")
        );

        let mut server = mockito::Server::new_async().await;

        // Set up the mock server to return the ICS data
        let mock = server
            .mock("GET", "/test.ics")
            .with_status(200)
            // .with_header("content-type", "text/calendar")
            .with_body(ics_data)
            .create();

        // Create the URL to point to the mock server's endpoint
        let ics_url = format!("{}/test.ics", &server.url());

        // Call the function and capture the result
        let events = fetch_and_parse_ics(&ics_url)
            .await
            .expect("Failed to fetch and parse ICS");

        // Ensure the mock server was called as expected
        mock.assert();

        // Validate the parsed event data
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.summary.as_deref(), Some("Test Event"));
        assert_eq!(event.location.as_deref(), Some("Test Location"));
        assert_eq!(event.description.as_deref(), Some("Test Description"));
        assert!(event.start.is_some());
        assert!(event.end.is_some());
    }
}
