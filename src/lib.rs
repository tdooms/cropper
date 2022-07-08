use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};
use wasm_bindgen::JsCast;
use yew::*;
use std::f64::consts::PI;
use std::rc::Rc;
use cobul::*;
use log::trace;

macro_rules! callback {
    ( $( $x:ident ),*; $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            Callback::from($y)
        }
    };
}

struct CenterImage {
    offset: (f64, f64),
    dims: (f64, f64),
    scale: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Dimensions {
    pub image_dims: (f64, f64),
    pub canvas_dims: (f64, f64),
    pub borders: (f64, f64),
    pub ratio: f64,
}

impl Dimensions {
    pub fn new(canvas: &HtmlCanvasElement, image: &HtmlImageElement) -> Self {
        let image_dims = (image.width() as f64, image.height() as f64);
        let canvas_dims = (canvas.width() as f64, canvas.height() as f64);
        let borders = (canvas_dims.0 / 20.0, canvas_dims.1 / 20.0);
        let ratio = canvas_dims.0 / canvas_dims.1;

        Self { image_dims, canvas_dims, borders, ratio }
    }
}

fn center_image(dims: Dimensions, zoom: f64) -> CenterImage {
    let Dimensions { image_dims: (i_width, i_height), canvas_dims: (c_width, c_height), borders, .. } = dims;

    let (l_width, l_height) = (c_width - 2.0 * borders.0, c_height - 2.0 * borders.1);
    let scale = (l_width / i_width).max(l_height / i_height) * zoom;

    let dims = (i_width * scale, i_height * scale);
    let offset = ((c_width - dims.0) / 2.0, (c_height - dims.1) / 2.0);

    CenterImage { dims, offset, scale }
}

fn constrain_position(dims: Dimensions, (pos_x, pos_y): (f64, f64), (off_x, off_y): (f64, f64)) -> (f64, f64) {
    let (x_win, y_win) = (dims.borders.0 - off_x, dims.borders.1 - off_y);

    let new_x = if x_win != 0.0 { pos_x.clamp(-x_win.abs(), x_win.abs()) } else { 0.0 };
    let new_y = if y_win != 0.0 { pos_y.clamp(-y_win.abs(), y_win.abs()) } else { 0.0 };

    (new_x, new_y)
}

fn bounding_box(dims: Dimensions, (pos_x, pos_y): (f64, f64), scale: f64, zoom: f64) -> ((f64, f64), (f64, f64)) {
    let Dimensions { image_dims: (width, height), ratio, .. } = dims;

    let factor = height.min(width / ratio);
    let dims = (ratio * factor / zoom, factor / zoom);

    let (x_center, y_center) = (pos_x / scale + width / 2.0, pos_y / scale + height / 2.0);

    let x_corner = x_center - 4.0 / 3.0 * factor / (2.0 * zoom);
    let y_corner = y_center - factor / (2.0 * zoom);

    ((x_corner, y_corner), dims)
}

fn draw(canvas: HtmlCanvasElement, image: HtmlImageElement, zoom: f64, pos: (f64, f64), radius: f64) {
    let element = canvas.get_context("2d").unwrap().unwrap();
    let context = element.dyn_into::<CanvasRenderingContext2d>().unwrap();

    let dimensions = Dimensions::new(&canvas, &image);
    let (v_border, h_border) = dimensions.borders;

    let CenterImage { dims, offset, scale } = center_image(dimensions, zoom);
    let (width, height) = (canvas.width() as f64, canvas.height() as f64);

    context.set_image_smoothing_enabled(true);
    context.clear_rect(0.0, 0.0, width, height);
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
    context.move_to(v_border, h_border + radius); // start top-left
    context.arc(v_border + radius, h_border + radius, radius, PI, 1.5 * PI).unwrap();
    context.line_to(width - v_border - radius, h_border); // top-right of image
    context.arc(width - v_border - radius, h_border + radius, radius, 1.5 * PI, 0.0).unwrap();
    context.line_to(width - v_border, height - h_border - radius); // bottom-right of image
    context.arc(width - v_border - radius, height - h_border - radius, radius, 0.0, 0.5 * PI).unwrap();
    context.line_to(v_border + radius, height - h_border); // bottom-left of image
    context.arc(v_border + radius, height - h_border - radius, radius, 0.5 * PI, PI).unwrap();
    context.close_path();

    context.rect(width, 0.0, -width, height);
    context.fill();
}

#[derive(Properties, PartialEq, Clone, Debug)]
pub struct Props {
    pub width: u64,
    pub height: u64,

    #[prop_or(3.0)]
    pub max_zoom: f64,

    #[prop_or(30.0)]
    pub radius: f64,

    pub src: Rc<String>,

    pub ondone: Callback<String>,
    pub oncancel: Callback<()>,
}

#[function_component(Cropper)]
pub fn cropper(props: &Props) -> Html {
    log::warn!("rendering component with {props:?}");
    let Props { width, height, max_zoom, src, radius, ondone, oncancel } = props.clone();

    let zoom = use_state_eq(|| 1.0);
    let position = use_state_eq(|| (0.0, 0.0));
    let clicked = use_state_eq(|| None);
    let loaded = use_state_eq(|| false);

    let image = use_node_ref();
    let canvas = use_node_ref();

    let (canvas_c, image_c) = (canvas.clone(), image.clone());
    use_effect_with_deps(move |(position, loaded, zoom)| {
        let (canvas, image) = (canvas_c.cast().unwrap(), image_c.cast().unwrap());

        if **loaded {
            log::error!("drawing {position:?} {zoom:?}");
            draw(canvas, image, **zoom, **position, radius);
        }
        || ()
    }, (position.clone(), loaded.clone(), zoom.clone()));

    let onload = callback!(loaded; move |_| loaded.set(true));

    let onchange = callback!(zoom, position, canvas, image; move |value| {
        let dims = Dimensions::new(&canvas.cast().unwrap(), &image.cast().unwrap());
        let CenterImage{offset, ..} = center_image(dims, *zoom);
        let pos = constrain_position(dims, *position, offset);

        position.set(pos);
        zoom.set(value);
    });
    let onmouseup = callback!(clicked; move |_: MouseEvent| clicked.set(None));
    let onmouseout = callback!(clicked; move |_: MouseEvent| clicked.set(None));

    let onmousedown = callback!(clicked; move |ev: MouseEvent| {
        let new = (ev.offset_x() as f64, ev.offset_y() as f64);
        clicked.set(Some(new));
    });
    let onmousemove = callback!(canvas, image, position, clicked, zoom; move |ev: MouseEvent| {
        let absolute = (ev.offset_x() as f64, ev.offset_y() as f64);
        if let Some((start_x, start_y)) = *clicked {
            let new = (position.0 - absolute.0 + start_x, position.1 - absolute.1 + start_y);

            let dims = Dimensions::new(&canvas.cast().unwrap(), &image.cast().unwrap());
            let CenterImage{offset, ..} = center_image(dims, *zoom);
            let pos = constrain_position(dims, new, offset);

            clicked.set(Some(absolute));
            position.set(pos);
        }
    });

    let onclose = oncancel.reform(|_| ());

    let ondone = callback!(image, canvas, position, zoom; move |_| {
        let canvas: HtmlCanvasElement = canvas.cast().unwrap();
        let image: HtmlImageElement = image.cast().unwrap();

        let element = canvas.get_context("2d").unwrap().unwrap();
        let context = element.dyn_into::<CanvasRenderingContext2d>().unwrap();

        let dims = Dimensions::new(&canvas, &image);
        let CenterImage{scale, ..} = center_image(dims, *zoom);
        let bb = bounding_box(dims, *position, scale, *zoom);

        context.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
        context.set_global_alpha(1.0);
        context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
            &image,
            bb.0.0,
            bb.0.1,
            bb.1.0,
            bb.1.1,
            0.0,
            0.0,
            canvas.width() as f64,
            canvas.height() as f64,
        ).unwrap();

        ondone.emit(canvas.to_data_url().unwrap())
    });

    let footer = html! {
        <Buttons>
        <Button onclick={onclose.clone()}> <span> {"Cancel"} </span> </Button>
        <Button color={Color::Primary} onclick={ondone}> <span> {"Save"} </span> </Button>
        </Buttons>
    };

    html! {
        <>
        <ModalCard title="Crop your image" active=true {footer} {onclose}>
            <img style="display:none" src={(*src).clone()} {onload} ref={image} />
            <canvas width={width.to_string()} height={height.to_string()} ref={canvas} style="border:1px" {onmousedown} {onmouseup} {onmousemove} {onmouseout}/>
            <Slider<f64> range={1.0..max_zoom} value={*zoom} steps=50 {onchange}/>
        </ModalCard>
        </>
    }
}