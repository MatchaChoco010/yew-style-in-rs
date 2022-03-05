use std::io::Write;

use anyhow::Result;
use parcel_css::stylesheet::{ParserOptions, StyleSheet};
use proc_macro2::TokenStream;
use quote::quote;

use super::state::STATE;

fn create_partial_css(filename: &str, css: &str) -> Result<String> {
    let mut state = STATE.lock().unwrap();
    let (id, mut file) = state.create_random_id_file()?;
    file.write(format!("{filename}\n").as_bytes())?;
    let css = format!(".{id} {{\n{css}\n}}");
    file.write(css.as_bytes())?;
    Ok(id)
}

enum CssItem {
    CodeOnly { code: String },
    CodeAndFileName { code: String, filename: String },
}
impl syn::parse::Parse for CssItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let option = ParserOptions {
            nesting: true,
            custom_media: false,
            css_modules: false,
            source_index: 0,
        };

        let lit_str_1 = input.parse::<syn::LitStr>()?;
        if let Ok(_comma) = input.parse::<syn::Token![,]>() {
            match input.parse::<syn::LitStr>() {
                Ok(lit_str_2) => {
                    if input.is_empty() {
                        let filename = lit_str_1.value();
                        let code = lit_str_2.value();
                        let dummy_code = format!(".dummy {{\n{code}\n}}");

                        StyleSheet::parse(filename.clone(), &dummy_code, option).map_err(|e| {
                            if let Some(loc) = e.loc {
                                syn::Error::new(
                                    lit_str_2.span(),
                                    format!(
                                        "CSS parse error: line {}, column {}",
                                        loc.line, loc.column
                                    ),
                                )
                            } else {
                                syn::Error::new(lit_str_2.span(), "CSS parse error")
                            }
                        })?;

                        Ok(CssItem::CodeAndFileName { filename, code })
                    } else {
                        Err(syn::Error::new(input.span(), "parse error"))
                    }
                }
                Err(e) => Err(e),
            }
        } else {
            if input.is_empty() {
                let code = lit_str_1.value();
                let dummy_code = format!(".dummy {{\n{code}\n}}");

                StyleSheet::parse("style.css".to_string(), &dummy_code, option).map_err(|e| {
                    if let Some(loc) = e.loc {
                        syn::Error::new_spanned(
                            lit_str_1,
                            format!("CSS parse error: line {}, column {}", loc.line, loc.column),
                        )
                    } else {
                        syn::Error::new(lit_str_1.span(), "CSS parse error")
                    }
                })?;

                Ok(CssItem::CodeOnly { code })
            } else {
                Err(syn::Error::new(input.span(), "parse error"))
            }
        }
    }
}

pub fn css(token: TokenStream) -> TokenStream {
    match syn::parse2::<CssItem>(token.clone()) {
        Err(err) => err.to_compile_error(),
        Ok(item) => match item {
            CssItem::CodeOnly { code } => match create_partial_css("style", &code) {
                Ok(id) => quote!(::yew_style_in_rs::static_style::style::StyleId::new(#id)),
                Err(_) => quote!(compile_error!(
                    "Failed to save internal file for yew-style-in-rs"
                )),
            },
            CssItem::CodeAndFileName { code, filename } => {
                match create_partial_css(&filename, &code) {
                    Ok(id) => quote!(::yew_style_in_rs::static_style::style::StyleId::new(#id)),
                    Err(_) => quote!(compile_error!(
                        "Failed to save internal file for yew-style-in-rs"
                    )),
                }
            }
        },
    }
}
