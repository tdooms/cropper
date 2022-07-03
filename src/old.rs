use yew::*;
use cobul::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};
use wasm_bindgen::JsCast;
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

pub fn draw(canvas: HtmlCanvasElement, image: HtmlImageElement, factor: u64) {
    let zoom = 1.0 + factor as f64 / 50.0;
    log::info!("zoom: {zoom}");

    let element = canvas.get_context("2d").unwrap().unwrap();
    let context = element.dyn_into::<CanvasRenderingContext2d>().unwrap();

    let (i_width, i_height) = (image.width() as f64, image.height() as f64);
    let (c_width, c_height) = (canvas.width() as f64, canvas.height() as f64);

    let scale = (c_width / i_width).min(c_height / i_height);

    let (d_width, d_height) = (i_width * scale * zoom, i_height * scale * zoom);
    let (x_offset, y_offset) = ((c_width - d_width) / 2.0, (c_height - d_height) / 2.0);

    context.set_image_smoothing_enabled(true);
    context.clear_rect(0.0, 0.0, c_width, c_height);

    context.draw_image_with_html_image_element_and_dw_and_dh(
        &image,
        x_offset,
        y_offset,
        d_width,
        d_height,
    ).unwrap();


    let ratio = 4.0 / 3.0;
    let size = (d_width * ratio).min(d_height) / (zoom * 2.0);
    let (x_start, y_start) = (c_width / 2.0 - size * ratio, c_height / 2.0 - size);

    context.set_global_alpha(0.5);
    context.fill_rect(0.0, 0.0, x_start, c_height);
    context.fill_rect(0.0, 0.0, c_width, y_start);
    context.fill_rect(c_width - x_start, 0.0, x_start, c_height);
    context.fill_rect(0.0, c_height - y_start, c_width, y_start);
}

#[function_component(App)]
pub fn app() -> Html {
    let source: UseStateHandle<Option<String>> = use_state(|| None);
    let position = use_state(|| None);
    let clicked = use_state(|| false);
    let zoom = use_state(|| 0);

    let canvas = use_node_ref();
    let image = use_node_ref();

    let onload = callback!(canvas, image, zoom; move |_| {
        draw(canvas.cast().unwrap(), image.cast().unwrap(), *zoom)
    });

    let onupload = callback!(source; move |files: Vec<web_sys::File>| {
        let blob = gloo::file::Blob::from(files[0].clone());
        spawn!(source; async move {source.set(Some(read_as_data_url(&blob).await.unwrap())) });
    });

    let onchange = callback!(canvas, image, zoom; move |value| {
        zoom.set(value);
        draw(canvas.cast().unwrap(), image.cast().unwrap(), value)
    });

    let onmousedown = callback!(clicked; move |_: MouseEvent| clicked.set(true));
    let onmouseup = callback!(clicked; move |_: MouseEvent| clicked.set(false));

    let onmousemove = callback!(position, clicked; move |ev: MouseEvent| {
        if !*clicked {
            return;
        }

        let new = (ev.offset_x(), ev.offset_y());
        position.set(Some(new))
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
        <canvas width=400 height=300 ref={canvas} style="border:1px" {onmousedown} {onmouseup} {onmousemove}/>
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
use yew::*;
use cobul::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};
use wasm_bindgen::JsCast;
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

pub fn draw(canvas: HtmlCanvasElement, image: HtmlImageElement, factor: u64) {
    let zoom = 1.0 + factor as f64 / 50.0;
    log::info!("zoom: {zoom}");

    let element = canvas.get_context("2d").unwrap().unwrap();
    let context = element.dyn_into::<CanvasRenderingContext2d>().unwrap();

    let (i_width, i_height) = (image.width() as f64, image.height() as f64);
    let (c_width, c_height) = (canvas.width() as f64, canvas.height() as f64);

    let scale = (c_width / i_width).min(c_height / i_height);

    let (d_width, d_height) = (i_width * scale * zoom, i_height * scale * zoom);
    let (x_offset, y_offset) = ((c_width - d_width) / 2.0, (c_height - d_height) / 2.0);

    context.set_image_smoothing_enabled(true);
    context.clear_rect(0.0, 0.0, c_width, c_height);

    context.draw_image_with_html_image_element_and_dw_and_dh(
        &image,
        x_offset,
        y_offset,
        d_width,
        d_height,
    ).unwrap();


    let ratio = 4.0 / 3.0;
    let size = (d_width * ratio).min(d_height) / (zoom * 2.0);
    let (x_start, y_start) = (c_width / 2.0 - size * ratio, c_height / 2.0 - size);

    context.set_global_alpha(0.5);
    context.fill_rect(0.0, 0.0, x_start, c_height);
    context.fill_rect(0.0, 0.0, c_width, y_start);
    context.fill_rect(c_width - x_start, 0.0, x_start, c_height);
    context.fill_rect(0.0, c_height - y_start, c_width, y_start);
}

#[function_component(App)]
pub fn app() -> Html {
    let source: UseStateHandle<Option<String>> = use_state(|| None);
    let position = use_state(|| None);
    let clicked = use_state(|| false);
    let zoom = use_state(|| 0);

    let canvas = use_node_ref();
    let image = use_node_ref();

    let onload = callback!(canvas, image, zoom; move |_| {
        draw(canvas.cast().unwrap(), image.cast().unwrap(), *zoom)
    });

    let onupload = callback!(source; move |files: Vec<web_sys::File>| {
        let blob = gloo::file::Blob::from(files[0].clone());
        spawn!(source; async move {source.set(Some(read_as_data_url(&blob).await.unwrap())) });
    });

    let onchange = callback!(canvas, image, zoom; move |value| {
        zoom.set(value);
        draw(canvas.cast().unwrap(), image.cast().unwrap(), value)
    });

    let onmousedown = callback!(clicked; move |_: MouseEvent| clicked.set(true));
    let onmouseup = callback!(clicked; move |_: MouseEvent| clicked.set(false));

    let onmousemove = callback!(position, clicked; move |ev: MouseEvent| {
        if !*clicked {
            return;
        }

        let new = (ev.offset_x(), ev.offset_y());
        position.set(Some(new))
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
        <canvas width=400 height=300 ref={canvas} style="border:1px" {onmousedown} {onmouseup} {onmousemove}/>
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
