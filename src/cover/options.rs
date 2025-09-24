use crate::prelude::*;

#[derive(Debug, Args)]
pub struct CoverOptions {
    /// ID of the downloaded podcast
    ///
    /// Must be alphanumeric and hyphenated
    #[arg(value_parser = Podcast::validate_id)]
    pub podcast_id: String,
}
