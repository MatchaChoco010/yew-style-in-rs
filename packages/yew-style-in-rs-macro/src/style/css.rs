use proc_macro2::TokenStream;
use quote::quote;
use std::io::Write;

use crate::state::*;
use yew_style_in_rs_core::ast::RuntimeCss;
use yew_style_in_rs_core::transpiler::TranspiledCss;

pub struct Css {
    code: syn::LitStr,
}
impl Css {
    pub fn expand(&self, filename: &Option<syn::LitStr>) -> TokenStream {
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

        match RuntimeCss::parse(&id, self.code.value()) {
            Ok(runtime_css) => {
                let transpiled_css = TranspiledCss::transpile(runtime_css);
                let css = transpiled_css.to_style_string();

                file.write(css.as_bytes())
                    .expect("Failed to save internal file for yew-style-in-rs");

                quote!({
                    use ::yew_style_in_rs::css::StyleId;
                    StyleId::new(#id)
                })
            }
            Err(_) => {
                quote!(std::compile_error!("CSS parse error"))
            }
        }
    }
}
impl syn::parse::Parse for Css {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let code: syn::LitStr = input.parse()?;
        Ok(Self { code })
    }
}
