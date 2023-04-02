pub(crate) fn get_avatar(hash: &Option<String>, discriminator: &str, id: u64) -> String {
  match hash {
    Some(hash) => {
      let ext = if hash.starts_with("a_") { "gif" } else { "png" };

      format!("https://cdn.discordapp.com/avatars/{id}/{hash}.{ext}?size=1024")
    }

    None => format!("https://cdn.discordapp.com/embed/avatars/{}.png", unsafe {
      discriminator.parse::<u16>().unwrap_unchecked() % 5
    }),
  }
}
