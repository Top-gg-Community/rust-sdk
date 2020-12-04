use serde::Deserialize;

#[derive(Deserialize)]
pub struct Snowflake(u64);

impl Snowflake {
    const DISCORD_EPOCH: u64 = 1_420_070_400_000;

    pub fn id(&self) -> u64 {
        self.0
    }

    /// The Unix epoch of the Snowflake in miliseconds
    #[allow(clippy::cast_possible_wrap)]
    pub fn timestamp(&self) -> i64 {
        ((self.id() >> 22) + Snowflake::DISCORD_EPOCH) as i64
    }

    /// The id of the internal worker that generated the Snowflake.
    ///
    /// Derived from bits 17..21 of the id.
    #[allow(clippy::cast_possible_truncation)]
    pub fn worker_id(&self) -> u8 {
        ((self.id() & 0x003E_0000) >> 17) as u8
    }

    /// The id of the internal process that generated the Snowflake.
    ///
    /// Derived from bits 12..16 of the id.
    #[allow(clippy::cast_possible_truncation)]
    pub fn process_id(&self) -> u8 {
        ((self.id() & 0x1F000) >> 12) as u8
    }

    /// The increment of the Snowflake. For every id that is generated on a process, this number is
    /// incremented.
    ///
    /// Derived from bits 0..11 of the id.
    #[allow(clippy::cast_possible_truncation)]
    pub fn increment(&self) -> u16 {
        (self.id() & 0xFFF) as u16
    }
}
