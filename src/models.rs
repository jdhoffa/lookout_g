use serde::{Deserialize, Serialize};

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
    pub dateTime: String,
    pub timeZone: String,
}
