use proc_macro2::TokenStream;
use quote::quote;

struct CssItem {
    code: syn::Expr,
}
impl syn::parse::Parse for CssItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expr = input.parse::<syn::Expr>()?;
        if input.is_empty() {
            Ok(CssItem { code: expr })
        } else {
            Err(syn::Error::new(input.span(), "parse error"))
        }
    }
}

pub fn css(token: TokenStream) -> TokenStream {
    match syn::parse2::<CssItem>(token.clone()) {
        Err(err) => err.to_compile_error(),
        Ok(item) => {
            let code = item.code;
            quote! {{
                let prev_style_handle = ::yew::use_mut_ref(|| None);
                let id_handle = ::yew::use_state_eq(|| None);

                let code = #code;

                ::yew::use_effect_with_deps(
                    {
                        let id_handle = id_handle.clone();
                        move |code: &String| {
                            let manager = ::yew_style_in_rs::dynamic_style::manager::StyleManager::default();

                            let style = ::yew_style_in_rs::dynamic_style::style::StyleContent::new(code);

                            if let Some(style) = prev_style_handle.borrow().clone() {
                                manager.unregister(style);
                            }
                            *prev_style_handle.borrow_mut() = Some(style.clone());
                            id_handle.set(Some(manager.register(style)));
                            || ()
                        }
                    },
                    code,
                );

                (*id_handle).as_ref().map_or_else(|| ::yew_style_in_rs::dynamic_style::style::StyleId("".into()), |id| id.clone())
            }}
        }
    }
}
