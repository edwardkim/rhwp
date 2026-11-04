//! rhwp가 저장한 HWP5/HWPX의 원 글꼴 프로그램 출처를 보존한다.
//! 표준 글꼴 선언이나 자간에 합성 값을 쓰지 않는다. 외부 편집으로 DocInfo/header가
//! 바뀌거나 보조 엔트리가 제거되면 출처를 추정해 복원하지 않는다.

use super::{document::DocInfo, style::NativeFontProgram};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub(crate) const HWPX_ENTRY: &str = "META-INF/rhwp-native-font-programs.json";
pub(crate) const HWP_STREAM: &str = "/RhwpNativeFontPrograms";
pub(crate) const MAX_BYTES: usize = 1024 * 1024;

#[derive(Serialize, Deserialize)]
struct Envelope {
    version: u8,
    header_sha256: Vec<u8>,
    programs: Vec<Vec<Option<NativeFontProgram>>>,
}

pub(crate) fn encode_with_storage<S: crate::storage::Storage>(
    info: &DocInfo,
    header: &[u8],
    storage: &S,
) -> Result<Option<Vec<u8>>, String>
where
    S::Error: std::fmt::Display,
{
    if !info
        .font_faces
        .iter()
        .flatten()
        .any(|f| f.native_program.is_some())
    {
        return Ok(None);
    }
    let envelope = Envelope {
        version: 1,
        header_sha256: storage
            .copy_slice(Sha256::digest(header).as_slice())
            .map_err(|e| e.to_string())?,
        // HWPX header always emits seven language slots, including empty slots.
        programs: storage
            .try_collect((0..7).map(|i| {
                storage.collect(
                    info.font_faces
                        .get(i)
                        .into_iter()
                        .flatten()
                        .map(|f| f.native_program),
                )
            }))
            .map_err(|e| e.to_string())?,
    };
    let mut writer = crate::serializer::budget_cursor::BudgetCursor::new(storage, usize::MAX);
    serde_json::to_writer(&mut writer, &envelope).map_err(|e| e.to_string())?;
    Ok(Some(writer.into_inner()))
}

pub(crate) fn decode(info: &mut DocInfo, header: &[u8], bytes: &[u8]) {
    if bytes.len() > MAX_BYTES {
        return;
    }
    let Ok(envelope) = serde_json::from_slice::<Envelope>(bytes) else {
        return;
    };
    if envelope.version != 1
        || envelope.header_sha256 != Sha256::digest(header).as_slice()
        || envelope.programs.len() != info.font_faces.len()
        || envelope
            .programs
            .iter()
            .zip(&info.font_faces)
            .any(|(p, f)| p.len() != f.len())
    {
        return;
    }
    for (fonts, programs) in info.font_faces.iter_mut().zip(envelope.programs) {
        for (font, program) in fonts.iter_mut().zip(programs) {
            if !font.is_embedded {
                font.native_program = program;
            }
        }
    }
}
