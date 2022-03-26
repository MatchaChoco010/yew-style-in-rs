use wasm_bindgen::prelude::wasm_bindgen;
use yew::prelude::*;

use my_component_a::MyComponentA;
use my_component_b::MyComponentB;
use my_component_c::MyComponentC;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[function_component(MyApp)]
fn my_app() -> Html {
    html! {
        <>
            <MyComponentA />
            <MyComponentB />
            <MyComponentA />
            <MyComponentC />
        </>
    }
}

#[no_mangle]
#[wasm_bindgen(start)]
pub fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<MyApp>();
}
