use serde::{Deserialize, Deserializer};

#[inline(always)]
pub(crate) fn deserialize_deprecated<'de, D, T>(_deserializer: D) -> Result<T, D::Error>
where
  T: Default + Deserialize<'de>,
  D: Deserializer<'de>,
{
  Ok(T::default())
}

cfg_if::cfg_if! {
  if #[cfg(feature = "api")] {
    use crate::{snowflake, Error};
    use base64::Engine;
    use chrono::{DateTime, TimeZone, Utc};
    use reqwest::Response;
    use serde::de::DeserializeOwned;

    const DISCORD_EPOCH: u64 = 1_420_070_400_000;

    macro_rules! debug_struct {
      (
        $(#[$struct_attr:meta])*
        $struct_name:ident {
          $(public {
            $(
              $(#[$pub_prop_attr:meta])*
              $pub_prop_name:ident: $pub_prop_type:ty,
            )*
          })?
          $(protected {
            $(
              $(#[$protected_prop_attr:meta])*
              $protected_prop_name:ident: $protected_prop_type:ty,
            )*
          })?
          $(private {
            $(
              $(#[$priv_prop_attr:meta])*
              $priv_prop_name:ident: $priv_prop_type:ty,
            )*
          })?
          $(getters($self:ident) {
            $(
              $(#[$getter_attr:meta])*
              $getter_name:ident: $getter_type:ty => $getter_code:tt
            )*
          })?
        }
      ) => {
        #[allow(deprecated)]
        $(#[$struct_attr])*
        pub struct $struct_name {
          $($(
            $(#[$pub_prop_attr])*
            pub $pub_prop_name: $pub_prop_type,
          )*)?
          $($(
            $(#[$protected_prop_attr])*
            pub(crate) $protected_prop_name: $protected_prop_type,
          )*)?
          $($(
            $(#[$priv_prop_attr])*
            $priv_prop_name: $priv_prop_type,
          )*)?
        }
    
        $(
          #[allow(deprecated)]
          impl $struct_name {
            $(
              $(#[$getter_attr])*
              pub fn $getter_name(&$self) -> $getter_type $getter_code
            )*
          }
        )?
    
        #[allow(deprecated)]
        impl std::fmt::Debug for $struct_name {
          fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            fmt
              .debug_struct(stringify!($struct_name))
              $($(
                .field(stringify!($pub_prop_name), &self.$pub_prop_name)
              )*)?
              $($(
                .field(stringify!($getter_name), &self.$getter_name())
              )*)?
              .finish()
          }
        }
      };
    }
    
    pub(crate) use debug_struct;
    
    #[inline(always)]
    pub(crate) fn deserialize_optional_string<'de, D>(
      deserializer: D,
    ) -> Result<Option<String>, D::Error>
    where
      D: Deserializer<'de>,
    {
      Ok(
        String::deserialize(deserializer)
          .ok()
          .filter(|s| !s.is_empty()),
      )
    }
    
    #[inline(always)]
    pub(crate) fn deserialize_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
      T: Default + Deserialize<'de>,
      D: Deserializer<'de>,
    {
      Option::deserialize(deserializer).map(Option::unwrap_or_default)
    }
    
    #[inline(always)]
    pub(crate) fn get_creation_date(id: u64) -> DateTime<Utc> {
      Utc
        .timestamp_millis_opt(((id >> 22) + DISCORD_EPOCH) as _)
        .single()
        .unwrap()
    }
    
    #[inline(always)]
    pub(crate) async fn parse_json<T>(response: Response) -> crate::Result<T>
    where
      T: DeserializeOwned,
    {
      if let Ok(bytes) = response.bytes().await {
        if let Ok(json) = serde_json::from_slice(&bytes) {
          return Ok(json);
        }
      }
    
      Err(Error::InternalServerError)
    }
    
    #[derive(Deserialize)]
    #[allow(clippy::used_underscore_binding)]
    struct TokenStructure {
      #[serde(deserialize_with = "snowflake::deserialize")]
      id: u64,
    }
    
    pub(crate) fn parse_api_token(token: &str) -> u64 {
      if let Some(base64_section) = token.split('.').nth(1) {
        if let Ok(decoded_base64) =
          base64::engine::general_purpose::STANDARD_NO_PAD.decode(base64_section)
        {
          if let Ok(token_structure) = serde_json::from_slice::<TokenStructure>(&decoded_base64) {
            return token_structure.id;
          }
        }
      }
    
      panic!("Got a malformed API token.");
    }
  }
}