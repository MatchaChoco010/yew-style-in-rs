use yew::prelude::*;
use yew_style_in_rs::*;

#[function_component(MyComponentB)]
pub fn my_component_b() -> Html {
    let important_css = css!("important", "background: red; color: #000033;");
    let css = css!(
        "
    border: solid green 1px;
    width: 100%;
    height: 150px;
    text-align: center;
    box-sizing: border-box;

    & > p {
        background: white;
    }
    "
    );

    html! {
        <div class={classes!(important_css, css)}>
            <p>{"compile time static css"}</p>
        </div>
    }
}
