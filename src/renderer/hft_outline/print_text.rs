//! 실제 FontFace의 주 프로그램과 완전한 CFF 결측만 Print 기하에 사용한다.

use super::{face_contour, first_family, HftOutlineFonts, PathCommand, SourceFace, TextStyle};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use ttf_parser::Face;

#[derive(Clone)]
pub(super) struct PrintPrimary {
    source: Arc<SourceFace>,
    complete: bool,
}

impl HftOutlineFonts {
    pub(crate) fn clear_print_primary(&mut self, family: &str) {
        let family = first_family(family).to_lowercase();
        self.print_primaries.remove(&family);
        self.print_complete_cff_families.remove(&family);
    }

    /// 로드한 주 FontFace만 등록한다. complete는 같은 스타일의 뒤 후보도 없다는 증거다.
    /// CFF는 기존에 확인한 전체 조각과 같은 첫 프로그램일 때만 완전성을 기록한다.
    pub(crate) fn register_print_primary(
        &mut self,
        family: &str,
        bytes: &[u8],
        index: u32,
        complete: bool,
    ) {
        if first_family(family).is_empty()
            || family.len() > 256
            || family.chars().any(char::is_control)
        {
            return;
        }
        if bytes.is_empty() || bytes.len() > super::MAX_SOURCE_BYTES {
            self.clear_print_primary(family);
            return;
        }
        self.register_print_font(family, bytes, index);
        let family = first_family(family).to_lowercase();
        let digest: [u8; 32] = Sha256::digest(bytes).into();
        if let Some(source) = self.print_faces.iter().find_map(|((name, _), source)| {
            (name == &family && source.digest == digest && source.face_index == index)
                .then(|| source.clone())
        }) {
            self.print_primaries
                .insert(family, PrintPrimary { source, complete });
        } else if complete
            && self.faces.get(&family).is_some_and(|faces| {
                faces
                    .first()
                    .is_some_and(|source| source.digest == digest && source.face_index == index)
            })
        {
            self.print_complete_cff_families.insert(family);
        }
    }

    /// 실제 CSS 사다리의 확인된 주 glyph만 재생한다. 미확인 후보를 건너뛰지 않는다.
    pub(crate) fn print_primary_glyph(
        &self,
        style: &TextStyle,
        character: char,
    ) -> Option<Arc<[PathCommand]>> {
        if style.is_visually_bold()
            || style.italic
            || style.outline_type != 0
            || style.shadow_type != 0
            || style.emboss
            || style.engrave
            || style.text_paint.is_some()
        {
            return None;
        }
        let families = crate::renderer::canvas_text_font::family_chain(style);
        for name in crate::renderer::internal_font_family_members(&families) {
            let family = name.to_lowercase();
            if let Some(primary) = self.print_primaries.get(&family) {
                let source = &primary.source;
                let face = Face::parse(&source.bytes, source.face_index).ok()?;
                if let Some(glyph) = face.glyph_index(character) {
                    // 빈 glyph나 지원하지 않는 윤곽도 현재 face가 소유한다.
                    // 윤곽 재생에 실패했다고 다음 family를 대신 그리지 않는다.
                    return self
                        .cache
                        .lock()
                        .ok()?
                        .get_or_build((source.digest, source.face_index, glyph.0), || {
                            face_contour(&face, glyph)
                        });
                }
                if !primary.complete {
                    return None;
                }
            } else if self.print_complete_cff_families.contains(&family) {
                let faces = self.faces.get(&family)?;
                for source in faces {
                    let face = Face::parse(&source.bytes, source.face_index).ok()?;
                    if face.glyph_index(character).is_some() {
                        return None;
                    }
                }
            } else {
                return None;
            }
        }
        None
    }
}
