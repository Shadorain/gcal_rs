//! This example showcases the Google OAuth2 process for requesting access to the Google Calendar features
//! and the user's events.
//!
//! Before running it, you'll need to generate your own Google OAuth2 credentials.
//!
//! In order to run the example call:
//!
//! ```sh
//! GOOGLE_CLIENT_ID=xxx GOOGLE_CLIENT_SECRET=yyy cargo run --example events
//! ```
//!
//! ...and follow the instructions.
use gcal::*;

#[tokio::main]
async fn main() {
    let client_id = std::env::var("GOOGLE_CLIENT_ID")
        .expect("Missing the GOOGLE_CLIENT_ID environment variable.");
    let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
        .expect("Missing the GOOGLE_CLIENT_SECRET environment variable.");
    let access_key = OAuth::naive(client_id, client_secret)
        .await
        .expect("Failed to get access key.");

    let (calendar_client, event_client) = GCalClient::new(access_key).unwrap().clients();

    let list = calendar_client
        .list(true, CalendarAccessRole::Reader)
        .await
        .unwrap();

    let start = chrono::Local::now();
    let end = start.checked_add_signed(chrono::Duration::days(7)).unwrap();

    let mut event_list = Vec::new();
    for calendar in list {
        event_list.extend(event_client.list(calendar.id, start, end).await.unwrap());
    }

    for event in &event_list {
        eprintln!("{:?} {:?}", event.id, event.summary);
    }
}