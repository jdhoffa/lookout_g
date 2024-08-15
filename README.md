# lookout_g

**lookout_g** is a Rust-based tool that syncs events from your Outlook calendar and pushes them to your Google Calendar. This utility is ideal for users who want to keep their calendars in sync across platforms without manual intervention.

## Features

- Automatically sync events from an Outlook Calendar to a Google Calendar.
- Bulk creation of Google Calendar events from Outlook data.
- OAuth2 authentication for secure access to Google Calendar.
- Configurable scheduling for periodic synchronization.

## Prerequisites

Before you begin, ensure you have the following installed:

- [Rust](https://www.rust-lang.org/tools/install)
- A [Google Cloud Project](https://console.cloud.google.com/) with the Google Calendar API enabled.
- An Outlook account with a calendar accessible via an `.ics` URL.

## Setup

### 1. Clone the Repository

```bash
git clone https://github.com/jdhoffa/lookout_g.git
cd lookout_g
```

### 2. Google Calendar API Credentials
- Go to the [Google Cloud Console](https://console.cloud.google.com/).
- Create a new project.
- Enable the Google Calendar API.
- Create OAuth 2.0 credentials and download the credentials.json file.
- Place credentials.json in the root directory of the project.

### 3. Configure Outlook Calendar URL
Open the main.rs file and replace the placeholder URL with your Outlook calendar .ics URL:

rust
Copy code
let outlook_calendar_url = "YOUR_OUTLOOK_CALENDAR_ICS_URL";

### 4. Install Dependencies
Ensure you have the necessary Rust dependencies by adding them to Cargo.toml:

toml
Copy code
[dependencies]
reqwest = { version = "0.11", features = ["json", "blocking"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
oauth2 = { version = "4.0", features = ["reqwest"] }
tokio = { version = "1", features = ["full"] }
Then, run:

bash
Copy code
cargo build

### 5. Run the Project
bash
Copy code
cargo run
Follow the on-screen instructions to authenticate with Google and start syncing your Outlook events to Google Calendar.

## Usage
Automatic Sync: The tool can be set up as a cron job or background service to automatically sync your calendars at regular intervals.
Manual Sync: Run the script manually whenever you need to sync your events.
Troubleshooting
Ensure your credentials.json file is correctly set up in the project root.
Make sure the .ics URL from Outlook is accessible and properly configured.
Check that you have the correct permissions enabled for the Google Calendar API in your Google Cloud project.
Contributions
Contributions are welcome! Feel free to submit a pull request or open an issue if you have suggestions or bug reports.

## License
This project is licensed under the MIT License. See the LICENSE file for details.

## Acknowledgments
* Inspired by frustration trying to get my calendars to sync automatically.
* Built with love and Rust.
