use super::snowflake;

use chrono::{DateTime, Utc};
use serde::Deserialize;

cfg_if::cfg_if! {
  if #[cfg(feature = "api")] {
    use super::{Client, Result, Snowflake};
    use std::ops::{Deref, DerefMut};

    /// A user account from an external platform that is linked to a Top.gg user account. This data carries a [`Snowflake`].
    #[non_exhaustive]
    #[cfg_attr(docsrs, doc(cfg(feature = "api")))]
    pub enum UserSource<S> {
      Discord(S),
      Topgg(S),
    }

    impl<S> UserSource<S> {
      pub(super) const fn name(&self) -> &'static str {
        match self {
          Self::Topgg(_) => "topgg",

          Self::Discord(_) => "discord",
        }
      }
    }

    impl<S> Snowflake for UserSource<S>
    where
      S: Snowflake,
    {
      fn as_snowflake(&self) -> u64 {
        match self {
          Self::Topgg(id) | Self::Discord(id) => id.as_snowflake(),
        }
      }
    }

    /// A project's vote information.
    #[derive(Clone, Debug, Deserialize)]
    #[cfg_attr(docsrs, doc(cfg(feature = "api")))]
    pub struct Vote {
      /// The voter's ID.
      #[serde(deserialize_with = "snowflake::deserialize", rename = "user_id")]
      pub voter_id: u64,

      /// The voter's ID on the project's platform.
      #[serde(deserialize_with = "snowflake::deserialize")]
      pub platform_id: u64,

      /// When the vote was cast.
      #[serde(rename = "created_at")]
      pub voted_at: DateTime<Utc>,

      /// When the vote expires and the user is required to vote again.
      pub expires_at: DateTime<Utc>,

      /// The number of votes this vote counted for. This is a rounded integer value which determines how many points this individual vote was worth.
      pub weight: u64,
    }

    /// An owned variation of [`PaginatedVotes`].
    #[derive(Clone, Deserialize)]
    #[cfg_attr(docsrs, doc(cfg(feature = "api")))]
    pub struct PaginatedVotesOwned {
      #[serde(rename = "data")]
      votes: Vec<Vote>,
      cursor: String,
    }

    impl From<PaginatedVotes<'_>> for PaginatedVotesOwned {
      fn from(votes: PaginatedVotes<'_>) -> Self {
        votes.data
      }
    }

    impl PaginatedVotesOwned {
      /// Tries to advance to the next page.
      ///
      /// # Panics
      ///
      /// Panics if the client uses an invalid API token.
      ///
      /// # Errors
      ///
      /// Returns [`Err`] if:
      /// - HTTP request failure from the client-side. ([`InternalClientError`][super::Error::InternalClientError])
      /// - HTTP request failure from the server-side. ([`InternalServerError`][super::Error::InternalServerError])
      /// - Ratelimited from sending more requests. ([`Ratelimit`][super::Error::Ratelimit])
      ///
      /// # Example
      ///
      /// ```rust,no_run
      /// use chrono::{TimeZone, Utc};
      ///
      /// let since = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).single().unwrap();
      /// let first_page = PaginatedVotesOwned::from(client.get_votes(since).await.unwrap());
      ///
      /// for vote in first_page.iter() {
      ///   println!("{vote:?}");
      /// }
      ///
      /// let second_page = first_page.next(&client).await.unwrap();
      ///
      /// for vote in second_page.iter() {
      ///   println!("{vote:?}");
      /// }
      /// ```
      pub async fn next(&self, client: &Client) -> Result<Self> {
        client.get_next_votes(&self.cursor).await
      }
    }

    impl Deref for PaginatedVotesOwned {
      type Target = [Vote];

      fn deref(&self) -> &Self::Target {
        &self.votes
      }
    }

    impl DerefMut for PaginatedVotesOwned {
      fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.votes
      }
    }

    /// A paginated list of a project's vote information.
    ///
    /// For an [owned variation][PaginatedVotesOwned], pass this to [`PaginatedVotesOwned::from`].
    #[derive(Clone)]
    #[cfg_attr(docsrs, doc(cfg(feature = "api")))]
    pub struct PaginatedVotes<'c> {
      pub(super) data: PaginatedVotesOwned,
      pub(super) client: &'c Client,
    }

    impl PaginatedVotes<'_> {
      /// Tries to advance to the next page.
      ///
      /// # Panics
      ///
      /// Panics if the client uses an invalid API token.
      ///
      /// # Errors
      ///
      /// Returns [`Err`] if:
      /// - HTTP request failure from the client-side. ([`InternalClientError`][super::Error::InternalClientError])
      /// - HTTP request failure from the server-side. ([`InternalServerError`][super::Error::InternalServerError])
      /// - Ratelimited from sending more requests. ([`Ratelimit`][super::Error::Ratelimit])
      ///
      /// # Example
      ///
      /// ```rust,no_run
      /// use chrono::{TimeZone, Utc};
      ///
      /// let since = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).single().unwrap();
      /// let first_page = client.get_votes(since).await.unwrap();
      ///
      /// for vote in first_page.iter() {
      ///   println!("{vote:?}");
      /// }
      ///
      /// let second_page = first_page.next().await.unwrap();
      ///
      /// for vote in second_page.iter() {
      ///   println!("{vote:?}");
      /// }
      /// ```
      pub async fn next(&self) -> Result<Self> {
        Ok(Self {
          data: self.data.next(self.client).await?,
          client: self.client,
        })
      }
    }

    impl Deref for PaginatedVotes<'_> {
      type Target = [Vote];

      fn deref(&self) -> &Self::Target {
        self.data.deref()
      }
    }

    impl DerefMut for PaginatedVotes<'_> {
      fn deref_mut(&mut self) -> &mut Self::Target {
        self.data.deref_mut()
      }
    }
  }
}

/// A Top.gg user.
#[cfg(feature = "webhooks")]
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(docsrs, doc(cfg(feature = "webhooks")))]
pub struct User {
  /// The user's ID.
  #[serde(deserialize_with = "snowflake::deserialize")]
  pub id: u64,

  /// The user's name.
  pub name: String,

  /// The user's avatar URL.
  pub avatar_url: String,

  /// The user's platform ID.
  #[serde(deserialize_with = "snowflake::deserialize")]
  pub platform_id: u64,
}

/// A brief information of a project's vote.
#[derive(Clone, Debug, Deserialize)]
pub struct PartialVote {
  /// When the vote was cast.
  #[serde(rename = "created_at")]
  pub voted_at: DateTime<Utc>,

  /// When the vote expires and the user is required to vote again.
  pub expires_at: DateTime<Utc>,

  /// The number of votes this vote counted for. This is a rounded integer value which determines how many points this individual vote was worth.
  pub weight: u64,
}
