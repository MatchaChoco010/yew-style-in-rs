use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::{cell::RefCell, iter::repeat_with};

use once_cell::unsync::Lazy;
use parcel_css::stylesheet::{ParserOptions, PrinterOptions, StyleSheet};
use parcel_css::targets::Browsers;

use crate::dynamic_style::style::{StyleContent, StyleId};

struct ManagedStyleId {
    count: u32,
    style_id: StyleId,
}
impl ManagedStyleId {
    fn new(id: StyleId) -> Self {
        Self {
            count: 0,
            style_id: id,
        }
    }

    fn increment(&mut self) -> &Self {
        self.count += 1;
        self
    }

    fn decrement(&mut self) -> &Self {
        self.count -= 1;
        self
    }
}

struct StyleManagerInner {
    managed_map: HashMap<String, ManagedStyleId>,
}

#[derive(Clone)]
pub struct StyleManager {
    inner: Rc<RefCell<StyleManagerInner>>,
}
impl StyleManager {
    pub fn register(&self, content: StyleContent) -> StyleId {
        let mut inner = self.inner.borrow_mut();
        let already_exists_ids = inner
            .managed_map
            .values()
            .map(|id| id.style_id.clone())
            .collect::<HashSet<StyleId>>();
        let entry = inner.managed_map.entry(content.content.to_owned());
        let managed_id = entry.or_insert_with(|| loop {
            let id = repeat_with(fastrand::alphabetic)
                .take(8)
                .collect::<String>();
            let id = StyleId(format!("dynamic-{id}"));
            if already_exists_ids.contains(&id) {
                continue;
            }

            let code = format!(".{} {{{}}}", id.id(), content.content);
            let parser_options = ParserOptions {
                nesting: true,
                custom_media: false,
                css_modules: false,
                source_index: 0,
            };
            let printer_options = PrinterOptions {
                minify: false,
                source_map: None,
                targets: Some(Browsers::default()),
                analyze_dependencies: false,
                pseudo_classes: None,
            };
            let css = StyleSheet::parse(id.id().to_owned(), &code, parser_options)
                .ok()
                .and_then(|ss| ss.to_css(printer_options).ok())
                .and_then(|css| Some(css.code))
                .unwrap_or_else(|| code);

            let document = web_sys::window()
                .and_then(|w| w.document())
                .unwrap_or_else(|| panic!("Failed to get document"));
            let head = document
                .head()
                .unwrap_or_else(|| panic!("Failed to get document head"));
            let style_element = document
                .create_element("style")
                .unwrap_or_else(|_| panic!("Failed to create style element"));
            style_element
                .set_attribute("data-style", id.id())
                .unwrap_or_else(|_| panic!("Failed to set style attribute"));
            style_element.set_text_content(Some(&css));
            head.append_child(&style_element)
                .unwrap_or_else(|_| panic!("Failed to mount style element"));

            break ManagedStyleId::new(id);
        });
        managed_id.increment();
        managed_id.style_id.clone()
    }

    pub fn unregister(&self, content: StyleContent) {
        let mut inner = self.inner.borrow_mut();
        if let Some(managed_id) = inner.managed_map.get_mut(&content.content) {
            managed_id.decrement();
            if managed_id.count == 0 {
                let document = web_sys::window()
                    .and_then(|w| w.document())
                    .unwrap_or_else(|| panic!("Failed to get document"));
                if let Some(style) = document
                    .query_selector(&format!("style[data-style={}]", managed_id.style_id.id()))
                    .unwrap_or_else(|_| panic!("Failed to query selector"))
                {
                    if let Some(parent) = style.parent_element() {
                        parent
                            .remove_child(&style)
                            .unwrap_or_else(|_| panic!("Failed to remove style"));
                    }
                }

                inner.managed_map.remove(&content.content);
            }
        }
    }
}
impl Default for StyleManager {
    fn default() -> Self {
        thread_local! {
            static MANAGER: Lazy<StyleManager> = Lazy::new(|| StyleManager { inner: Rc::new(RefCell::new(StyleManagerInner { managed_map: HashMap::new()}))});
        }
        MANAGER.with(|m| (*m).clone())
    }
}
