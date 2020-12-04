use serde::Deserialize;

use super::Snowflake;

#[derive(Deserialize)]
pub struct User {
    /// ID
    pub id: Snowflake,

    /// Username
    pub username: String,

    /// Descriminator
    pub discriminator: String,

    /// Avatar hash
    pub avatar: Option<String>,

    /// CDN hash of the user's avatar if the user has none
    #[serde(rename(deserialize = "defAvatar"))]
    pub default_avatar: String,

    /// Biography
    pub biography: Option<String>,

    /// Banner image URL
    pub banner_image: Option<String>,

    /// Social usernames
    pub social: UserSocial,

    /// Custom hex color
    pub color: Option<String>,

    /// Supporter status
    pub supporter: bool,

    /// Certified developer status
    #[serde(rename(deserialize = "certifiedDev"))]
    pub certified_dev: bool,

    /// Moderator status
    #[serde(rename(deserialize = "mod"))]
    pub moderator: bool,

    /// Website moderator status
    #[serde(rename(deserialize = "webMod"))]
    pub web_moderator: bool,

    /// Admin status
    pub admin: bool,
}

#[derive(Deserialize)]
pub struct UserSocial {
    /// Youtube channel ID
    pub youtube: Option<String>,

    /// Reddit username
    pub reddit: Option<String>,

    /// Twitter username
    pub twitter: Option<String>,

    /// Instagram username
    pub instagram: Option<String>,

    /// Github username
    pub github: Option<String>,
}
