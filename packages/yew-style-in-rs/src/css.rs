use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct StyleId(&'static str);
impl StyleId {
    pub fn new(id: &'static str) -> Self {
        Self(id)
    }

    fn id(&self) -> &'static str {
        &self.0
    }
}
impl Into<Classes> for StyleId {
    fn into(self) -> Classes {
        classes!(self.id())
    }
}
