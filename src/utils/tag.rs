use crate::prelude::*;
use lofty::config::WriteOptions;
use lofty::error::LoftyError;
use lofty::id3::v2::Id3v2Tag;
use lofty::picture::Picture;
use lofty::prelude::{Accessor, TagExt};
use lofty::tag::TagType;

pub(crate) struct Tag;

impl Tag {
    pub(crate) fn execute(
        podcast: &Podcast,
        episode: &Episode,
        cover: Option<Picture>,
        path: &PathBuf,
    ) -> Result<(), LoftyError> {
        let tag = Tag::create(podcast, episode, cover);
        Tag::save(path, tag)
    }

    #[allow(
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    fn create(podcast: &Podcast, episode: &Episode, cover: Option<Picture>) -> Id3v2Tag {
        let mut tag = Id3v2Tag::default();
        tag.set_title(episode.title.trim().to_owned());
        tag.set_artist(podcast.title.clone());
        if let Some(season) = episode.season {
            tag.set_album(format!("Season {season}"));
        }
        tag.set_disk(episode.season.unwrap_or_default() as u32);
        let year = episode.published_at.year() as u32;
        tag.set_year(year);
        if let Some(number) = episode.number {
            tag.set_track(number as u32);
        }
        tag.set_comment(episode.description.clone());
        if let Some(cover) = cover {
            tag.insert_picture(cover);
        }
        tag
    }

    fn save(path: &PathBuf, tag: Id3v2Tag) -> Result<(), LoftyError> {
        TagType::Ape.remove_from_path(path)?;
        TagType::Id3v1.remove_from_path(path)?;
        TagType::Id3v2.remove_from_path(path)?;
        tag.save_to_path(path, WriteOptions::default())?;
        Ok(())
    }
}
