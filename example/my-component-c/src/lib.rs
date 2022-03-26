use yew::prelude::*;
use yew_style_in_rs::*;

#[function_component(MyComponentC)]
pub fn my_component_c() -> Html {
    style! {
        let important_css = css!(filename = "important") {r#"
            background: green;
            color: #000033;
        "#};
        let css = css! {r#"
            border: solid 20px;
            width: 100%;
            height: 150px;
            text-align: center;
            box-sizing: border-box;

            animation: ##border## 3s infinite alternate;

            & > p {
                animation: ##rotation## 1s infinite linear;
            }
        "#};
        keyframes!(filename = "important") {r#"
            @keyframes rotation {
                from {
                    transform: rotate(0deg);
                }
                to {
                    transform: rotate(360deg)
                }
            }
        "#}
        keyframes!{r#"
            @keyframes border {
                from {
                    border-color: lime;
                }
                to {
                    border-color: purple;
                }
            }
        "#}
    }
    html! {
        <div class={classes!(important_css, css)}>
            <p>{"compile time static css animation"}</p>
        </div>
    }
}
