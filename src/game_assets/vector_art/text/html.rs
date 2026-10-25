//! The HTML subset Flash stores in text fields: one paragraph with an alignment and one font tag
//! with a size and color.

use swf::{EditText, TextAlign};

use super::TWIPS_PER_PIXEL;

const DEFAULT_SIZE_PIXELS: f32 = 12.0;

/// What a field shows: its text, size in pixels, color and alignment.
pub struct Paragraph {
    pub text: String,
    pub size_pixels: f32,
    pub color: swf::Color,
    pub align: TextAlign,
}

pub fn paragraph_of(edit_text: &EditText) -> Paragraph {
    let raw = edit_text
        .initial_text()
        .map(|text| text.to_str_lossy(swf::UTF_8).into_owned())
        .unwrap_or_default();
    let plain = Paragraph {
        text: raw.clone(),
        size_pixels: edit_text
            .height()
            .map_or(DEFAULT_SIZE_PIXELS, |height| height.get() as f32 / TWIPS_PER_PIXEL),
        color: edit_text.color().unwrap_or(swf::Color::BLACK),
        align: edit_text.layout().map_or(TextAlign::Left, |layout| layout.align),
    };
    if !edit_text.is_html() {
        return plain;
    }
    Paragraph {
        text: html_text(&raw),
        size_pixels: attribute(&raw, "size")
            .and_then(|size| size.parse().ok())
            .unwrap_or(plain.size_pixels),
        color: attribute(&raw, "color")
            .and_then(|color| hex_color(&color))
            .unwrap_or(plain.color),
        align: match attribute(&raw, "align").as_deref() {
            Some("center") => TextAlign::Center,
            Some("right") => TextAlign::Right,
            Some("left") => TextAlign::Left,
            _ => plain.align,
        },
    }
}

/// The first `name="value"` in the markup.
fn attribute(html: &str, name: &str) -> Option<String> {
    let start = html.find(&format!("{name}=\""))? + name.len() + 2;
    let end = html[start..].find('"')? + start;
    Some(html[start..end].to_string())
}

/// The markup's text without tags and with the common entities decoded.
fn html_text(html: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    for character in html.chars() {
        match character {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(character),
            _ => {}
        }
    }
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
}

fn hex_color(value: &str) -> Option<swf::Color> {
    let digits = u32::from_str_radix(value.trim_start_matches('#'), 16).ok()?;
    let [_, red, green, blue] = digits.to_be_bytes();
    Some(swf::Color {
        r: red,
        g: green,
        b: blue,
        a: u8::MAX,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_html_subset_flash_writes() {
        let html =
            r##"<p align="center"><font face="Upheaval TT BRK" size="18" color="#ff0000">x99 &amp; co</font></p>"##;
        assert_eq!(html_text(html), "x99 & co");
        assert_eq!(attribute(html, "size").as_deref(), Some("18"));
        assert_eq!(
            hex_color("#ff0000"),
            Some(swf::Color {
                r: 255,
                g: 0,
                b: 0,
                a: 255
            })
        );
    }
}
