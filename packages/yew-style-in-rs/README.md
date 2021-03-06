# yew-style-in-rs

## Features

- `dry-run`: No write css file to disk. Without this feature, this crate create and write css file to target disk. This feature is useful for document build.

If you would like to publish some components uses `yew-style-in-rs` to crates.io, you might need to write following contents to Cargo.toml because crates.io docs build environment can't write filesystem:

```toml
[features]
default = []
dry-run = ["yew-style-in-rs/dry-run"]

[package.metadata.docs.rs]
cargo-args = ["--features=dry-run"]
```

You might need to publish with `dry-run`.

```sh
$ cargo publish --features dry-run
```

## Usage

### `style!` macro

`style!` macro generates scoped css.
`style!` macro can contain `css!`, `dyn css!`, `keyframes!` or `dyn keyframes!` declarations.

#### `css!` declaration

`css!` declaration generates scoped css at compile time.

```rust
use yew::prelude::*;
use yew_style_in_rs::*;

#[function_component(MyComponent)]
pub fn my_component() -> Html {
    style! {
        // You can use CSS Nesting
        let css = css! {r#"
            border: solid green 1px;
            width: 100%;
            height: 150px;
            text-align: center;
            box-sizing: border-box;

            & > p {
                background: white;
            }
        "#};
    }
    html! {
        <div class={classes!(css)}>
            <p>{"compile time static css"}</p>
        </div>
    }
}
```

The above code generates the following `style.css` in the build directory.

```css
.AbCdEfGh {
  border: solid green 1px;
  width: 100%;
  height: 150px;
  text-align: center;
  box-sizing: border-box;
}

.AbCdEfGh > p {
  background: white;
}
```

`AbCdEfGh` is a random 8-letter alphabet.
Note that CSS Nesting can be used.

`css!` declaration can specify the name of the css file to be generated.

```rust
use yew::prelude::*;
use yew_style_in_rs::*;

#[function_component(MyComponent)]
pub fn my_component() -> Html {
    style! {
        // You can pass filename
        let important_css = css!(filename = "important") {r#"
            background: red;
            color: #000033;
        "#};
        // You can use CSS Nesting
        let css = css! {r#"
            border: solid green 1px;
            width: 100%;
            height: 150px;
            text-align: center;
            box-sizing: border-box;

            & > p {
                background: white;
            }
        "#};
    }
    html! {
        <div class={classes!(important_css, css)}>
            <p>{"compile time static css"}</p>
        </div>
    }
}
```

The above code generates `style.css` and `important.css`.

You can load `important.css` synchronously and `style.css` asynchronously as follows in html file.

```html
<!DOCTYPE html>
<html lang="en-US">
  <head>
    <meta charset="utf-8"/>
    <title>yew-style-in-rs</title>

    <!-- important style -->
    <link rel="stylesheet" href="./important.css"/>

    <!-- async load style -->
    <link rel="stylesheet" href="./style.css" media="print" onload="this.media='all'">
  </head>
</html>
```

The `css!` declaration can be only static because of compile time CSS generation.
If you want to change the style at runtime, use the following `dyn css!` declaration.

#### `dyn css!` declaration

`dyn css!` declaration generates scoped css at runtime.

```rust
use yew::prelude::*;
use yew_style_in_rs::*;

#[function_component(MyComponent)]
pub fn my_component() -> Html {
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
        let dynamic_css = dyn css! {r#"
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
```

The above code generates a following style html element and insert into the head of the html.

```html
<style data-style="dynamic-AbCdEfGh">
  .dynamic-AbCdEfGh {
    background: pink;
  }

  .dynamic-AbCdEfGh > p {
    box-shadow: 0 0 10px #ffffff;
  }
</style>
```

`AbCdEfGh` is a random 8-letter alphabet.
Note that CSS Nesting can be used.

You can use both `css!` declaration and `dyn css!` declaration in one `style!` macro.

```rust
use yew::prelude::*;
use yew_style_in_rs::*;

#[function_component(MyComponent)]
pub fn my_component() -> Html {
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
        let dynamic_css = dyn css! {r#"
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
```

Interpolation by ${{}} can be used in various places.

```rs
# use yew::prelude::*;
# use yew_style_in_rs::*;
#
#[function_component(MyComponent)]
pub fn my_component() -> Html {
    let property_name = "background";
    let property_value = "black";
    let selector = "p";
    let declaration = "& > div { background: white; }";
    style! {
        let dynamic_css = dyn css! {r#"
            ${property_name}: ${property_value};
            & > ${selector} {
                box-shadow: 0 0 10px rgba(0, 0, 0, 0.5);
            }
            ${declaration}
        "#};
    }
    html! {
        <div class={dynamic_css}>
            <p>{"dynamic css"}</p>
            <div>{"Hi"}</div>
        </div>
    }
}
```

#### `keyframes!` declaration

`keyframes!` declaration generates scoped @keyframes at compile time.
`style.css` will be generated in the build directory at compile time.

```rs
# use yew::prelude::*;
# use yew_style_in_rs::*;
#
#[function_component(MyComponent)]
pub fn my_component() -> Html {
    style! {
        let css = css! {r#"
            animation: ##anim## 1s;
        "#};
        keyframes!{r#"
            @keyframes anim {
                from { transform: translateX(0px); }
                to { transform: translateX(100px); }
            }
        "#}
    }
    html! {
        <div class={classes!(css)}>
            <p>{"compile time css animation"}</p>
        </div>
    }
}
```

The declared animation is used by writing `##<ANIMATION NAME>##` like `##anim##`.
To create scope, `anim` will be renamed like `anim-AbCdEfGh` when compiled.

You can pass a filename of the style sheet.

```rs
# use yew::prelude::*;
# use yew_style_in_rs::*;
#
#[function_component(MyComponent)]
pub fn my_component() -> Html {
    style! {
        let css = css! {r#"
            animation: ##anim## 1s;
        "#};
        keyframes!(filename = "important") {r#"
            @keyframes anim {
                from { transform: translateX(0px); }
                to { transform: translateX(100px); }
            }
        "#}
    }
    html! {
        <div class={classes!(css)}>
            <p>{"compile time css animation"}</p>
        </div>
    }
}
```

The above code generates `important.css`.

#### `dyn keyframes!` declaration

`dyn keyframes!` declaration generates scoped css at runtime.
`style` html elements are generated and inserted into the head of the html.

```rs
# use yew::prelude::*;
# use yew_style_in_rs::*;
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
```

The declared animation is used by writing `##<ANIMATION NAME>##` like `##translate##`.
To create scope, `translate` will be renamed like `translate-dynamic-AbCdEfGh` when compiled.

The keyframes declare in `dyn keyframes!` can be used only in `dyn css!` declaration.
The keyframes declare in `dyn keyframes!` can't be used in`css!` declaration.
