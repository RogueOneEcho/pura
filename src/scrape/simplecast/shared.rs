use crate::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct SimplecastSite {
    pub subdomain: String,
    pub external_website: Url,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct SimplecastAuthors {
    pub collection: Vec<SimplecastAuthor>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct SimplecastAuthor {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct SimplecastCount {
    pub count: u32,
}
