# yew-style-in-rs

CSS in Rust for Yew

## example

```
$ cd example/my-app
$ trunk serve --release
```

Note: `example/my-app/Trunk.toml` needs to be rewritten for non-Windows.

## Usage

### `css!` macro

`css!` macro generates scoped css at compile time.

```rust
use yew::prelude::*;
use yew_style_in_rs::*;

#[function_component(MyComponentB)]
pub fn my_component_b() -> Html {
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
        <div class={css}>
            <p>{"compile time static css!"}</p>
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

#[function_component(MyComponentB)]
pub fn my_component_b() -> Html {
    let important_css = css!("important", "background: red;");
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
            <p>{"compile time static css!"}</p>
        </div>
    }
}
```

The above code generates `style.css` and `important.css`.

You can load `important.css` synchronously and `style.css` asynchronously as follows iin html file.

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

The `css!` macro only accepts string literals. Only strings determined at compile time are accepted.
If you want to change the style at runtime, use the following `dynamic_css!` macro.

### `dynamic_css!` macro

`dynamic_css!` macro generates scoped css at runtime.

```rust
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

    let dynamic_css = dynamic_css!(format!(
        "background: {background};

        & > p {{
            box-shadow: 0 0 10px {box_shadow};
        }}
        "
    ));

    html! {
        <div class={classes!(css, dynamic_css)} {onclick}>
            <p>{"Click Me"}</p>
            <p>{"dynamic css"}</p>
        </div>
    }
}
```

The above code generates the following style elements and insert into the head of the html.

```html
<style data-style="dynamic-AbCdEfGh">
  .AbCdEfGh {
    background: pink;
  }

  .AbCdEfGh > p {
    box-shadow: 0 0 10px #ffffff;
  }
</style>
```

`AbCdEfGh` is a random 8-letter alphabet.
Note that CSS Nesting can be used.
