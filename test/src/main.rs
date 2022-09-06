use std::rc::Rc;
use yew::*;
use cobul::*;

use gloo::file::futures::read_as_data_url;
use cropper::Cropper;


macro_rules! spawn {
    ( $( $x:ident ),*; $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            wasm_bindgen_futures::spawn_local($y)
        }
    };
}

macro_rules! callback {
    ( $( $x:ident ),*; $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            Callback::from($y)
        }
    };
}


#[function_component(App)]
pub fn app() -> Html {
    let source: UseStateHandle<Option<Rc<String>>> = use_state_eq(|| None);
    let result = use_state(|| None);

    let input = callback!(source; move |files: Vec<web_sys::File>| {
        let blob = gloo::file::Blob::from(files[0].clone());

        spawn!(source; async move {
            let data = read_as_data_url(&blob).await.unwrap();
            source.set(Some(Rc::new(data)))
        });
    });

    let done = callback!(result; move |value| result.set(Some(value)));
    let cancel = callback!(source; move |_| source.set(None));

    let image = match ((*source).clone(), result.as_ref()) {
        (Some(src), None) => html! {<Cropper width=600 height=450 {src} {done} {cancel}/>},
        _ => html! {}
    };

    html! {
        <Section>

        {image}
        <File {input} />

        <img src={(*result).clone()} />

        </Section>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    Renderer::<App>::new().render();
}
