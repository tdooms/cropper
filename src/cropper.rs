use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};
use wasm_bindgen::JsCast;
use yew::*;
use crate::{callback, effect};
use std::f64::consts::PI;
use std::rc::Rc;

const V_BORDER: f64 = 40.0;
const H_BORDER: f64 = 30.0;

pub struct CenterImage {
    offset: (f64, f64),
    dims: (f64, f64),
    scale: f64,
}

pub fn center_image(canvas: &HtmlCanvasElement, image: &HtmlImageElement, zoom: f64) -> CenterImage {
    let (i_width, i_height) = (image.width() as f64, image.height() as f64);
    let (c_width, c_height) = (canvas.width() as f64, canvas.height() as f64);
    let (l_width, l_height) = (c_width - 2.0 * V_BORDER, c_height - 2.0 * H_BORDER);

    let scale = (l_width / i_width).max(l_height / i_height) * zoom;

    let dims = (i_width * scale, i_height * scale);
    let offset = ((c_width - dims.0) / 2.0, (c_height - dims.1) / 2.0);

    CenterImage { dims, offset, scale }
}

pub fn constrain_position((pos_x, pos_y): (f64, f64), (off_x, off_y): (f64, f64)) -> (f64, f64) {
    let (x_win, y_win) = (V_BORDER - off_x, H_BORDER - off_y);

    let new_x = if x_win != 0.0 { pos_x.clamp(-x_win.abs(), x_win.abs()) } else { 0.0 };
    let new_y = if y_win != 0.0 { pos_y.clamp(-y_win.abs(), y_win.abs()) } else { 0.0 };

    (new_x, new_y)
}

pub fn bounding_box(image: &HtmlImageElement, (pos_x, pos_y): (f64, f64), scale: f64, zoom: f64) -> ((f64, f64), (f64, f64)) {
    log::debug!("{}, {}", pos_x / scale, pos_y / scale);
    let dims = (4.0 / 3.0 * image.height() as f64 / zoom, image.height() as f64 / zoom);

    log::info!("{zoom}");

    let x_corner = pos_x / scale + image.width() as f64 / 2.0 -  2.0/3.0 * image.height() as f64 / zoom;
    let y_corner = pos_y / scale + image.height() as f64 / 2.0 - image.height() as f64 / (2.0 * zoom);

    ((x_corner, y_corner), dims)
}

pub fn draw(canvas: HtmlCanvasElement, image: HtmlImageElement, zoom: f64, pos: (f64, f64), radius: f64) {
    let element = canvas.get_context("2d").unwrap().unwrap();
    let context = element.dyn_into::<CanvasRenderingContext2d>().unwrap();

    let CenterImage { dims, offset, scale } = center_image(&canvas, &image, zoom);
    let (width, height) = (canvas.width() as f64, canvas.height() as f64);

    context.set_image_smoothing_enabled(true);
    context.clear_rect(0.0, 0.0, width, height);
    context.fill_rect(0.0, 0.0, width, height);
    context.set_global_alpha(1.0);
    context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
        &image,
        pos.0 / scale,
        pos.1 / scale,
        image.width() as f64,
        image.height() as f64,
        offset.0,
        offset.1,
        dims.0,
        dims.1,
    ).unwrap();

    context.set_global_alpha(0.5);

    context.begin_path();
    context.move_to(V_BORDER, H_BORDER + radius); // start top-left
    context.arc(V_BORDER + radius, H_BORDER + radius, radius, PI, 1.5 * PI).unwrap();
    context.line_to(width - V_BORDER - radius, H_BORDER); // top-right of image
    context.arc(width - V_BORDER - radius, H_BORDER + radius, radius, 1.5 * PI, 0.0).unwrap();
    context.line_to(width - V_BORDER, height - H_BORDER - radius); // bottom-right of image
    context.arc(width - V_BORDER - radius, height - H_BORDER - radius, radius, 0.0, 0.5 * PI).unwrap();
    context.line_to(V_BORDER + radius, height - H_BORDER); // bottom-left of image
    context.arc(V_BORDER + radius, height - H_BORDER - radius, radius, 0.5 * PI, PI).unwrap();
    context.close_path();

    context.rect(width, 0.0, -width, height);
    context.fill();
}

#[derive(Properties, PartialEq, Clone, Debug)]
pub struct Props {
    pub width: u64,
    pub height: u64,

    #[prop_or(1.0)]
    pub zoom: f64,

    #[prop_or(30.0)]
    pub radius: f64,

    pub src: Rc<String>,

    pub callback: Callback<((f64, f64), (f64, f64))>,
}

// TODO: deal with zoom out when at border -> leads to 'out of bounds'
#[function_component(Cropper)]
pub fn cropper(props: &Props) -> Html {
    let Props { width, height, zoom, src, radius, callback } = props.clone();

    let image = use_state(|| {
        let image = HtmlImageElement::new().unwrap();
        image.set_src(&src);
        image
    });

    let position = use_state(|| (0.0, 0.0));
    let clicked = use_state(|| None);
    let canvas = use_node_ref();

    let bounding = use_state(|| ((0.0, 0.0), (0.0, 0.0)));
    let preview = use_node_ref();

    let (canvas_c, image_c, position_c, bounding_c, preview_c) = (canvas.clone(), image.clone(), position.clone(), bounding.clone(), preview.clone());
    use_effect(move || {
        draw(canvas_c.cast().unwrap(), (*image_c).clone(), zoom, *position_c, radius);

        let canvas: HtmlCanvasElement = preview_c.cast().unwrap();
        let element = canvas.get_context("2d").unwrap().unwrap();
        let context = element.dyn_into::<CanvasRenderingContext2d>().unwrap();

        context.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
        log::info!("{bounding_c:?}");
        context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
            &image_c,
            bounding_c.0.0,
            bounding_c.0.1,
            bounding_c.1.0,
            bounding_c.1.1,
            0.0,
            0.0,
            400.0,
            300.0,
        ).unwrap();

        || ()
    });

    let onmousedown = callback!(clicked; move |ev: MouseEvent| {
        let new = (ev.offset_x() as f64, ev.offset_y() as f64);
        clicked.set(Some(new));
    });
    let onmouseup = callback!(clicked; move |_: MouseEvent| clicked.set(None));
    let onmouseout = callback!(clicked; move |_: MouseEvent| clicked.set(None));

    let onmousemove = callback!(canvas, image, position, clicked, zoom; move |ev: MouseEvent| {
        let absolute = (ev.offset_x() as f64, ev.offset_y() as f64);
        if let Some((start_x, start_y)) = *clicked {
            let new = (position.0 - absolute.0 + start_x, position.1 - absolute.1 + start_y);

            let CenterImage{offset, scale, ..} = center_image(&canvas.cast().unwrap(), &*image.clone(), zoom);
            let pos = constrain_position(new, offset);

            clicked.set(Some(absolute));
            position.set(pos);

            log::warn!("{offset:?}");
            let bb = bounding_box(&*image.clone(), pos, scale, zoom);
            bounding.set(bb);
            callback.emit(bb);
        }
    });

    html! {
        <>
        <canvas width={width.to_string()} height={height.to_string()} ref={canvas} style="border:1px" {onmousedown} {onmouseup} {onmousemove} {onmouseout}/>
        <canvas width={width.to_string()} height={height.to_string()} ref={preview}/>
        </>
    }
}