use yew::*;
use cobul::*;
use photon_rs::PhotonImage;

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

#[function_component(App)]
pub fn app() -> Html {
    let image = use_state(|| None);

    let onupload = callback!(image; move |files: Vec<web_sys::File>| {
        let blob = gloo::file::Blob::from(files[0].clone());

        spawn!(image; async move {
            let url = gloo::file::futures::read_as_data_url(&blob).await.unwrap();
            image.set(Some(url));
        });
    });

    let image = match (*image).clone() {
        Some(src) => html! {
            <img {src}/> 
            <div style="height:50px;border-left: 6px solid green"></div>
        },
        None => html! {}
    };

    html! {
        <Section>
        <Box>
        <File {onupload} />
        {image}
        </Box>
        </Section>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    Renderer::<App>::new().render();
}
