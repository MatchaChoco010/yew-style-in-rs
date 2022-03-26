use self::keyframes::RegisteredAnimationName;
use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};
use std::collections::HashSet;

mod css;
mod dyn_css;
mod keyframes;

mod kw {
    syn::custom_keyword!(filename);
}

// --- CSS Declaration ---

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

// Parse css! macro declaration.
//
// eg)
// css!(filename = "filename")
//
// eg)
// css! {" some style... "}
enum CssMacro {
    Filename(Filename),
    CssMacro(css::Css),
}
impl syn::parse::Parse for CssMacro {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(kw::filename) {
            Ok(Self::Filename(input.parse()?))
        } else {
            Ok(Self::CssMacro(input.parse()?))
        }
    }
}

// Parse CSS declaration both dyn or static.
//
// eg)
// let <ident> = css! {" some style... "};
//
// eg)
// let <ident> = css!(filename = "filename") {" some style... "};
//
// eg)
// let <ident> = dyn css! {" some style... "};
enum CssDeclaration {
    Css {
        ident: syn::Ident,
        filename: Option<syn::LitStr>,
        css: css::Css,
    },
    DynCss {
        ident: syn::Ident,
        dyn_css: dyn_css::DynCss,
    },
}
impl CssDeclaration {
    fn expand(&self, animation_names: &Vec<RegisteredAnimationName>) -> TokenStream {
        let mut tokens = TokenStream::new();
        match self {
            Self::Css {
                ident,
                filename,
                css,
            } => {
                let css = css.clone().expand(filename, animation_names);
                tokens.append_all(quote! (let #ident = #css;))
            }
            Self::DynCss { ident, dyn_css } => {
                let dyn_css = dyn_css.expand(animation_names);
                tokens.append_all(quote!(let #ident = #dyn_css;))
            }
        }
        tokens
    }
}
impl syn::parse::Parse for CssDeclaration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<syn::Token![let]>()?;
        let ident: syn::Ident = input.parse()?;
        input.parse::<syn::Token![=]>()?;

        if input.peek(syn::Token![dyn]) {
            input.parse::<syn::Token![dyn]>()?;
            let dyn_css_macro: syn::Macro = input.parse()?;
            let dyn_css: dyn_css::DynCss = dyn_css_macro.parse_body()?;
            input.parse::<syn::Token![;]>()?;
            Ok(Self::DynCss { ident, dyn_css })
        } else {
            let css_macro: syn::Macro = input.parse()?;
            let css_macro: CssMacro = css_macro.parse_body()?;
            let (filename, css) = match css_macro {
                CssMacro::Filename(filename) => {
                    let filename = Some(filename.filename);
                    let css;
                    syn::braced!(css in input);
                    let css: css::Css = css.parse()?;
                    (filename, css)
                }
                CssMacro::CssMacro(css) => (None, css),
            };
            input.parse::<syn::Token![;]>()?;
            Ok(Self::Css {
                ident,
                filename,
                css,
            })
        }
    }
}

// --- Style ---

// Parse Style Item.
// Currently only CSS declarations.
enum StyleItem {
    CssDeclaration(CssDeclaration),
    Keyframes(keyframes::Keyframes),
}
impl syn::parse::Parse for StyleItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Token![let]) {
            Ok(Self::CssDeclaration(input.parse()?))
        } else if input.peek(syn::Ident) && input.peek2(syn::Token![!]) {
            Ok(Self::Keyframes(input.parse()?))
        } else {
            Err(input.error("expected css declarations."))
        }
    }
}

// A body of the `style!` macro.
pub struct Style {
    items: Vec<StyleItem>,
}
impl syn::parse::Parse for Style {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut items = vec![];
        while !input.is_empty() {
            items.push(input.parse()?);
        }
        Ok(Self { items })
    }
}
impl Style {
    pub fn expand(&self) -> TokenStream {
        let mut token_stream = TokenStream::new();

        let mut css_declarations = vec![];
        let mut animation_names = vec![];

        for item in &self.items {
            match item {
                StyleItem::CssDeclaration(declaration) => css_declarations.push(declaration),
                StyleItem::Keyframes(keyframes) => match keyframes.register() {
                    Ok(mut names) => animation_names.append(&mut names),
                    Err(msg) => return quote!(std::compile_error!(#msg)),
                },
            }
        }

        // check duplicate animation name
        {
            let mut set = HashSet::new();
            for name in &animation_names {
                if let Some(_) = set.get(&name.animation_name) {
                    return quote!(std::compile_error!("Duplicate animation name"));
                } else {
                    set.insert(name.animation_name.to_owned());
                }
            }
        }

        for declaration in css_declarations {
            let item = declaration.expand(&animation_names);
            token_stream.append_all(quote!(#item));
        }
        token_stream
    }
}
