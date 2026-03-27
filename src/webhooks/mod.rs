mod payload;

pub use payload::Payload;

#[cfg(feature = "actix-web")]
mod actix_web;

#[cfg(feature = "rocket")]
mod rocket;

cfg_if::cfg_if! {
  if #[cfg(feature = "axum")] {
    /// Extra helpers for working with axum.
    #[cfg_attr(docsrs, doc(cfg(feature = "axum")))]
    pub mod axum;
  }
}

cfg_if::cfg_if! {
  if #[cfg(feature = "warp")] {
    /// Extra helpers for working with warp.
    #[cfg_attr(docsrs, doc(cfg(feature = "warp")))]
    pub mod warp;
  }
}

cfg_if::cfg_if! {
  if #[cfg(any(feature = "actix-web", feature = "rocket"))] {
    use std::collections::HashMap;

    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    /// An incoming [`Payload`] that is yet to be [authenticated with a secret][IncomingPayload::authenticate].
    ///
    /// # Examples
    ///
    /// With actix-web:
    ///
    /// ```rust,no_run
    /// use topgg::IncomingPayload;
    /// use std::io;
    ///
    /// use actix_web::{
    ///   error::{Error, ErrorUnauthorized},
    ///   get, post, App, HttpServer,
    /// };
    ///
    /// #[get("/")]
    /// async fn index() -> &'static str {
    ///   "Hello, World!"
    /// }
    ///
    /// #[post("/webhook")]
    /// async fn webhook(payload: IncomingPayload) -> Result<&'static str, Error> {
    ///   match payload.authenticate(env!("TOPGG_WEBHOOK_SECRET")) {
    ///     Some(payload) => {
    ///       println!("{payload:?}");
    ///
    ///       Ok("ok")
    ///     }
    ///
    ///     _ => Err(ErrorUnauthorized("401")),
    ///   }
    /// }
    ///
    /// #[actix_web::main]
    /// async fn main() -> io::Result<()> {
    ///   HttpServer::new(|| App::new().service(index).service(webhook))
    ///     .bind("127.0.0.1:8080")?
    ///     .run()
    ///     .await
    /// }
    /// ```
    ///
    /// With rocket:
    ///
    /// ```rust,no_run
    /// use topgg::IncomingPayload;
    ///
    /// use rocket::{get, http::Status, launch, post, routes, Build, Rocket};
    ///
    /// #[get("/")]
    /// fn index() -> &'static str {
    ///   "Hello, World!"
    /// }
    ///
    /// #[post("/webhook", data = "<payload>")]
    /// fn webhook(payload: IncomingPayload) -> Status {
    ///   match payload.authenticate(env!("TOPGG_WEBHOOK_SECRET")) {
    ///     Some(payload) => {
    ///       println!("{payload:?}");
    ///
    ///       Status::Ok
    ///     },
    ///     _ => {
    ///       println!("found an unauthorized attacker.");
    ///
    ///       Status::Unauthorized
    ///     }
    ///   }
    /// }
    ///
    /// #[launch]
    /// fn rocket() -> Rocket<Build> {
    ///   rocket::build().mount("/", routes![index, webhook])
    /// }
    /// ```
    #[cfg_attr(docsrs, doc(cfg(any(feature = "actix-web", feature = "rocket"))))]
    pub struct IncomingPayload {
      t: String,
      signature: String,
      body: String,
      trace: String,
    }

    impl IncomingPayload {
      pub(super) fn new(signature: &str, body: Vec<u8>, trace: &str) -> Option<Self> {
        let signature = signature.split(',').filter_map(|p| p.split_once('=')).collect::<HashMap<_, _>>();

        Some(Self {
          t: signature.get("t")?.to_string(),
          signature: signature.get("v1")?.to_string(),
          body: String::from_utf8(body).ok()?,
          trace: trace.into(),
        })
      }

      /// Tries to authenticate a valid secret with this request.
      #[must_use]
      pub fn authenticate(&self, secret: &str) -> Option<Payload> {
        let mut hmac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).ok()?;

        hmac.update(format!("{}.{}", self.t, self.body).as_bytes());

        let digest = hex::encode(hmac.finalize().into_bytes());

        if digest == self.signature && let Ok(payload) = serde_json::from_str(&self.body) {
          Some(payload)
        } else {
          None
        }
      }

      /// Retrieves the payload's `x-topgg-trace` header for debugging and correlating requests with Top.gg support.
      #[must_use]
      pub fn get_trace(&self) -> &str {
        &self.trace
      }
    }
  }
}
