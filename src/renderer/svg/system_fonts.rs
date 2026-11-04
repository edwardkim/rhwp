//! 시스템 글꼴은 파일명 대신 내부 family·굵기로 조회한다.

use std::path::PathBuf;
use std::sync::OnceLock;
use usvg::fontdb::{Database, Family, Query, Source, Style, Weight};

fn system_fonts() -> &'static Database {
    static FONTS: OnceLock<Database> = OnceLock::new();
    FONTS.get_or_init(|| {
        let mut fonts = Database::new();
        fonts.load_system_fonts();
        fonts
    })
}

pub(super) fn family_file(names: &[String], bold: bool) -> Option<PathBuf> {
    family_file_for_chars(names, bold, &std::collections::HashSet::new())
}

pub(super) fn family_file_for_chars(
    names: &[String],
    bold: bool,
    chars: &std::collections::HashSet<char>,
) -> Option<PathBuf> {
    family_file_from_database(system_fonts(), names, bold, chars)
}

pub(super) fn family_file_from_database(
    fonts: &Database,
    names: &[String],
    bold: bool,
    chars: &std::collections::HashSet<char>,
) -> Option<PathBuf> {
    family_file_from_database_with_style(fonts, names, bold, chars, Style::Normal)
}

pub(super) fn italic_family_file(
    extra_paths: &[PathBuf],
    names: &[String],
    bold: bool,
    chars: &std::collections::HashSet<char>,
) -> Option<PathBuf> {
    let mut supplied = Database::new();
    for path in extra_paths {
        if path.is_dir() {
            supplied.load_fonts_dir(path);
        } else if path.is_file() {
            let _ = supplied.load_font_file(path);
        }
    }
    family_file_from_database_with_style(&supplied, names, bold, chars, Style::Italic).or_else(
        || family_file_from_database_with_style(system_fonts(), names, bold, chars, Style::Italic),
    )
}

fn family_file_from_database_with_style(
    fonts: &Database,
    names: &[String],
    bold: bool,
    chars: &std::collections::HashSet<char>,
    style: Style,
) -> Option<PathBuf> {
    // family 별 우선순위는 호출자가 정한다. generic 폴백을 요청하지 않아
    // 없는 이름을 무관한 시스템 글꼴로 잘못 내장하지 않는다.
    for name in names {
        let families = [Family::Name(name)];
        let Some(id) = fonts.query(&Query {
            families: &families,
            weight: if bold { Weight::BOLD } else { Weight::NORMAL },
            style,
            ..Default::default()
        }) else {
            continue;
        };
        let preferred = fonts.face(id)?;
        // Regular만 있는 family를 Bold face로 선언하지 않는다.
        // TTC의 face 선택은 기존 파일 임베딩 계약 밖이므로 여기서 확대하지 않는다.
        if (bold && preferred.weight < Weight::SEMIBOLD)
            || preferred.index != 0
            || (style != Style::Normal && preferred.style == Style::Normal)
        {
            continue;
        }
        // 같은 family의 부분 글꼴이 먼저 등록되어도 실제 표시 문자를 모두 공급하는
        // 같은 스타일의 설치 face를 찾는다. 다른 family나 굵기로 바꾸지 않는다.
        let candidates = std::iter::once(preferred).chain(fonts.faces().filter(|face| {
            face.id != id
                && face.index == 0
                && face.weight == preferred.weight
                && face.style == preferred.style
                && face.stretch == preferred.stretch
                && face
                    .families
                    .iter()
                    .any(|(family, _)| family.eq_ignore_ascii_case(name))
        }));
        for face in candidates {
            let covered = chars.is_empty()
                || fonts
                    .with_face_data(face.id, |data, index| {
                        ttf_parser::Face::parse(data, index).is_ok_and(|parsed| {
                            chars
                                .iter()
                                .filter(|ch| !ch.is_whitespace() && !ch.is_control())
                                .all(|ch| parsed.glyph_index(*ch).is_some_and(|glyph| glyph.0 != 0))
                        })
                    })
                    .unwrap_or(false);
            if !covered {
                continue;
            }
            match &face.source {
                Source::File(path) | Source::SharedFile(path, _) if path.is_file() => {
                    return Some(path.clone());
                }
                _ => {}
            }
        }
    }
    None
}
