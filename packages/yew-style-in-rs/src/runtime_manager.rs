use once_cell::unsync::Lazy;
use std::collections::HashMap;
use std::rc::Rc;
use std::{cell::RefCell, iter::repeat_with};

use crate::dyn_css::{StyleContent, StyleId};
use yew_style_in_rs_core::ast::RuntimeCss;
use yew_style_in_rs_core::transpiler::TranspiledCss;

struct StyleManagerInner {
    managed_ids: HashMap<String, StyleContent>,
}

// StyleManager is intended to be used as a singleton.
// Singleton instances are accessed via `default()`.
//
// In register, the same random id is generated for strings of
// the same code and a common style element is used.
// In unregister, the style element is deleted when the last style of the same code disappears.
#[derive(Clone)]
pub struct StyleManager {
    inner: Rc<RefCell<StyleManagerInner>>,
}
impl StyleManager {
    pub fn register(&self, code: String) -> StyleContent {
        let mut inner = self.inner.borrow_mut();
        let already_exists_ids = inner
            .managed_ids
            .values()
            .map(|content| content.style_id())
            .collect::<Vec<_>>();
        let managed_content = inner
            .managed_ids
            .entry(code.to_owned())
            .or_insert_with(|| loop {
                let id = repeat_with(fastrand::alphabetic)
                    .take(8)
                    .collect::<String>();
                let style_id = StyleId::new(&format!("dynamic-{id}"));
                if already_exists_ids.contains(&style_id) {
                    continue;
                }

                let css = match RuntimeCss::parse(&code) {
                    Ok(css) => css,
                    Err((css, _)) => css,
                };
                let css = TranspiledCss::transpile(&[format!(".{}", style_id.id())], css);
                let css_code = css.to_style_string();

                let document = gloo::utils::document();
                let head = gloo::utils::head();
                let style_element = document
                    .create_element("style")
                    .unwrap_or_else(|_| panic!("Failed to create style element"));
                style_element
                    .set_attribute("data-style", style_id.id())
                    .unwrap_or_else(|_| panic!("Failed to set style attribute"));
                style_element.set_text_content(Some(&css_code));
                head.append_child(&style_element)
                    .unwrap_or_else(|_| panic!("Failed to mount style element"));

                break StyleContent::new(style_id, code);
            });

        managed_content.increment();
        managed_content.clone()
    }

    pub fn unregister(&self, content: StyleContent) {
        let mut inner = self.inner.borrow_mut();
        let managed_ids = &mut inner.managed_ids;
        let document = gloo::utils::document();
        let code = content.code();
        if let Some(content) = managed_ids.get_mut(&code) {
            if content.decrement() {
                if let Some(style) = document
                    .query_selector(&format!("style[data-style={}]", content.style_id().id()))
                    .unwrap_or_else(|_| panic!("Failed to query selector"))
                {
                    if let Some(parent) = style.parent_element() {
                        parent
                            .remove_child(&style)
                            .unwrap_or_else(|_| panic!("Failed to remove style"));
                    }
                }
                managed_ids.remove(&code);
            }
        }
    }
}
impl Default for StyleManager {
    fn default() -> Self {
        thread_local! {
            static MANAGER: Lazy<StyleManager> = Lazy::new(|| StyleManager { inner: Rc::new(RefCell::new(StyleManagerInner { managed_ids: HashMap::new()}))});
        }
        MANAGER.with(|m| (*m).clone())
    }
}
