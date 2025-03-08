use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
        <div class="app">
            <h1>{ "Anarchy Inference" }</h1>
            <div class="main-content">
                <p>{ "Welcome to the Anarchy Inference Development Environment" }</p>
            </div>
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}
