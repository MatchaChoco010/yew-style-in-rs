use proc_macro::TokenStream;

mod cursor;
mod state;
mod style;
mod util;

#[proc_macro]
pub fn style(tokens: TokenStream) -> TokenStream {
    let style = syn::parse_macro_input!(tokens as style::Style);
    style.expand().into()
}
