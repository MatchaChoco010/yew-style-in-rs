use yew::prelude::*;
use yew_style_in_rs::*;

#[function_component(MyComponentA)]
pub fn my_component_a() -> Html {
    let background_state = use_state(|| "pink");
    let background = *background_state;
    let box_shadow_state = use_state(|| "#ffffff");
    let box_shadow = *box_shadow_state;

    let onclick = Callback::from({
        let background_state = background_state.clone();
        move |_| {
            if *background_state == "pink" {
                background_state.set("cyan");
                box_shadow_state.set("#101010");
            } else {
                background_state.set("pink");
                box_shadow_state.set("#ffffff");
            }
        }
    });

    style! {
        let css = css!{r#"
            border: solid 1px black;
            width: 100%;
            height: 150px;
            text-align: center;
            box-sizing: border-box;

            &:hover {
                border: solid 10px black;
            }
        "#};
        let dynamic_css = dyn css!{r#"
            background: ${background};

            & > p {
                box-shadow: 0 0 10px ${box_shadow};
            }
        "#};
    }
    html! {
        <div class={classes!(css, dynamic_css)} {onclick}>
            <p>{"Click Me"}</p>
            <p>{"dynamic css"}</p>
        </div>
    }
}
