mod modules;
use image::DynamicImage::ImageRgba8;
use modules::*;
use std::fs::read_to_string;
use std::path::Path;

fn main() {
    let read_source = |filename: &str| read_to_string(filename).expect("Failed to read file");

    let html = read_source("tests/rainbow.html");
    let css = read_source("tests/empty.css");

    let initial_containing_block = layout::Dimensions {
        content: layout::Rect {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        },
        padding: Default::default(),
        border: Default::default(),
        margin: Default::default(),
    };

    let root_node = parser::parse_html(html);
    let stylesheet = parser::parse_css(css, &root_node);
    let style_root = style::style_tree(&root_node, &stylesheet);
    let layout_root = layout::layout_tree(&style_root, initial_containing_block);
    let canvas = display::paint(&layout_root, initial_containing_block.content);

    let filename = "output.png";

    let (w, h) = (canvas.width as u32, canvas.height as u32);
    let buffer: Vec<image::Rgba<u8>> = unsafe { std::mem::transmute(canvas.pixels) };
    let img = image::ImageBuffer::from_fn(
        w,
        h,
        Box::new(|x: u32, y: u32| buffer[(y * w + x) as usize]),
    );

    let _ = ImageRgba8(img).save(Path::new(filename));
}
