use proc_macro2::TokenStream;
use quote::quote;
use std::io::Write;
use std::iter::Peekable;
use std::str::Chars;
use yew_style_in_rs_core::ast::RuntimeCss;
use yew_style_in_rs_core::transpiler::TranspiledCss;

use crate::state::*;
use crate::style::keyframes::*;

// Parser for `##anim_name##` in code.
#[derive(Clone, Debug)]
pub struct Cursor<'a> {
    content: Peekable<Chars<'a>>,
}
impl<'a> Cursor<'a> {
    fn new(content: &'a str) -> Self {
        Self {
            content: content.chars().peekable(),
        }
    }

    fn is_empty(&mut self) -> bool {
        self.content.peek().is_none()
    }

    fn peek(&mut self, ch: char) -> bool {
        self.content.peek() == Some(&ch)
    }

    fn next(&mut self) -> Option<char> {
        self.content.next()
    }

    fn take(&mut self, ch: char) -> Option<char> {
        if let Some(c) = self.content.next() {
            if c == ch {
                Some(ch)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn take_until(&mut self, delimiter: char) -> Option<String> {
        let mut ret = String::new();
        loop {
            if let Some(&c) = self.content.peek() {
                if c == delimiter {
                    return Some(ret);
                } else {
                    ret.push(c);
                    self.content.next();
                }
            } else {
                return None;
            }
        }
    }
}

// replace animation name to animation name with id
fn replace_animation_name(
    code: String,
    animation_names: &Vec<RegisteredAnimationName>,
) -> Result<String, String> {
    let mut cursor = Cursor::new(&code);
    let mut code = String::new();

    while !cursor.is_empty() {
        if cursor.peek('#') {
            cursor.take('#').unwrap();
            if cursor.peek('#') {
                cursor.take('#').unwrap();
                let name = cursor
                    .take_until('#')
                    .ok_or("`##<animation_name>##` is expected")?;

                if let Some(name) = animation_names.iter().find(|n| n.animation_name == name) {
                    code += &name.animation_name_with_scoped_id;
                } else {
                    return Err(format!(
                        "animation name is not defined in `keyframe!` declaration: `##{name}##`"
                    ));
                }

                cursor
                    .take('#')
                    .ok_or("`##<animation_name>##` is expected")?;
                cursor
                    .take('#')
                    .ok_or("`##<animation_name>##` is expected")?;
            } else {
                code.push('#');
            }
        } else {
            let ch = cursor.next().unwrap();
            code.push(ch);
        }
    }

    Ok(code)
}

// Parse declaration when `parse()`.
// Transpile CSS nesting and write CSS fragment when `expand()`.
#[derive(Clone)]
pub struct Css {
    code: syn::LitStr,
}
impl Css {
    pub fn expand(
        self,
        filename: &Option<syn::LitStr>,
        animation_names: &Vec<RegisteredAnimationName>,
    ) -> TokenStream {
        let mut state = STATE.lock().unwrap();
        let (id, mut file) = state
            .create_random_id_file()
            .expect("Failed to save internal file for yew-style-in-rs");

        let filename = filename
            .as_ref()
            .map(|l| l.value())
            .unwrap_or("style".into());
        file.write(format!("{filename}\n").as_bytes())
            .expect("Failed to save internal file for yew-style-in-rs");

        let code = self.code.value();
        let code = match replace_animation_name(code, animation_names) {
            Ok(code) => code,
            Err(msg) => return quote!(std::compile_error!(#msg)),
        };

        let runtime_css = match RuntimeCss::parse(code) {
            Ok(runtime_css) => runtime_css,
            Err((_, msg)) => return quote!(std::compile_error!(#msg)),
        };
        let transpiled_css = TranspiledCss::transpile(&[format!(".{id}")], runtime_css);
        let css = transpiled_css.to_style_string();

        file.write(css.as_bytes())
            .expect("Failed to save internal file for yew-style-in-rs");

        quote!({
            use ::yew_style_in_rs::css::StyleId;
            StyleId::new(#id)
        })
    }
}
impl syn::parse::Parse for Css {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let code: syn::LitStr = input.parse()?;
        Ok(Self { code })
    }
}
