use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use rogue_logging::Colors;

#[derive(Clone, Debug)]
pub struct Progress {
    bar: ProgressBar,
}

impl Progress {
    pub(crate) fn new(length: u64) -> Progress {
        let bar = ProgressBar::new(length);
        bar.set_draw_target(ProgressDrawTarget::stderr());
        bar.set_style(create_progress_style());
        Self { bar }
    }

    pub(crate) fn update(&self) {
        self.bar.inc(1);
    }

    pub(crate) fn finish(&self) {
        self.bar.finish();
    }
}

fn create_progress_style() -> ProgressStyle {
    let template = format!(
        "{{bar:50}} {}",
        "{pos:>4}/{len} {elapsed:>3} elapsed, {eta} remain".gray()
    )
    .to_string();
    ProgressStyle::default_bar()
        .template(&template)
        .expect("Progress style should compile")
}
