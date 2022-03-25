use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};

pub mod css;

mod kw {
    syn::custom_keyword!(filename);
}

// --- CSS Declaration ---

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

enum CssDeclaration {
    Css {
        ident: syn::Ident,
        filename: Option<syn::LitStr>,
        css: css::Css,
    },
}
impl syn::parse::Parse for CssDeclaration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<syn::Token![let]>()?;
        let ident: syn::Ident = input.parse()?;
        input.parse::<syn::Token![=]>()?;

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
impl ToTokens for CssDeclaration {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Css {
                ident,
                filename,
                css,
            } => {
                let css = css.expand(filename);
                tokens.append_all(quote! (let #ident = #css;))
            }
        }
    }
}

// --- Style ---

enum StyleItem {
    CssDeclaration(CssDeclaration),
}
impl syn::parse::Parse for StyleItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Token![let]) {
            Ok(Self::CssDeclaration(input.parse()?))
        } else {
            Err(input.error("expected css declarations."))
        }
    }
}
impl ToTokens for StyleItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::CssDeclaration(css_declaration) => tokens.append_all(quote!(#css_declaration)),
        }
    }
}

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
        for item in &self.items {
            token_stream.append_all(quote!(#item));
        }
        token_stream
    }
}
