use yew::prelude::*;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct StyleId(String);
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

#[derive(Clone, PartialEq)]
pub struct StyleContent {
    style_id: StyleId,
    code: String,
    count: usize,
}
impl StyleContent {
    pub fn new(style_id: StyleId, code: String) -> Self {
        Self {
            style_id,
            code,
            count: 0,
        }
    }

    pub fn style_id(&self) -> StyleId {
        self.style_id.clone()
    }

    pub fn code(&self) -> String {
        self.code.to_owned()
    }

    pub fn increment(&mut self) {
        self.count += 1;
    }

    pub fn decrement(&mut self) -> bool {
        self.count -= 1;
        self.count == 0
    }
}
