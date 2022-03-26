use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};
use std::iter::Peekable;
use std::str::Chars;

use crate::style::keyframes::*;

// Parser for `${<ident>}` in code.
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

    fn take_brace(&mut self) -> Option<String> {
        self.take('{')?;
        if let Some(content) = self.take_until('}') {
            self.take('}');
            Some(content)
        } else {
            None
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

// When `parse()`, inspect code and replace `{` with `{{`, `}` with `}}`, `${ident}` with `{ident}`
// and collect idents to use when expanding macro.
// When `expand()`, generate code with idents using `format!` macro
// and using `use_effect_with_deps` to register/unregister runtime manager
// when code is change or destroy this element.
pub struct DynCss {
    code: syn::LitStr,
    idents: Vec<syn::Ident>,
}
impl DynCss {
    pub fn expand(&self, animation_names: &Vec<RegisteredAnimationName>) -> TokenStream {
        let code = self.code.value();
        let code = match replace_animation_name(code, animation_names) {
            Ok(code) => code,
            Err(msg) => return quote!(std::compile_error!(#msg)),
        };

        let dependencies = {
            let mut dependencies = TokenStream::new();
            for ident in &self.idents {
                dependencies.append_all(quote!(#ident = #ident, ));
            }
            dependencies
        };

        quote! {{
            let prev_style_handle = ::yew::use_mut_ref(|| None);
            let style_state = ::yew::use_state_eq(|| None);

            let code = format!(#code, #dependencies);

            // Unregister previous style and register new style when code is changed.
            ::yew::use_effect_with_deps(
                {
                    let style_state = style_state.clone();
                    move |code: &String| {
                        let manager = ::yew_style_in_rs::runtime_manager::StyleManager::default();
                        let style = manager.register(code.to_string());
                        if let Some(style) = prev_style_handle.borrow().clone() {
                            manager.unregister(style);
                        }
                        *prev_style_handle.borrow_mut() = Some(style.clone());
                        style_state.set(Some(style));
                        || ()
                    }
                },
                code
            );

            // Unregister style when destroy elements.
            ::yew::use_effect_with_deps(
                {
                    let style_state = style_state.clone();
                    move |_| {
                        let manager = ::yew_style_in_rs::runtime_manager::StyleManager::default();
                        let style_state = style_state.clone();
                        move || {
                            if let Some(style) = (*style_state).as_ref() {
                                manager.unregister(style.clone());
                            }
                        }
                    }
                },
                ()
            );

            // return `dyn_css::StyleId` of current style.
            // If no style, return empty `StyleId`
            (*style_state).as_ref().map_or_else(|| ::yew_style_in_rs::dyn_css::StyleId::new(""), |s| s.style_id())
        }}
    }
}
impl syn::parse::Parse for DynCss {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let raw_code: syn::LitStr = input.parse()?;
        let raw_code = raw_code.value();

        let mut cursor = Cursor::new(&raw_code);

        let mut code = String::new();
        let mut idents = vec![];
        while !cursor.is_empty() {
            if cursor.peek('$') {
                cursor.take('$');
                if cursor.peek('{') {
                    let ident = cursor.take_brace().unwrap_or_default();
                    code.push_str(&format!("{{{ident}}}"));

                    let ident = syn::Ident::new(&ident, input.span());
                    idents.push(ident);
                } else {
                    code.push('$');
                }
            } else if cursor.peek('{') {
                cursor.take('{');
                code.push_str("{{");
            } else if cursor.peek('}') {
                cursor.take('}');
                code.push_str("}}");
            } else {
                let ch = cursor.next().unwrap();
                code.push(ch);
            }
        }
        let code = syn::LitStr::new(&code, input.span());

        Ok(Self { code, idents })
    }
}
