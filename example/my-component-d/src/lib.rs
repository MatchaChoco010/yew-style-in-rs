use yew::prelude::*;
use yew_style_in_rs::*;

#[function_component(MyComponentD)]
pub fn my_component_d() -> Html {
    let from_x_ref = use_mut_ref(|| 20);
    let to_x_ref = use_mut_ref(|| 20);
    let disabled_button = use_mut_ref(|| false);
    let animate_class = use_state(|| None);

    let onclick = Callback::from({
        let to_x_ref = to_x_ref.clone();
        let disabled_button = disabled_button.clone();
        let animate_class = animate_class.clone();
        move |_| {
            *to_x_ref.borrow_mut() += 20;
            if *to_x_ref.borrow() >= 100 {
                *to_x_ref.borrow_mut() = 0;
            }
            *disabled_button.borrow_mut() = true;
            animate_class.set(Some("animate"));
        }
    });
    let onanimationend = Callback::from({
        let from_x_ref = from_x_ref.clone();
        let disabled_button = disabled_button.clone();
        let animate_class = animate_class.clone();
        move |_| {
            *from_x_ref.borrow_mut() += 20;
            if *from_x_ref.borrow() >= 100 {
                *from_x_ref.borrow_mut() = 0;
            }
            *disabled_button.borrow_mut() = false;
            animate_class.set(None);
        }
    });

    let from_x = *from_x_ref.borrow();
    let to_x = *to_x_ref.borrow();
    let disabled = *disabled_button.borrow();

    style! {
        let css = css! {r#"
            border-top: solid 20px;
            border-bottom: solid 20px;
            width: 100%;
            height: 150px;
            text-align: center;
            box-sizing: border-box;

            & > div {
                width: 50px;
                height: 50px;
                background: black;
            }
        "#};
        let dyn_css = dyn css!{r#"
            & > div {
                transform: translateX(${from_x}vw);
            }
            & > div.animate {
                animation: ##translate## 1s;
            }
        "#};
        dyn keyframes!{r#"
            @keyframes translate {
                to {
                    transform: translateX(${to_x}vw);
                }
            }
        "#}
    }
    html! {
        <div class={classes!(css, dyn_css)}>
            <div class={*animate_class} {onanimationend}/>
            <button {onclick} {disabled}>{"Click Me!"}</button>
        </div>
    }
}
