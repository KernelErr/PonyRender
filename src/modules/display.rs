use super::css::{Color, Value};
use super::layout::{BoxType, LayoutBox, Rect};

pub fn paint(layout_root: &LayoutBox, bounds: Rect) -> Canvas {
    let display_list = build_display_list(layout_root);
    let mut canvas = Canvas::new(bounds.width as usize, bounds.height as usize);
    for item in display_list {
        canvas.paint_item(&item);
    }
    canvas
}

type DisplayList = Vec<DisplayCommand>;

#[derive(Debug)]
enum DisplayCommand {
    SolidColor(Color, Rect),
}

fn build_display_list(layout_root: &LayoutBox) -> DisplayList {
    let mut list = Vec::new();
    render_layout_box(&mut list, layout_root);
    list
}

fn render_layout_box(list: &mut DisplayList, layout_box: &LayoutBox) {
    render_background(list, layout_box);
    render_borders(list, layout_box);

    for child in &layout_box.children {
        render_layout_box(list, child);
    }
}

fn render_background(list: &mut DisplayList, layout_box: &LayoutBox) {
    if let Some(color) = get_color(layout_box, "background") {
        list.push(DisplayCommand::SolidColor(
            color,
            layout_box.clone().dimensions.border_box(),
        ))
    }
}

fn get_color(layout_box: &LayoutBox, name: &str) -> Option<Color> {
    match layout_box.box_type {
        BoxType::BlockNode(style) | BoxType::InlineNode(style) => match style.value(name) {
            Some(Value::Color(color)) => Some(color),
            _ => None,
        },
        BoxType::AnonymousBlock => None,
    }
}

fn render_borders(list: &mut DisplayList, layout_box: &LayoutBox) {
    let color = match get_color(layout_box, "border-color") {
        Some(color) => color,
        _ => return, // bail out if no border-color is specified
    };

    let d = &layout_box.dimensions;
    let border_box = (*d).border_box();

    // Left border
    list.push(DisplayCommand::SolidColor(
        color.clone(),
        Rect {
            x: border_box.x,
            y: border_box.y,
            width: d.border.left,
            height: border_box.height,
        },
    ));

    // Right border
    list.push(DisplayCommand::SolidColor(
        color.clone(),
        Rect {
            x: border_box.x + border_box.width - d.border.right,
            y: border_box.y,
            width: d.border.right,
            height: border_box.height,
        },
    ));

    // Top border
    list.push(DisplayCommand::SolidColor(
        color.clone(),
        Rect {
            x: border_box.x,
            y: border_box.y,
            width: border_box.width,
            height: d.border.top,
        },
    ));

    // Bottom border
    list.push(DisplayCommand::SolidColor(
        color,
        Rect {
            x: border_box.x,
            y: border_box.y + border_box.height - d.border.bottom,
            width: border_box.width,
            height: d.border.bottom,
        },
    ));
}

#[derive(Debug)]
pub struct Canvas {
    pub pixels: Vec<Color>,
    pub width: usize,
    pub height: usize,
}

impl Canvas {
    fn new(width: usize, height: usize) -> Canvas {
        let white = Color {
            red: 255,
            green: 255,
            blue: 255,
            alpha: 255,
        };
        Canvas {
            pixels: vec![white; width * height],
            width,
            height,
        }
    }

    fn paint_item(&mut self, item: &DisplayCommand) {
        match item {
            DisplayCommand::SolidColor(color, rect) => {
                // Clip the rectangle to the canvas boundaries.
                let x0 = rect.x.clamp(0.0, self.width as f32) as usize;
                let y0 = rect.y.clamp(0.0, self.height as f32) as usize;
                let x1 = (rect.x + rect.width).clamp(0.0, self.width as f32) as usize;
                let y1 = (rect.y + rect.height).clamp(0.0, self.height as f32) as usize;

                for y in y0..y1 {
                    for x in x0..x1 {
                        // TODO: alpha compositing with existing pixel
                        self.pixels[y * self.width + x] = color.clone();
                    }
                }
            }
        }
    }
}
