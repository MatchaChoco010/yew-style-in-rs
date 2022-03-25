/// `style!` macro generates scoped css.
/// `style!` macro can contain `css!` and/or `dyn css!` declarations.
///
/// # `css!` declaration
///
/// `css!` declaration generates scoped css at compile time.
/// `style.css` will be generated in the build directory at compile time.
///
/// ## Example
///
/// ```
/// # use yew::prelude::*;
/// # use yew_style_in_rs::*;
/// #
/// #[function_component(MyComponent)]
/// pub fn my_component() -> Html {
///     style! {
///         // You can use CSS Nesting
///         let css = css! {r#"
///             border: solid green 1px;
///             width: 100%;
///             height: 150px;
///             text-align: center;
///             box-sizing: border-box;
///
///             & > p {
///                 background: white;
///             }
///         "#};
///     }
///     html! {
///         <div class={classes!(css)}>
///             <p>{"compile time static css"}</p>
///         </div>
///     }
/// }
/// ```
///
/// You can pass a filename of the style sheet.
///
/// ```
/// # use yew::prelude::*;
/// # use yew_style_in_rs::*;
/// #
/// #[function_component(MyComponent)]
/// pub fn my_component() -> Html {
///     style! {
///         // You can pass filename
///         let important_css = css!(filename = "important") {r#"
///             background: red;
///             color: #000033;
///         "#};
///         // You can use CSS Nesting
///         let css = css! {r#"
///             border: solid green 1px;
///             width: 100%;
///             height: 150px;
///             text-align: center;
///             box-sizing: border-box;
///
///             & > p {
///                 background: white;
///             }
///         "#};
///     }
///     html! {
///         <div class={classes!(important_css, css)}>
///             <p>{"compile time static css"}</p>
///         </div>
///     }
/// }
/// ```
///
/// The above code generates `style.css` and `important.css`.
///
/// # `dyn css!` declaration
///
/// `dyn css!` macro generates scoped css at runtime.
/// `style` html elements are generated and inserted into the head of the html.
///
/// ## Example
///
/// ```
/// # use yew::prelude::*;
/// # use yew_style_in_rs::*;
/// #
/// #[function_component(MyComponent)]
/// pub fn my_component() -> Html {
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
///     style! {
///         let css = css!{r#"
///             border: solid 1px black;
///             width: 100%;
///             height: 150px;
///             text-align: center;
///             box-sizing: border-box;
///
///             &:hover {
///                 border: solid 10px black;
///             }
///         "#};
///         let dynamic_css = dyn css! {r#"
///             background: ${background};
///
///             & > p {
///                 box-shadow: 0 0 10px ${box_shadow};
///             }
///         "#};
///     }
///     html! {
///         <div class={classes!(css, dynamic_css)} {onclick}>
///             <p>{"Click Me"}</p>
///             <p>{"dynamic css"}</p>
///         </div>
///     }
/// }
/// ```
pub use yew_style_in_rs_macro::style;

#[doc(hidden)]
pub use yew_style_in_rs_core::*;

#[doc(hidden)]
pub mod css;

#[doc(hidden)]
pub mod dyn_css;

#[doc(hidden)]
pub mod runtime_manager;
