use yew::prelude::*;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct StyleId(pub String);
impl StyleId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }

    pub(crate) fn id(&self) -> &str {
        &self.0
    }
}
impl Into<Classes> for StyleId {
    fn into(self) -> Classes {
        classes!(self.id().to_string())
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct StyleContent {
    pub content: String,
}
impl StyleContent {
    pub fn new(code: &str) -> Self {
        Self {
            content: code.to_owned(),
        }
    }
}
