use proc_macro::TokenStream;

mod cursor;
mod state;
mod style;
mod util;

// expand macro with writing css files
#[proc_macro]
pub fn style_with_write(tokens: TokenStream) -> TokenStream {
    // set write_flag to true
    {
        use crate::state::*;
        let mut state = STATE.lock().unwrap();
        state.set_write_flag(true);
    }

    // expand macro
    let style = syn::parse_macro_input!(tokens as style::Style);
    style.expand().into()
}

// expand macro without writing css files
#[proc_macro]
pub fn style_without_write(tokens: TokenStream) -> TokenStream {
    // set write_flag to false
    {
        use crate::state::*;
        let mut state = STATE.lock().unwrap();
        state.set_write_flag(false);
    }

    // expand macro
    let style = syn::parse_macro_input!(tokens as style::Style);
    style.expand().into()
}
