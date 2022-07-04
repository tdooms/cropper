use yew::*;
use cobul::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};
use wasm_bindgen::{JsCast, JsValue};
use gloo::file::futures::read_as_data_url;


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

pub fn draw(canvas: HtmlCanvasElement, image: HtmlImageElement, zoom: u64, position: (i32, i32)) {
    let (x_border, y_border) = (40.0, 30.0);
    let zoom = 1.0 + zoom as f64 / 50.0;

    let element = canvas.get_context("2d").unwrap().unwrap();
    let context = element.dyn_into::<CanvasRenderingContext2d>().unwrap();

    let (i_width, i_height) = (image.width() as f64, image.height() as f64);
    let (c_width, c_height) = (canvas.width() as f64, canvas.height() as f64);
    let (l_width, l_height) = (c_width - 2.0 * y_border, c_height - 2.0 * x_border);

    let scale = (l_width / i_width).max(l_height / i_height);

    let (d_width, d_height) = (i_width * scale * zoom, i_height * scale * zoom);
    let (x_offset, y_offset) = ((c_width - d_width) / 2.0, (c_height - d_height) / 2.0);

    context.set_image_smoothing_enabled(true);
    // context.clear_rect(0.0, 0.0, c_width, c_height);

    context.set_global_alpha(1.0);
    context.set_fill_style(&JsValue::from("red"));
    context.fill_rect(0.0, 0.0, c_width, c_height);
    context.set_fill_style(&JsValue::from("black"));

    context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
        &image,
        position.0 as f64,
        position.1 as f64,
        i_width,
        i_height,
        x_offset,
        y_offset,
        d_width,
        d_height,
    ).unwrap();

    context.set_global_alpha(0.5);
    context.fill_rect(0.0, 0.0, y_border, c_height);
    context.fill_rect(0.0, 0.0, c_width, x_border);
    context.fill_rect(c_width - y_border, 0.0, y_border, c_height);
    context.fill_rect(0.0, c_height - x_border, c_width, x_border);
}

#[function_component(App)]
pub fn app() -> Html {
    let source: UseStateHandle<Option<String>> = use_state(|| None);
    let position = use_state(|| (0, 0));
    let clicked = use_state(|| None);
    let zoom = use_state(|| 0);

    let canvas = use_node_ref();
    let image = use_node_ref();

    let onload = callback!(canvas, image, zoom, position; move |_| {
        draw(canvas.cast().unwrap(), image.cast().unwrap(), *zoom, *position)
    });

    let onupload = callback!(source; move |files: Vec<web_sys::File>| {
        let blob = gloo::file::Blob::from(files[0].clone());
        spawn!(source; async move {source.set(Some(read_as_data_url(&blob).await.unwrap())) });
    });

    let onchange = callback!(canvas, image, zoom, position; move |value| {
        zoom.set(value);
        draw(canvas.cast().unwrap(), image.cast().unwrap(), value, *position)
    });

    let onmousedown = callback!(clicked; move |ev: MouseEvent| {
        let new = (ev.offset_x(), ev.offset_y());
        clicked.set(Some(new));
    });
    let onmouseup = callback!(clicked; move |_: MouseEvent| clicked.set(None));
    let onmouseout = callback!(clicked; move |_: MouseEvent| clicked.set(None));

    let onmousemove = callback!(canvas, image, position, zoom, clicked; move |ev: MouseEvent| {
        let new = (ev.offset_x(), ev.offset_y());
        if let Some((start_x, start_y)) = *clicked {
            clicked.set(Some(new));

            let new = (position.0 - new.0 + start_x, position.1 - new.1 + start_y);
            position.set(new);
            draw(canvas.cast().unwrap(), image.cast().unwrap(), *zoom, new);

        }
    });

    // let lens_move = callback!(position; move |ev: MouseEvent| {
    //     let new = (position.unwrap().0 + ev.offset_x() - 50, position.unwrap().1 + ev.offset_y() - 50);
    //     position.set(Some(new));
    // });

    // let lens = match *position {
    //     Some((x, y)) => {
    //         let style = format!("position:absolute;left:{}px;top:{}px;height:100px;width:100px;border:2px solid green", x - 50, y - 50);
    //         html! {<div {style} onmousemove={lens_move}/>}
    //     }
    //     None => html! {}
    // };

    log::info!("position: {:?}, zoom: {}", *position, *zoom);

    let src = (*source).clone();
    html! {
        <Section>
        <canvas width=400 height=300 ref={canvas} style="border:1px" {onmousedown} {onmouseup} {onmousemove} {onmouseout}/>
        <File {onupload} />

        <Columns>
        <Column size={ColumnSize::Is3}>
        <Slider<u64> range={0..100} value={*zoom} steps=100 {onchange}/>
        </Column>
        </Columns>

        <img {src} style="display:none" ref={image} {onload}/>
        </Section>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    Renderer::<App>::new().render();
}
