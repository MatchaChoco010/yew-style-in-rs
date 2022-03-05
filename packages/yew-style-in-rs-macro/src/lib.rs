use proc_macro::TokenStream;

mod dynamic_style;
mod static_style;
mod util;

/// `css!` macro generates scoped css at compile time.
/// `style.css` will be generated in the build directory at compile time.
///
/// # Example
///
/// ```
/// # use yew::prelude::*;
/// # use yew_style_in_rs::*;
/// #
/// #[function_component(MyComponentB)]
/// pub fn my_component_b() -> Html {
///     // You can pass filename
///     let important_css = css!("important", "background: red; color: #000033;");
///
///     // You can use CSS Nesting
///     let css = css!(
///         "
///     border: solid green 1px;
///     width: 100%;
///     height: 150px;
///     text-align: center;
///     box-sizing: border-box;
///
///     & > p {
///         background: white;
///     }
///     "
///     );
///
///     html! {
///         <div class={classes!(important_css, css)}>
///             <p>{"compile time static css"}</p>
///         </div>
///     }
/// }
/// ```
#[proc_macro]
pub fn css(token: TokenStream) -> TokenStream {
    static_style::css::css(token.into()).into()
}

/// `dynamic_css!` macro generates scoped css at runtime.
/// `style` elements are generated and inserted into the head of the html.
///
/// # Example
///
/// ```
/// # use yew::prelude::*;
/// # use yew_style_in_rs::*;
/// #
/// #[function_component(MyComponentA)]
/// pub fn my_component_a() -> Html {
///     let background_state = use_state(|| "pink");
///     let background = *background_state;
///     let box_shadow_state = use_state(|| "#ffffff");
///     let box_shadow = *box_shadow_state;
///
///     let onclick = Callback::from({
///         let background_state = background_state.clone();
///         move |_| {
///             if *background_state == "pink" {
///                 background_state.set("cyan");
///                 box_shadow_state.set("#101010");
///             } else {
///                 background_state.set("pink");
///                 box_shadow_state.set("#ffffff");
///             }
///         }
///     });
///
///     let css = css!(
///         "border: solid 1px black;
///     width: 100%;
///     height: 150px;
///     text-align: center;
///     box-sizing: border-box;
///
///     &:hover {
///         border: solid 10px black;
///     }
///     "
///     );
///     let dynamic_css = dynamic_css!(format!(
///         "background: {background};
///
///         & > p {{
///             box-shadow: 0 0 10px {box_shadow};
///         }}
///         "
///     ));
///
///     html! {
///         <div class={classes!(css, dynamic_css)} {onclick}>
///             <p>{"Click Me"}</p>
///             <p>{"dynamic css"}</p>
///         </div>
///     }
/// }
/// ```
#[proc_macro]
pub fn dynamic_css(token: TokenStream) -> TokenStream {
    dynamic_style::css::css(token.into()).into()
}
