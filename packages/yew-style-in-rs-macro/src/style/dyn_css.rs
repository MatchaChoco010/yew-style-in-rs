use proc_macro2::{Span, TokenStream};
use quote::{quote, TokenStreamExt};
use std::collections::HashSet;

use crate::cursor::*;
use crate::style::keyframes::*;

// replace animation name to animation name with id
fn replace_animation_name(
    code: String,
    animation_names: &Vec<RegisteredAnimationName>,
    dyn_animation_names: &Vec<String>,
) -> Result<String, String> {
    let mut cursor = Cursor::new(&code);
    let mut code = String::new();

    while !cursor.is_empty() {
        if cursor.peek('#') {
            cursor.take('#').unwrap();
            if cursor.peek('#') {
                cursor.take('#').unwrap();
                let (name, _) = cursor
                    .take_until(&['#'])
                    .map_err(|_| "`##<animation_name>##` is expected")?;

                if name.is_empty() {
                    return Err(format!("animation name is empty"));
                }
                if name.chars().any(|c| c.is_whitespace()) {
                    return Err(format!("animation name  can not contain whitespace"));
                }

                if let Some(name) = animation_names.iter().find(|n| n.animation_name == name) {
                    code += &name.animation_name_with_scoped_id;
                } else if dyn_animation_names.contains(&name) {
                    code += "##";
                    code += &name;
                    code += "##";
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
    pub fn expand(
        &self,
        animation_names: &Vec<RegisteredAnimationName>,
        dyn_animation_names: &Vec<String>,
    ) -> TokenStream {
        let code = self.code.value();
        let code = match replace_animation_name(code, animation_names, dyn_animation_names) {
            Ok(code) => code,
            Err(msg) => return quote!(std::compile_error!(#msg)),
        };

        let dependencies = {
            let mut dependencies = TokenStream::new();
            let mut hashset = HashSet::new();
            for ident in &self.idents {
                hashset.insert(ident.to_string());
            }
            for ident in hashset.iter() {
                let ident = syn::Ident::new(ident, Span::call_site());
                dependencies.append_all(quote!(#ident = #ident, ));
            }
            dependencies
        };
        let animation_names_vec = {
            let mut tokens = TokenStream::new();
            for name in animation_names {
                let name = name.animation_name.clone();
                tokens.append_all(quote!((#name).to_string(), ));
            }
            quote!(vec![#tokens])
        };

        quote! {{
            let prev_style_handle = ::yew::use_mut_ref(|| None);
            let style_state = ::yew::use_state_eq(|| None);

            let animation_names: Vec<String> = #animation_names_vec;

            let code = format!(#code, #dependencies);

            // Unregister previous style and register new style when code is changed.
            ::yew::use_effect_with_deps(
                {
                    let style_state = style_state.clone();
                    move |(code, dyn_names_map): &(String, std::collections::HashMap<String, Vec<String>>)| {
                        let manager = ::yew_style_in_rs::runtime_manager::StyleManager::default();

                        let mut cursor = ::yew_style_in_rs::cursor::Cursor::new(code);
                        let mut code = String::new();
                        while !cursor.is_empty() {
                            if cursor.peek('#') {
                                cursor.take('#').unwrap();
                                if cursor.peek('#') {
                                    cursor.take('#').unwrap();
                                    let name = match cursor.take_until('#') {
                                        Ok(name) => name,
                                        Err(_) => break,
                                    };

                                    if let Some(name) = animation_names.iter().find(|&n| n == &name) {
                                        code += &name;
                                    } else {
                                        if let Some((id, _)) = dyn_names_map.iter().find(|(_, n)| n.contains(&name)) {
                                            code += &name;
                                            code += "-";
                                            code += id;
                                        } else {
                                            code += "##";
                                            code += &name;
                                            code += "##";
                                        }
                                    }

                                    cursor.take('#').unwrap();
                                    if cursor.peek('#') {
                                        cursor.take('#').unwrap();
                                    }
                                } else {
                                    code.push('#');
                                }
                            } else {
                                let ch = cursor.next().unwrap();
                                code.push(ch);
                            }
                        }

                        let style = manager.register(code);
                        if let Some(style) = prev_style_handle.borrow().clone() {
                            manager.unregister(style);
                        }
                        *prev_style_handle.borrow_mut() = Some(style.clone());
                        style_state.set(Some(style));
                        || ()
                    }
                },
                (code, dyn_names_map.clone())
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
