use yew::*;
use cobul::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};
use wasm_bindgen::JsCast;


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
    let image: UseStateHandle<Option<String>> = use_state(|| None);
    let position = use_state(|| None);
    let canvas = use_node_ref();

    use_effect_with_deps(move |(canvas, image, position)| {
        let canvas = canvas.cast::<HtmlCanvasElement>().unwrap();
        let element = canvas.get_context("2d").unwrap().unwrap();
        let context = element.dyn_into::<CanvasRenderingContext2d>().unwrap();

        if let Some(src) = &**image {
            let img = HtmlImageElement::new().unwrap();
            img.set_src(src.as_str());

            let (x, y) = match **position {
                Some(tuple) => tuple,
                None => (0, 0),
            };

            context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &img,
                x as f64,
                y as f64,
                100.0,
                100.0,
                0.0,
                0.0,
                100.0,
                100.0,
            ).unwrap();
            log::info!("bing bong");
        }
        || ()
    }, (canvas.clone(), image.clone(), position.clone()));

    let onupload = callback!(image; move |files: Vec<web_sys::File>| {
        let blob = gloo::file::Blob::from(files[0].clone());

        spawn!(image; async move {
            let url = gloo::file::futures::read_as_data_url(&blob).await.unwrap();
            image.set(Some(url));
        });
    });

    let img_move = callback!(position; move |ev: MouseEvent| {
        let new = (ev.offset_x(), ev.offset_y());
        position.set(Some(new))
    });
    let lens_move = callback!(position; move |ev: MouseEvent| {
        let new = (position.unwrap().0 + ev.offset_x() - 50, position.unwrap().1 + ev.offset_y() - 50);
        position.set(Some(new));
    });

    let lens = match *position {
        Some((x, y)) => {
            let style = format!("position:absolute;left:{}px;top:{}px;height:100px;width:100px;border:2px solid green", x - 50, y - 50);
            html! {<div {style} onmousemove={lens_move}/>}
        }
        None => html! {}
    };

    let image = match (*image).clone() {
        Some(src) => html! {
            <div style="position:relative">
            {lens}
            <img src={src.clone()} onmousemove={img_move}/>
            </div>
        },
        None => html! {}
    };

    html! {
        <Section>
        <Box>
        <canvas style="display:block" ref={canvas}/>
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
