use crate::{
  bot::{Bot, BotStats, Bots, IsWeekend, NewBotStats, QueryLike},
  http::{Http, GET, POST},
  snowflake::SnowflakeLike,
  user::{User, Voted, Voter},
  Result,
};
use core::mem::transmute;

pub struct Client<'a> {
  http: Http<'a>,
}

impl<'a> Client<'a> {
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

  pub async fn get_user<I>(&self, id: I) -> Result<User>
  where
    I: SnowflakeLike,
  {
    let path = format!("/users/{}", id.as_snowflake());

    self.http.request(GET, &path, None).await
  }
  
  pub async fn get_bot<I>(&self, id: I) -> Result<Bot>
  where
    I: SnowflakeLike,
  {
    let path = format!("/bots/{}", id.as_snowflake());

    self.http.request(GET, &path, None).await
  }

  pub async fn get_bot_stats<I>(&self, id: I) -> Result<BotStats>
  where
    I: SnowflakeLike,
  {
    let path = format!("/bots/{}/stats", id.as_snowflake());

    self.http.request(GET, &path, None).await
  }

  pub async fn set_bot_stats<I>(&self, id: I, new_stats: NewBotStats) -> Result<()>
  where
    I: SnowflakeLike,
  {
    assert!(
      new_stats.server_count != 0,
      "server count attribute is required"
    );

    let path = format!("/bots/{}/stats", id.as_snowflake());
    let body = unsafe { serde_json::to_string(&new_stats).unwrap_unchecked() };

    self.http.request(POST, &path, Some(&body)).await?;

    Ok(())
  }

  pub async fn get_bot_voters<I>(&self, id: I) -> Result<Vec<Voter>>
  where
    I: SnowflakeLike,
  {
    let path = format!("/bots/{}/votes", id.as_snowflake());

    self.http.request(GET, &path, None).await
  }

  pub async fn get_bots<Q>(&self, query: Q) -> Result<Vec<Bot>>
  where
    Q: QueryLike,
  {
    let path = format!("/bots{}", query.into_query_string());

    Ok(self.http.request::<Bots>(GET, &path, None).await?.results)
  }

  #[allow(clippy::transmute_int_to_bool)]
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

  #[allow(clippy::transmute_int_to_bool)]
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
