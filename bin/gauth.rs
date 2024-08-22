use google_calendar3::api::{Event, EventDateTime};
use google_calendar3::{hyper, hyper_rustls, oauth2, CalendarHub};
use tokio;

#[tokio::main]
async fn main() {
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

    // Create a new event
    let event = Event {
        summary: Some("Sent via API by Rust!".to_string()),
        location: Some("A Test Meeting".to_string()),
        start: Some(EventDateTime {
            date_time: Some(chrono::Utc::now()),
            time_zone: Some("Europe/Paris".to_string()),
            ..Default::default()
        }),
        end: Some(EventDateTime {
            date_time: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
            time_zone: Some("Europe/Paris".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    // Insert the event into the calendar
    let result = hub.events().insert(event, &test_calendar_id).doit().await;

    match result {
        Ok((_, event)) => {
            println!("Event created: {:?}", event.html_link);
        }
        Err(e) => {
            println!("Error creating event: {:?}", e);
        }
    }
}
