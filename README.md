# yew-style-in-rs
[![crates.io](https://img.shields.io/crates/v/yew-style-in-rs.svg)](https://crates.io/crates/yew-style-in-rs)
[![docs.rs](https://docs.rs/yew-style-in-rs/badge.svg)](https://docs.rs/yew-style-in-rs)

Scoped CSS in Rust for Yew.

## example

```
$ cd example/my-app
$ trunk serve --release
```

Note: `example/my-app/Trunk.toml` needs to be rewritten for non-Windows because relay on windows `copy` cmd.

## Usage

### `style!` macro

`css!` declaration generates scoped css at compile time.
`style.css` will be generated in the build directory at compile time.

#### `css!` declaration

`css!` macro generates scoped css at compile time.

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

`css!` macro can specify the name of the css file to be generated.

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
