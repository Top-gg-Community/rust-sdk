use crate::{
  bot::{Bot, BotStats, Bots, IsWeekend, NewBotStats, QueryLike},
  http::{Http, GET, POST},
  snowflake::SnowflakeLike,
  user::{User, Voted, Voter},
  Result,
};
use core::mem::transmute;

/// A struct representing a [top.gg](https://top.gg) API client instance.
pub struct Client<'a> {
  http: Http<'a>,
}

impl<'a> Client<'a> {
  /// Creates a brand new client instance from a [top.gg](https://top.gg) token.
  ///
  /// You can get a [top.gg](https://top.gg) token if you own a listed discord bot on [top.gg](https://top.gg) (open the edit page, see in `Webhooks` section)
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Client;
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let _client = topgg::Client::new(env!("TOPGG_TOKEN"));
  /// }
  /// ```
  #[must_use]
  #[inline(always)]
  pub fn new<T>(token: &'a T) -> Self
  where
    T: AsRef<str> + ?Sized,
  {
    Self {
      http: Http::new(token.as_ref()),
    }
  }

  /// Fetches a user from a Discord ID if available.
  ///
  /// # Panics
  ///
  /// Panics if the following conditions are met:
  /// - The ID argument is a string and it's not a valid snowflake (expected things like `"123456789"`)
  /// - The client uses an invalid [top.gg](https://top.gg) API token (unauthorized)
  ///
  /// # Errors
  ///
  /// Errors if the following conditions are met:
  /// - An internal error from the client itself preventing it from sending a HTTP request to the [top.gg](https://top.gg) ([`InternalClientError`][`crate::Error::InternalClientError`])
  /// - An unexpected response from the [top.gg](https://top.gg) servers ([`InternalServerError`][`crate::Error::InternalServerError`])
  /// - The requested user does not exist ([`NotFound`][`crate::Error::NotFound`])
  /// - The client is being ratelimited from sending more HTTP requests ([`Ratelimit`][`crate::Error::Ratelimit`])
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Client;
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let client = topgg::Client::new(env!("TOPGG_TOKEN"));
  ///   
  ///   let best_user_of_all_time = client.get_user(661200758510977084u64).await.unwrap();
  ///   
  ///   assert_eq!(best_user_of_all_time.username, "null");
  ///   assert_eq!(best_user_of_all_time.discriminator, "8626");
  ///   assert_eq!(best_user_of_all_time.id, 661200758510977084u64);
  ///   
  ///   println!("{:?}", best_user_of_all_time);
  /// }
  /// ```
  pub async fn get_user<I>(&self, id: I) -> Result<User>
  where
    I: SnowflakeLike,
  {
    let path = format!("/users/{}", id.as_snowflake());

    self.http.request(GET, &path, None).await
  }

  /// Fetches a listed discord bot from a Discord ID if available.
  ///
  /// # Panics
  ///
  /// Panics if the following conditions are met:
  /// - The ID argument is a string and it's not a valid snowflake (expected things like `"123456789"`)
  /// - The client uses an invalid [top.gg](https://top.gg) API token (unauthorized)
  ///
  /// # Errors
  ///
  /// Errors if the following conditions are met:
  /// - An internal error from the client itself preventing it from sending a HTTP request to the [top.gg](https://top.gg) ([`InternalClientError`][`crate::Error::InternalClientError`])
  /// - An unexpected response from the [top.gg](https://top.gg) servers ([`InternalServerError`][`crate::Error::InternalServerError`])
  /// - The requested discord bot is not listed on [top.gg](https://top.gg) ([`NotFound`][`crate::Error::NotFound`])
  /// - The client is being ratelimited from sending more HTTP requests ([`Ratelimit`][`crate::Error::Ratelimit`])
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Client;
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let client = topgg::Client::new(env!("TOPGG_TOKEN"));
  ///   
  ///   let best_bot_ever_made = client.get_bot(264811613708746752u64).await.unwrap();
  ///   
  ///   assert_eq!(best_bot_ever_made.username, "Luca");
  ///   assert_eq!(best_bot_ever_made.discriminator, "1375");
  ///   assert_eq!(best_bot_ever_made.id, 264811613708746752u64);
  ///   
  ///   println!("{:?}", best_bot_ever_made);
  /// }
  /// ```
  pub async fn get_bot<I>(&self, id: I) -> Result<Bot>
  where
    I: SnowflakeLike,
  {
    let path = format!("/bots/{}", id.as_snowflake());

    self.http.request(GET, &path, None).await
  }

  /// Fetches a listed discord bot's statistics from a Discord ID if available.
  ///
  /// # Panics
  ///
  /// Panics if the following conditions are met:
  /// - The ID argument is a string and it's not a valid snowflake (expected things like `"123456789"`)
  /// - The client uses an invalid [top.gg](https://top.gg) API token (unauthorized)
  ///
  /// # Errors
  ///
  /// Errors if the following conditions are met:
  /// - An internal error from the client itself preventing it from sending a HTTP request to the [top.gg](https://top.gg) ([`InternalClientError`][`crate::Error::InternalClientError`])
  /// - An unexpected response from the [top.gg](https://top.gg) servers ([`InternalServerError`][`crate::Error::InternalServerError`])
  /// - The requested discord bot is not listed on [top.gg](https://top.gg) ([`NotFound`][`crate::Error::NotFound`])
  /// - The client is being ratelimited from sending more HTTP requests ([`Ratelimit`][`crate::Error::Ratelimit`])
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Client;
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let client = topgg::Client::new(env!("TOPGG_TOKEN"));
  ///   
  ///   let epic_stats = client.get_bot_stats(264811613708746752u64).await.unwrap();
  ///   
  ///   println!("{:?}", epic_stats);
  /// }
  /// ```
  pub async fn get_bot_stats<I>(&self, id: I) -> Result<BotStats>
  where
    I: SnowflakeLike,
  {
    let path = format!("/bots/{}/stats", id.as_snowflake());

    self.http.request(GET, &path, None).await
  }

  /// Posts a listed discord bot's statistics.
  ///
  /// # Panics
  ///
  /// Panics if the following conditions are met:
  /// - The ID argument is a string and it's not a valid snowflake (expected things like `"123456789"`)
  /// - The client uses an invalid [top.gg](https://top.gg) API token (unauthorized)
  /// - The client posts to a discord bot not owned by the owner of the [top.gg](https://top.gg) token. (unauthorized)
  /// - The new stats' required server count property is still empty or zero.
  ///
  /// # Errors
  ///
  /// Errors if the following conditions are met:
  /// - An internal error from the client itself preventing it from sending a HTTP request to the [top.gg](https://top.gg) ([`InternalClientError`][`crate::Error::InternalClientError`])
  /// - An unexpected response from the [top.gg](https://top.gg) servers ([`InternalServerError`][`crate::Error::InternalServerError`])
  /// - The requested discord bot is not listed on [top.gg](https://top.gg) ([`NotFound`][`crate::Error::NotFound`])
  /// - The client is being ratelimited from sending more HTTP requests ([`Ratelimit`][`crate::Error::Ratelimit`])
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::{Client, NewBotStats};
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let client = topgg::Client::new(env!("TOPGG_TOKEN"));
  ///   let my_bot_id = 123456789u64;
  ///
  ///   let stats = NewBotStats::new()
  ///     .server_count(1234); // be TRUTHFUL!
  ///
  ///   client.set_bot_stats(my_bot_id, stats).await.unwrap();
  /// }
  /// ```
  pub async fn set_bot_stats<I>(&self, id: I, new_stats: NewBotStats) -> Result<()>
  where
    I: SnowflakeLike,
  {
    assert!(
      new_stats.server_count != 0,
      "required server count property is still empty or zero"
    );

    let path = format!("/bots/{}/stats", id.as_snowflake());
    let body = unsafe { serde_json::to_string(&new_stats).unwrap_unchecked() };

    self.http.request(POST, &path, Some(&body)).await?;

    Ok(())
  }

  /// Fetches a listed discord bot's list of voters from a Discord ID if available.
  ///
  /// # Panics
  ///
  /// Panics if the following conditions are met:
  /// - The ID argument is a string and it's not a valid snowflake (expected things like `"123456789"`)
  /// - The client uses an invalid [top.gg](https://top.gg) API token (unauthorized)
  ///
  /// # Errors
  ///
  /// Errors if the following conditions are met:
  /// - An internal error from the client itself preventing it from sending a HTTP request to the [top.gg](https://top.gg) ([`InternalClientError`][`crate::Error::InternalClientError`])
  /// - An unexpected response from the [top.gg](https://top.gg) servers ([`InternalServerError`][`crate::Error::InternalServerError`])
  /// - The requested discord bot is not listed on [top.gg](https://top.gg) ([`NotFound`][`crate::Error::NotFound`])
  /// - The client is being ratelimited from sending more HTTP requests ([`Ratelimit`][`crate::Error::Ratelimit`])
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Client;
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let client = topgg::Client::new(env!("TOPGG_TOKEN"));
  ///   
  ///   for based_voter in client.get_bot_voters(264811613708746752u64).await.unwrap() {
  ///     println!("{:?}", based_voter);
  ///   }
  /// }
  /// ```
  pub async fn get_bot_voters<I>(&self, id: I) -> Result<Vec<Voter>>
  where
    I: SnowflakeLike,
  {
    let path = format!("/bots/{}/votes", id.as_snowflake());

    self.http.request(GET, &path, None).await
  }

  /// Queries/searches through the [top.gg](https://top.gg) database to look for matching listed discord bots.
  ///
  /// # Panics
  ///
  /// Panics if the following conditions are met:
  /// - The ID argument is a string and it's not a valid snowflake (expected things like `"123456789"`)
  /// - The client uses an invalid [top.gg](https://top.gg) API token (unauthorized)
  ///
  /// # Errors
  ///
  /// Errors if the following conditions are met:
  /// - An internal error from the client itself preventing it from sending a HTTP request to the [top.gg](https://top.gg) ([`InternalClientError`][`crate::Error::InternalClientError`])
  /// - An unexpected response from the [top.gg](https://top.gg) servers ([`InternalServerError`][`crate::Error::InternalServerError`])
  /// - The requested discord bot is not listed on [top.gg](https://top.gg) ([`NotFound`][`crate::Error::NotFound`])
  /// - The client is being ratelimited from sending more HTTP requests ([`Ratelimit`][`crate::Error::Ratelimit`])
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::{Client, Filter, Query};
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let client = topgg::Client::new(env!("TOPGG_TOKEN"));
  ///   
  ///   // inputting a string searches a bot that matches that username
  ///   for bot in client.get_bots("shiro").await.unwrap() {
  ///     println!("{:?}", bot);
  ///   }
  ///
  ///   // advanced query with filters
  ///   let filter = Filter::new()
  ///     .username("shiro")
  ///     .certified(true);
  ///
  ///   let query = Query::new()
  ///     .limit(250)
  ///     .skip(50)
  ///     .filter(filter);
  ///
  ///   for bot in client.get_bots(query).await.unwrap() {
  ///     println!("{:?}", bot);
  ///   }
  /// }
  /// ```
  pub async fn get_bots<Q>(&self, query: Q) -> Result<Vec<Bot>>
  where
    Q: QueryLike,
  {
    let path = format!("/bots{}", query.into_query_string());

    Ok(self.http.request::<Bots>(GET, &path, None).await?.results)
  }

  /// Checks if the specified user has voted for the listed discord bot.
  ///
  /// # Panics
  ///
  /// Panics if the following conditions are met:
  /// - The bot ID argument is a string and it's not a valid snowflake (expected things like `"123456789"`)
  /// - The user ID argument is a string and it's not a valid snowflake (expected things like `"123456789"`)
  /// - The client uses an invalid [top.gg](https://top.gg) API token (unauthorized)
  ///
  /// # Errors
  ///
  /// Errors if the following conditions are met:
  /// - An internal error from the client itself preventing it from sending a HTTP request to the [top.gg](https://top.gg) ([`InternalClientError`][`crate::Error::InternalClientError`])
  /// - An unexpected response from the [top.gg](https://top.gg) servers ([`InternalServerError`][`crate::Error::InternalServerError`])
  /// - The requested discord bot is not listed on [top.gg](https://top.gg) ([`NotFound`][`crate::Error::NotFound`])
  /// - The requested user does not exist ([`NotFound`][`crate::Error::NotFound`])
  /// - The client is being ratelimited from sending more HTTP requests ([`Ratelimit`][`crate::Error::Ratelimit`])
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Client;
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let client = topgg::Client::new(env!("TOPGG_TOKEN"));
  ///   
  ///   let best_bot_id = 264811613708746752u64;
  ///   let best_user_id = 661200758510977084u64;
  ///
  ///   if client.has_user_voted(best_bot_id, best_user_id).await.unwrap() {
  ///     println!("this user is based");
  ///   }
  /// }
  /// ```
  pub async fn has_user_voted<B, U>(&self, bot_id: B, user_id: U) -> Result<bool>
  where
    B: SnowflakeLike,
    U: SnowflakeLike,
  {
    let path = format!(
      "/bots/{}/votes?userId={}",
      bot_id.as_snowflake(),
      user_id.as_snowflake()
    );

    Ok(unsafe { transmute(self.http.request::<Voted>(GET, &path, None).await?.voted) })
  }

  /// Checks if the weekend multiplier is active.
  ///
  /// # Panics
  ///
  /// Panics if the client uses an invalid [top.gg](https://top.gg) API token (unauthorized)
  ///
  /// # Errors
  ///
  /// Errors if the following conditions are met:
  /// - An internal error from the client itself preventing it from sending a HTTP request to the [top.gg](https://top.gg) ([`InternalClientError`][`crate::Error::InternalClientError`])
  /// - An unexpected response from the [top.gg](https://top.gg) servers ([`InternalServerError`][`crate::Error::InternalServerError`])
  /// - The client is being ratelimited from sending more HTTP requests ([`Ratelimit`][`crate::Error::Ratelimit`])
  ///
  /// # Examples
  ///
  /// Basic usage:
  ///
  /// ```rust,no_run
  /// use topgg::Client;
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let client = topgg::Client::new(env!("TOPGG_TOKEN"));
  ///   
  ///   if client.is_weekend().await.unwrap() {
  ///     println!("guess what? it's the weekend! woohoo! 🎉🎉🎉🎉");
  ///   }
  /// }
  /// ```
  pub async fn is_weekend(&self) -> Result<bool> {
    Ok(unsafe {
      transmute(
        self
          .http
          .request::<IsWeekend>(GET, "/weekend", None)
          .await?
          .is_weekend,
      )
    })
  }
}
