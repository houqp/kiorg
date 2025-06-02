// Take from https://github.com/emilk/egui/discussions/1344#discussioncomment-11919481

use std::collections::HashMap;
use std::fs::read;
use std::sync::Arc;

use eframe::epaint::text::FontFamily;
use egui::{FontData, FontDefinitions};
use font_kit::{
    family_name::FamilyName, handle::Handle, properties::Properties, source::SystemSource,
};
use tracing::debug;

/// Attempt to load a system font by any of the given `family_names`, returning the first match.
fn load_font_family(family_names: &[&str]) -> Option<Vec<u8>> {
    let system_source = SystemSource::new();
    for &name in family_names {
        let font_handle = system_source
            .select_best_match(&[FamilyName::Title(name.to_string())], &Properties::new());
        match font_handle {
            Ok(h) => match &h {
                Handle::Memory { bytes, .. } => {
                    debug!("Loaded {name} from memory.");
                    return Some(bytes.to_vec());
                }
                Handle::Path { path, .. } => {
                    debug!("Loaded {name} from path: {:?}", path);
                    if let Ok(data) = read(path) {
                        return Some(data);
                    }
                }
            },
            Err(e) => debug!("Could not load {}: {:?}", name, e),
        }
    }
    None
}

pub fn load_system_fonts(mut fonts: FontDefinitions) -> FontDefinitions {
    debug!("Attempting to load system fonts");
    let mut fontdb = HashMap::new();

    // load system front to render non-breaking spaces
    #[cfg(target_os = "macos")]
    fontdb.insert("system", vec!["Lucida Grande"]);

    fontdb.insert(
        "simplified_chinese",
        vec![
            "Heiti SC",
            "Songti SC",
            "Noto Sans CJK SC", // Good coverage for Simplified Chinese
            "Noto Sans SC",
            "WenQuanYi Zen Hei", // INcludes both Simplified and Traditional Chinese.
            "SimSun",
            "Noto Sans SC",
            "PingFang SC",
            "Source Han Sans CN",
        ],
    );

    fontdb.insert("traditional_chinese", vec!["Source Han Sans HK"]);

    fontdb.insert(
        "japanese",
        vec![
            "Noto Sans JP",
            "Noto Sans CJK JP",
            "Source Han Sans JP",
            "MS Gothic",
        ],
    );

    fontdb.insert("korean", vec!["Source Han Sans KR"]);

    fontdb.insert("taiwanese", vec!["Source Han Sans TW"]);

    fontdb.insert(
        "arabic_fonts",
        vec![
            "Noto Sans Arabic",
            "Amiri",
            "Lateef",
            "Al Tarikh",
            "Segoe UI",
        ],
    );

    for (region, font_names) in fontdb {
        if let Some(font_data) = load_font_family(&font_names) {
            debug!("Inserting font {region}");
            fonts
                .font_data
                .insert(region.to_owned(), Arc::new(FontData::from_owned(font_data)));

            fonts
                .families
                .get_mut(&FontFamily::Proportional)
                .unwrap()
                .push(region.to_owned());
        } else {
            debug!("Could not load a font for region {region}. If you experience incorrect file names, try installing one of these fonts: [{}]", font_names.join(", "));
        }
    }

    fonts
}
