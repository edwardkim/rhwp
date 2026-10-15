//! #7661: 한컴 교차 저장으로 확인한 None/ZOOM, FitToSize/TOTAL의 저장 계약.
//! 원본의 글상자와 BorderFill을 모두 검사한다. 배치/픽셀 기대값은 이 검사에 두지 않는다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::style::ImageFillMode;
use std::{path::PathBuf, process::Command};

struct Files(PathBuf);
impl Files {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "rhwp-7661-save-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&dir).unwrap();
        Self(dir)
    }
    fn save(
        &self,
        core: &DocumentCore,
        from_hwpx: bool,
        to_hwpx: bool,
        step: usize,
    ) -> DocumentCore {
        let input = self.0.join(format!(
            "input-{step}.{}",
            if from_hwpx { "hwpx" } else { "hwp" }
        ));
        let output = self.0.join(format!(
            "output-{step}.{}",
            if to_hwpx { "hwpx" } else { "hwp" }
        ));
        let bytes = if from_hwpx {
            core.export_hwpx_native().unwrap()
        } else {
            core.export_hwp_native().unwrap()
        };
        std::fs::write(&input, bytes).unwrap();
        let bin = std::env::var("CARGO_BIN_EXE_rhwp")
            .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
        let result = Command::new(bin)
            .arg(if to_hwpx { "export-hwpx" } else { "convert" })
            .arg(input)
            .arg(&output)
            .output()
            .expect("save CLI");
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        DocumentCore::from_bytes(&std::fs::read(output).unwrap()).unwrap()
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn sample() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue7235/156086935_none_image_fill.hwp");
    DocumentCore::from_bytes(&std::fs::read(path).expect("원본")).expect("HWP parse")
}
fn modes(doc: &Document) -> Vec<ImageFillMode> {
    let mut out: Vec<_> = doc
        .doc_info
        .border_fills
        .iter()
        .filter_map(|b| b.fill.image.as_ref().map(|i| i.fill_mode))
        .collect();
    for section in &doc.sections {
        for para in &section.paragraphs {
            for control in &para.controls {
                if let Control::Shape(shape) = control {
                    if let Some(img) = shape.drawing().and_then(|d| d.fill.image.as_ref()) {
                        out.push(img.fill_mode);
                    }
                }
            }
        }
    }
    out
}
#[test]
fn hwp_none_image_fills_save_as_zoom_and_return_to_hwp_none() {
    let source = sample();
    assert_eq!(modes(source.document()), vec![ImageFillMode::None; 3]);
    let files = Files::new();
    let mut current = source;
    for turn in 0..2 {
        let xml = files.save(&current, false, true, turn * 2);
        assert_eq!(
            modes(xml.document()),
            vec![ImageFillMode::Zoom; 3],
            "글상자와 BorderFill 모두 보존"
        );
        current = files.save(&xml, true, false, turn * 2 + 1);
        assert_eq!(modes(current.document()), vec![ImageFillMode::None; 3]);
    }
}
#[test]
fn total_and_actual_tiling_remain_distinct_in_both_formats() {
    let mut source = sample();
    // 합성 모드 변경만 적용한다. 저장 줄·메트릭·그림 바이트는 변경하지 않는다.
    let doc = source.document_mut();
    for b in &mut doc.doc_info.border_fills {
        if let Some(i) = &mut b.fill.image {
            i.fill_mode = ImageFillMode::TileAll;
        }
    }
    for section in &mut doc.sections {
        for para in &mut section.paragraphs {
            for control in &mut para.controls {
                if let Control::Shape(shape) = control {
                    if let Some(i) = shape.drawing_mut().and_then(|d| d.fill.image.as_mut()) {
                        i.fill_mode = ImageFillMode::Total;
                    }
                }
            }
        }
    }
    let xml = DocumentCore::from_bytes(&source.export_hwpx_native().expect("HWPX save"))
        .expect("HWPX reopen");
    assert_eq!(
        modes(xml.document()),
        vec![
            ImageFillMode::TileAll,
            ImageFillMode::TileAll,
            ImageFillMode::Total
        ]
    );
    let files = Files::new();
    let binary = files.save(&xml, true, false, 0);
    assert_eq!(
        modes(binary.document()),
        vec![
            ImageFillMode::TileAll,
            ImageFillMode::TileAll,
            ImageFillMode::FitToSize
        ]
    );
    let xml = files.save(&binary, false, true, 1);
    assert_eq!(
        modes(xml.document()),
        vec![
            ImageFillMode::TileAll,
            ImageFillMode::TileAll,
            ImageFillMode::Total
        ]
    );
}
