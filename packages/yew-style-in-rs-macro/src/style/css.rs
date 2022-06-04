use proc_macro2::TokenStream;
use quote::quote;

use crate::style::keyframes::*;

use crate::state::*;

// replace animation name to animation name with id
fn replace_animation_name(
    code: String,
    animation_names: &Vec<RegisteredAnimationName>,
) -> Result<String, String> {
    use crate::cursor::*;

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
        use std::io::Write;
        use yew_style_in_rs_core::ast::RuntimeCss;
        use yew_style_in_rs_core::transpiler::TranspiledCss;

        let mut state = STATE.lock().unwrap();

        let id = if state.write_flag {
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

            id
        } else {
            "dummy".into()
        };

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
