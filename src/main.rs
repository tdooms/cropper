mod cropper;

use std::rc::Rc;
use yew::*;
use cobul::*;

use gloo::file::futures::read_as_data_url;
use crate::cropper::Cropper;


#[macro_export]
macro_rules! callback {
    ( $( $x:ident ),*; $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            Callback::from($y)
        }
    };
}

#[macro_export]
macro_rules! spawn {
    ( $( $x:ident ),*; $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            wasm_bindgen_futures::spawn_local($y)
        }
    };
}

#[macro_export]
macro_rules! effect {
    ( $( $x:ident ),*; $y:expr, $z:expr) => {
        {
            $(let $x = $x.clone();)*
            yew::use_effect_with_deps($y, $z)
        }
    };
}



#[function_component(App)]
pub fn app() -> Html {
    let source: UseStateHandle<Option<Rc<String>>> = use_state_eq(|| None);
    let zoom = use_state_eq(|| 1.0);
    let bounding = use_state(|| None);

    let onupload = callback!(source; move |files: Vec<web_sys::File>| {
        let blob = gloo::file::Blob::from(files[0].clone());

        spawn!(source; async move {
            let data = read_as_data_url(&blob).await.unwrap();
            source.set(Some(Rc::new(data)))
        });
    });

    let onchange = callback!(zoom; move |value| zoom.set(value));
    let onclose = callback!(source; move |_| source.set(None));
    let ondone = callback!(; |_| log::info!("done"));

    let callback = callback!(bounding; move |value| bounding.set(Some(value)));

    let footer = html!{
        <Buttons>
        <Button onclick={onclose.clone()}> <span> {"Cancel"} </span> </Button>
        <Button color={Color::Primary} onclick={ondone}> <span> {"Save"} </span> </Button>
        </Buttons>
    };

    let image = match (*source).clone() {
        Some(src) => html! {
            <ModalCard title="Crop image tool" active=true {footer} {onclose}>
                <Cropper width=400 height=300 zoom={*zoom} {src} {callback}/>
                <Slider<f64> range={1.0..3.0} value={*zoom} steps=50 {onchange}/>
            </ModalCard>
        },
        None => html! {}
    };

    html! {
        <Section>

        {image}
        <File {onupload} />

        </Section>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    Renderer::<App>::new().render();
}
