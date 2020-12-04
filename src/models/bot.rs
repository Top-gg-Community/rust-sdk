use serde::Deserialize;

use super::Snowflake;

#[derive(Deserialize)]
pub struct Bot {
    /// ID
    pub id: Snowflake,

    pub username: String,

    pub discriminator: String,

    /// Avatar hash of the bot's avatar
    pub avatar: Option<String>,

    /// CDNN hash of the bot's avatar if the bot has none
    #[serde(rename(deserialize = "defAvatar"))]
    pub default_avatar: String,

    /// Library of the bot
    pub lib: String,

    pub prefix: String,

    #[serde(rename(deserialize = "shortdesc"))]
    pub short_description: String,

    #[serde(rename(deserialize = "longdesc"))]
    pub long_description: Option<String>,

    pub tags: Vec<String>,

    pub website: Option<String>,

    /// Support server invite code
    pub support: Option<String>,

    /// Github repo of the bot
    pub github: Option<String>,

    /// Owners of the bot.
    /// First one in the array is the main owner
    pub owners: Vec<Snowflake>,

    /// Guilds featured on the bot page
    pub guilds: Vec<Snowflake>,

    /// Custom invite URL of the bot
    pub invite: Option<String>,

    /// Date when the bot was approved.
    pub date: String,

    /// Certified status of the bot
    #[serde(rename(deserialize = "certifiedBot"))]
    pub certified_bot: bool,

    /// Vanity URL
    pub vanity: Option<String>,

    /// Amount of upvotes
    pub points: u64,

    /// Amount of upvotes this month
    #[serde(rename(deserialize = "monthlyPoints"))]
    pub monthly_points: u64,

    /// Guild ID for the donatebot setup
    #[serde(rename(deserialize = "donatebotguildid"))]
    pub donate_bot_guild_id: Snowflake,
}
