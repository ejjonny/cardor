use std::{env, fs::File, io::Write};
use image::*;
use kurbo::*;

fn main() {
    env::set_var("RUST_BACKTRACE", "full");
    let columns = 24;
    let rows = 22;
    let punch_radius = 0.3 / 2.;
    let row_height = 0.5;
    let column_width = 0.45;
    let leading_padding = 1.25;
    let track_padding = 0.5;
    let trailing_padding = 1.25;
    let track_punch_radius = 0.1;
    // let full_card_width = 14.25;
    let selection =
        tinyfiledialogs::open_file_dialog("Open PNG", "", Some((&["*.png", "*.PNG"], "")));
    let file_path = if let Some(path) = selection {
        path
    } else {
        tinyfiledialogs::message_box_ok(
            "No file selected",
            "",
            tinyfiledialogs::MessageBoxIcon::Error,
        );
        std::process::exit(1)
    };
    let img = image::open(file_path).unwrap();
    if img.dimensions().0 != columns || img.dimensions().1 != rows {
        tinyfiledialogs::message_box_ok(
            format!(
                "The image should be {width} pixels wide & {height} pixels tall",
                width = columns,
                height = rows
            )
            .as_str(),
            "",
            tinyfiledialogs::MessageBoxIcon::Error,
        );
        std::process::exit(1)
    }
    let mut values = Vec::<Vec<bool>>::new();
    for _ in 0..=columns {
        values.push(vec![false; (rows + 1) as usize])
    }
    img.pixels().for_each(|(x, y, color)| {
        let x = x as usize;
        let y = y as usize;
        if color.0 == [0, 0, 0, 255] {
            values[x][y] = false;
        } else if color.0 == [255, 255, 255, 255] {
            values[x][y] = true;
        } else {
            tinyfiledialogs::message_box_ok(
                "The image should only contain black and white",
                "",
                tinyfiledialogs::MessageBoxIcon::Error,
            );
            std::process::exit(1)
        }
    });

    let mut punches = Vec::<Circle>::new();
    for row in 0..rows {
        let row_y = row_height + (row_height * row as f64);
        let left_track_punch = Circle::new(Point::new(leading_padding, row_y), track_punch_radius);
        let right_track_punch = Circle::new(
            Point::new(
                leading_padding + track_padding + column_width * (columns as f64),
                row_y,
            ),
            track_punch_radius,
        );
        punches.push(left_track_punch);
        punches.push(right_track_punch);
    }

    for column in 0..columns {
        for row in 0..rows {
            if !values[column as usize][row as usize] {
                let punch_x = leading_padding + track_padding + (column_width * column as f64);
                let punch_y = row_height + (row_height * row as f64);
                let punch = Circle::new(Point::new(punch_x, punch_y), punch_radius);
                punches.push(punch);
            }
        }
    }
    let tolerance = 0.00001;
    let elements: Vec<PathEl> = punches
        .iter()
        .map(|punch| punch.path_elements(tolerance))
        .flat_map(|i| i)
        .collect();
    // let card_rect: Vec<PathEl> =
    //     Rect::new(0., 0., full_card_width, (row_height * (rows + 1) as f64))
    //         .path_elements(tolerance)
    //         .collect();
    // let card_path = BezPath::from_vec(card_rect);
    let path = kurbo::BezPath::from_vec(elements);
    let card_width =
        leading_padding + track_padding + ((column_width) * columns as f64) + trailing_padding;
    let card_height = row_height * (rows + 1) as f64;
    let svg = path.to_svg();
    let mut svg_document = String::new();
    svg_document.push_str(
        "
        <!DOCTYPE html>
        <html>
        <body>
        ",
    );
    let svg_bounds = format!(
        "<svg viewBox=\"0 0 {} {h}\" xmlns=\"http://www.w3.org/2000/svg\">",
        card_width,
        h = card_height
    );
    svg_document.push_str(svg_bounds.as_str());
    let path_string = format!("<path d=\"{}\" stroke=\"none\" fill=\"black\" />", svg);
    svg_document.push_str(path_string.as_str());
    // let card_path_string = format!(
    //     "<path d=\"{}\" stroke=\"black\" fill=\"none\" stroke-width=\"0.05\"/>",
    //     card_path.to_svg()
    // );
    // svg_document.push_str(card_path_string.as_str());
    svg_document.push_str(
        "
        </svg>
        </body>
        </html>
        ",
    );
    let output_file_path = tinyfiledialogs::save_file_dialog("Save SVG", "card.svg");
    let file_creation = File::create(output_file_path.unwrap());
    let write = if let Ok(mut file) = file_creation {
        file.write(svg_document.as_bytes())
    } else {
        tinyfiledialogs::message_box_ok(
            "File creation failed",
            "",
            tinyfiledialogs::MessageBoxIcon::Error,
        );
        std::process::exit(1)
    };
    match write {
        Err(error) => {
            tinyfiledialogs::message_box_ok(
                format!("File write failed {}", error).as_str(),
                "",
                tinyfiledialogs::MessageBoxIcon::Error,
            );
            std::process::exit(1)
        }
        Ok(..) => {
            tinyfiledialogs::message_box_ok("All done!", "", tinyfiledialogs::MessageBoxIcon::Info);
        }
    }
}
