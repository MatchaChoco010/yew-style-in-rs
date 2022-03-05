use yew::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleId(String);
impl StyleId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }

    fn id(&self) -> &str {
        &self.0
    }
}
impl Into<Classes> for StyleId {
    fn into(self) -> Classes {
        classes!(self.id().to_string())
    }
}
