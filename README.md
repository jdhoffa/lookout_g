# lookout_g

lookout_g is a tool to sync events from an Outlook Calendar to a Google Calendar. It uses the Google Calendar API and OAuth2 for secure access to your Google Calendar. The tool can be run manually or set up as a cron job for automatic synchronization.

[![Rust](https://github.com/jdhoffa/lookout_g/actions/workflows/ci.yml/badge.svg)](https://github.com/jdhoffa/lookout_g/actions/workflows/ci.yml)
[![MIT License](https://img.shields.io/github/license/jdhoffa/lookout_g)]
## Prerequisites

Before you begin, ensure you:

- Install [Rust](https://www.rust-lang.org/tools/install)
- Set-up a [Google Cloud Project](https://console.cloud.google.com/) with the Google Calendar API enabled. You will also need to download the `credentials.json` file to authenticate with the Google Calendar API.
- Create an `.ics` URL for your Outlook. 

## Setup

### Clone the Repository

```bash
git clone https://github.com/jdhoffa/lookout_g.git
cd lookout_g
```

### Google Calendar API Credentials
- Go to the [Google Cloud Console](https://console.cloud.google.com/).
- Create a new project.
- Enable the Google Calendar API.
- Create OAuth 2.0 credentials and download the `credentials.json` file.
- Move `credentials.json` to the root directory of the project.

### Install Dependencies and Build the Project

```bash
cargo build
```

### Run the Project
You may want to set your ICS URL as an environment variable to avoid entering it every time you run the program. You can do this by adding the following line to your `.bashrc` or `.zshrc` file:

```bash
export ICS_URL="https://outlook.office.com/owa/calendar/your-calendar-id/Calendar/calendar.ics"
```

Then run:

  ```bash
  cargo run -- $ICS_URL
  ```

(Optional) You can also format the output JSON to be more readable by piping the output to the `jq` CLI (must have `jq` installed):

```bash
cargo run -- $ICS_URL | jq [.]
```

Follow the on-screen instructions to authenticate with Google and start syncing your Outlook events to Google Calendar.


Automatic Sync: The tool can be set up as a cron job or background service to automatically sync your calendars at regular intervals.

Manual Sync: Run the script manually whenever you need to sync your events.

## Contributions
Contributions are welcome! Feel free to submit a pull request or open an issue if you have suggestions or bug reports.

## License
This project is licensed under the MIT License. See the [LICENSE.md](LICENSE.md) file for details.
