use std::io::Write;
use syn::braced;

use crate::cursor::*;
use crate::state::*;

mod kw {
    syn::custom_keyword!(filename);
}

// Parse filename setting.
//
// eg)
// filename = "filename"
struct Filename {
    filename: syn::LitStr,
}
impl syn::parse::Parse for Filename {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<kw::filename>()?;
        input.parse::<syn::Token![=]>()?;
        let filename: syn::LitStr = input.parse()?;
        Ok(Self { filename })
    }
}

// Parse keyframes! macro declaration.
//
// eg)
// keyframes!(filename = "filename")
//
// eg)
// keyframes! {" some keyframes... "}
enum MacroBody {
    Filename(Filename),
    Body(syn::LitStr),
}
impl syn::parse::Parse for MacroBody {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(kw::filename) {
            Ok(Self::Filename(input.parse()?))
        } else {
            Ok(Self::Body(input.parse()?))
        }
    }
}

// Mapping for animation_name and animation_name_with_scoped_id
pub struct RegisteredAnimationName {
    pub animation_name: String,
    pub animation_name_with_scoped_id: String,
}

// Parse keyframes declaration.
//
// eg)
// keyframes! {r#"
//     @keyframes anim {
//          to {
//              transform:translateX(200px);
//          }
//     }
// "#}
//
// eg)
// keyframes!(filename = "filename") {r#"
//     @keyframes anim {
//          to {
//              transform:translateX(200px);
//          }
//     }
// "#}
pub struct Keyframes {
    filename: Option<syn::LitStr>,
    code: syn::LitStr,
}
impl Keyframes {
    pub fn register(&self) -> Result<Vec<RegisteredAnimationName>, String> {
        let mut state = STATE.lock().unwrap();
        let (id, mut file) = state
            .create_random_id_file()
            .expect("Failed to save internal file for yew-style-in-rs");

        let code = self.code.value();
        let mut cursor = Cursor::new(&code);
        let mut code = String::new();
        let mut anim_names = vec![];

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
            code += "@keyframes";

            cursor.skip_white_space();
            code += " ";

            let animation_name = cursor
                .take_until_whitespace()
                .ok_or("animation name is expected")?;
            let animation_name_with_scoped_id = String::new() + &animation_name + "-" + &id;
            code += &animation_name_with_scoped_id;
            anim_names.push(RegisteredAnimationName {
                animation_name,
                animation_name_with_scoped_id,
            });

            cursor.skip_white_space();

            code += "{";
            let content = cursor.take_brace().ok_or("`{` is expected")?;
            let mut content_cursor = Cursor::new(&content);
            content_cursor.skip_white_space();
            while let Ok((percentage, _)) = content_cursor.take_until(&['{']) {
                code += percentage.trim();

                content_cursor.skip_white_space();

                code += "{";
                let content = content_cursor.take_brace().ok_or("`{` is expected")?;
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
                    code += property.trim_end();
                    code += ":";
                    code += value.trim_end();
                    code += ";";
                    content_cursor.skip_white_space();
                }
                code += "}";
            }
            code += "}";

            cursor.skip_white_space();
        }

        let filename = self
            .filename
            .as_ref()
            .map(|l| l.value())
            .unwrap_or("style".into());
        file.write(format!("{filename}\n").as_bytes())
            .expect("Failed to save internal file for yew-style-in-rs");

        file.write(code.as_bytes())
            .expect("Failed to save internal file for yew-style-in-rs");

        Ok(anim_names)
    }
}
impl syn::parse::Parse for Keyframes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
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

        let body: MacroBody = keyframes.parse_body()?;

        match body {
            MacroBody::Filename(filename) => {
                let content;
                braced!(content in input);
                let code: syn::LitStr = content.parse()?;
                Ok(Self {
                    filename: Some(filename.filename),
                    code,
                })
            }
            MacroBody::Body(body) => Ok(Self {
                filename: None,
                code: body,
            }),
        }
    }
}
