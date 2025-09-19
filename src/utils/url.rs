use url::Url;

pub trait UrlExtensions {
    fn get_extension(&self) -> Option<String>;
}

impl UrlExtensions for Url {
    fn get_extension(&self) -> Option<String> {
        let path = self.path();
        path.rsplit('.').next().map(ToOwned::to_owned)
    }
}
