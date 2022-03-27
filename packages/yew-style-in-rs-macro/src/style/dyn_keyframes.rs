use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};

use crate::cursor::*;

mod kw {
    syn::custom_keyword!(filename);
}

// `parse()` dyn keyframes declaration.
//
// eg)
// dyn keyframes! {r#"
//     @keyframes anim {
//          to {
//              transform: ${translate};
//          }
//     }
// "#}
//
// When `expand()`, generate code with idents using `format!` macro
// and using `use_effect_with_deps` to register/unregister runtime manager
// when code is change or destroy this element.
// `expand()` also return animation names vec.
pub struct DynKeyframes {
    code: syn::LitStr,
}
impl DynKeyframes {
    pub fn expand(&self) -> Result<(TokenStream, Vec<String>), String> {
        let code = self.code.value();
        let mut cursor = Cursor::new(&code);
        let mut raw_code = String::new();
        let mut animation_names = vec![];

        // get fragmented code and anim names.
        // fragments is a sequence of strings that
        // does not contain anim name separated by anim name.
        cursor.skip_white_space();
        while cursor.peek('@') {
            cursor.take('@').ok_or("`@keyframes` is expected")?;
            if &cursor
                .take_until_whitespace()
                .ok_or("`@keyframes` is expected")?
                != "keyframes"
            {
                return Err("`@keyframes` is expected".into());
            }
            raw_code += "@keyframes";

            cursor.skip_white_space();
            raw_code += " ";

            let (animation_name, _) = cursor
                .take_until(&['{'])
                .map_err(|_| "animation name is expected")?;
            let animation_name = animation_name.trim_end().to_string();
            raw_code += &animation_name;
            animation_names.push(animation_name);

            cursor.skip_white_space();

            raw_code += "{";
            let content = cursor.take_brace().ok_or("`{` is expected")?;
            let mut content_cursor = Cursor::new(&content);
            content_cursor.skip_white_space();
            while let Ok((percentage, _)) = content_cursor.take_until(&['{']) {
                raw_code += percentage.trim();

                content_cursor.skip_white_space();

                raw_code += "{";
                let content = content_cursor.take_brace().ok_or("`{` is expected!")?;
                let mut content_cursor = Cursor::new(&content);
                content_cursor.skip_white_space();
                while !content_cursor.is_empty() {
                    let (property, _) = content_cursor
                        .take_until(&[':'])
                        .map_err(|_| "property is expected")?;
                    content_cursor.take(':').ok_or("`:` is expected")?;
                    content_cursor.skip_white_space();
                    let value = match content_cursor.take_until(&[';']) {
                        Ok((value, _)) => {
                            content_cursor.take(';').ok_or("`;` is expected")?;
                            value
                        }
                        Err(value) => value,
                    };
                    raw_code += property.trim_end();
                    raw_code += ":";
                    raw_code += value.trim_end();
                    raw_code += ";";
                    content_cursor.skip_white_space();
                }
                raw_code += "}";
            }
            raw_code += "}";

            cursor.skip_white_space();
        }

        let mut cursor = Cursor::new(&raw_code);
        let mut code = String::new();
        let mut idents = vec![];
        while !cursor.is_empty() {
            if cursor.peek('$') {
                cursor.take('$');
                if cursor.peek('{') {
                    let ident = cursor.take_brace().unwrap_or_default();
                    code.push_str(&format!("{{{ident}}}"));

                    let ident = syn::Ident::new(&ident, self.code.span());
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

        let dependencies = {
            let mut dependencies = TokenStream::new();
            for ident in &idents {
                dependencies.append_all(quote!(#ident = #ident, ));
            }
            dependencies
        };
        let animation_names_vec = {
            let mut tokens = TokenStream::new();
            for name in &animation_names {
                let name = name.to_string();
                tokens.append_all(quote!((#name ).to_string(), ));
            }
            quote!(vec![#tokens])
        };

        let tokens = quote! {{
            let prev_style_handle = ::yew::use_mut_ref(|| None);
            let style_state = ::yew::use_state_eq(|| None);

            let code = format!(#code, #dependencies);

            // Unregister previous style and register new style when code is changed.
            ::yew::use_effect_with_deps(
                {
                    let style_state = style_state.clone();
                    move |code: &String| {
                        let manager = ::yew_style_in_rs::runtime_manager::StyleManager::default();
                        let style = manager.register_dyn_keyframes(code.to_string());
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

            let style_id = (*style_state).as_ref().map_or_else(|| ::yew_style_in_rs::dyn_css::StyleId::new(""), |s| s.style_id());
            dyn_names_map.insert(style_id.id().to_string(), #animation_names_vec);
        }};

        Ok((tokens, animation_names))
    }
}
impl syn::parse::Parse for DynKeyframes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<syn::Token![dyn]>()?;

        let keyframes: syn::Macro = input.parse()?;

        if let Some(path) = keyframes.path.get_ident() {
            if path.to_string() != "keyframes".to_string() {
                return Err(syn::parse::Error::new(
                    path.span(),
                    "`keyframes!` is expected".to_string(),
                ));
            }
        } else {
            return Err(syn::parse::Error::new(
                keyframes.path.segments[0].ident.span(),
                "`keyframes!` is expected".to_string(),
            ));
        };

        let body: syn::LitStr = keyframes.parse_body()?;
        Ok(Self { code: body })
    }
}
