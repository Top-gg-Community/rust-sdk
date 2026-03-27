use super::{Client, UserSource};

use serde_json::json;
use tokio::time::{Duration, sleep};

macro_rules! delayed {
  ($($b:tt)*) => {
    $($b)*
    sleep(Duration::from_secs(1)).await
  };
}

#[tokio::test]
#[allow(clippy::unreadable_literal)]
async fn api() {
  let client = Client::new(env!("TOPGG_TOKEN").into());

  delayed! {
    let _project = client.get_self().await.unwrap();
  }

  delayed! {
    client.post_commands(json!([{
      "id": "1",
      "type": 1,
      "application_id": "1",
      "name": "test",
      "description": "command description",
      "default_member_permissions": "",
      "version": "1"
    }])).await.unwrap();
  }

  delayed! {
    let _vote = client.get_vote(UserSource::Discord(661200758510977084)).await.unwrap();
  }

  delayed! {
    let _vote = client.get_vote(UserSource::Topgg(8226924471638491136)).await.unwrap();
  }

  delayed! {
    use chrono::{TimeZone, Utc};

    let since = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).single().unwrap();

    let _first_page = client.get_votes(since).await.unwrap();
    let _second_page = _first_page.next().await.unwrap();
  }
}
