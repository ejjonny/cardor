use std::{fs::File, io::Write};

use image::*;
use kurbo::*;
use native_dialog::*;

fn main() {
    let columns = 24;
    let rows = 100;
    let punch_radius = 0.3 / 2.;
    let row_height = 0.5;
    let column_width = 0.45;
    let leading_padding = 1.25;
    let track_padding = 0.5;
    let trailing_padding = 1.25;
    let track_punch_radius = 0.1;
    let full_card_width = 14.25;
    let path = FileDialog::new()
        .set_location("~/Downloads")
        .add_filter("PNG Image", &["png"])
        .show_open_single_file()
        .unwrap();
    println!("{:?}", path);
    let file_path = if let Some(path) = path {
        path
    } else {
        MessageDialog::new()
            .set_type(MessageType::Error)
            .set_title("No file selected")
            .show_alert()
            .unwrap();
        std::process::exit(1)
    };
    if !file_path.exists() {
        MessageDialog::new()
            .set_type(MessageType::Error)
            .set_title("File doesn't exist! How did that happen?")
            .show_alert()
            .unwrap();
        std::process::exit(1)
    }
    let img = image::open(file_path.as_path()).unwrap();
    if img.dimensions().0 != columns || img.dimensions().1 != rows {
        MessageDialog::new()
            .set_type(MessageType::Error)
            .set_title("The image doesn't have the right dimensions")
            .set_text(format!("Expected width: {}, height: {y}", columns, y = rows).as_str())
            .show_alert()
            .unwrap();
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
            MessageDialog::new()
                .set_type(MessageType::Error)
                .set_title("The image needs to have only black and white")
                .show_confirm()
                .unwrap();
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
    // elements.extend(
    //     Rect::new(0., 0., full_card_width, (row_height * (rows + 1) as f64))
    //         .path_elements(tolerance),
    // );
    let card_rect: Vec<PathEl> = Rect::new(0., 0., full_card_width, (row_height * (rows + 1) as f64))
        .path_elements(tolerance)
        .collect();
    let card_path = BezPath::from_vec(card_rect);
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
    let card_path_string = format!("<path d=\"{}\" stroke=\"black\" fill=\"none\" stroke-width=\"0.05\"/>", card_path.to_svg());
    svg_document.push_str(card_path_string.as_str());
    svg_document.push_str(
        "
        </svg>
        </body>
        </html>
        ",
    );
    let output_file_path = FileDialog::new()
        // .set_location("~/Downloads")
        .set_filename("card.svg")
        .show_save_single_file()
        .unwrap();
    let file_creation = File::create(output_file_path.unwrap());
    let write = if let Ok(mut file) = file_creation {
        file.write(svg_document.as_bytes())
    } else {
        MessageDialog::new()
            .set_type(MessageType::Error)
            .set_title("File creation failed")
            .show_alert()
            .unwrap();
        std::process::exit(1)
    };
    match write {
        Err(error) => {
            MessageDialog::new()
                .set_type(MessageType::Error)
                .set_title("File write failed")
                .set_text(format!("Error: {}", error).as_str())
                .show_alert()
                .unwrap();
            std::process::exit(1)
        }
        Ok(..) => {
            MessageDialog::new()
                .set_type(MessageType::Info)
                .set_title("All done!")
                .show_alert()
                .unwrap();
        }
    }
}
