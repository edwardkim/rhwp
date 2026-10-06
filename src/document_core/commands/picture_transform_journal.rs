//! #6806: 리사이즈 Undo는 common 크기뿐 아니라 원본 행렬/파생 치수도 복원한다.
//! 문서 전체, 이미지 바이트, 캡션은 보관하지 않는다. TS 선형 히스토리가 ID를 소유한다.

use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::event::DocumentEvent;
use crate::model::image::Picture;
use crate::model::shape::{CommonObjAttr, ShapeComponentAttr, ShapeObject};

#[derive(Clone, Debug, serde::Deserialize)]
struct CellStep {
    #[serde(rename = "controlIndex", alias = "controlIdx")]
    control: usize,
    #[serde(rename = "cellIndex", alias = "cellIdx")]
    cell: usize,
    #[serde(rename = "cellParaIndex", alias = "cellParaIdx")]
    paragraph: usize,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct HeaderFooter {
    kind: String,
    outer_para_idx: usize,
    outer_control_idx: usize,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Target {
    sec: usize,
    ppi: usize,
    ci: usize,
    #[serde(default)]
    cell_path: Vec<CellStep>,
    header_footer: Option<HeaderFooter>,
}

#[derive(Clone, Debug)]
pub(crate) struct PictureTransformCapture {
    target: Target,
    identity: (u32, u32, u16),
    common: CommonObjAttr,
    shape: ShapeComponentAttr,
}

fn error(message: &str) -> HwpError {
    HwpError::RenderError(message.to_owned())
}

fn identity(picture: &Picture) -> (u32, u32, u16) {
    (
        picture.instance_id,
        picture.common.instance_id,
        picture.image_attr.bin_data_id,
    )
}

impl DocumentCore {
    fn picture_transform_target(&mut self, target: &Target) -> Result<&mut Picture, HwpError> {
        if target.header_footer.is_none() && target.cell_path.is_empty() {
            return self.resolve_picture_control_mut(target.sec, target.ppi, target.ci);
        }
        if target.header_footer.is_some() && !target.cell_path.is_empty() {
            return Err(error("머리말/꼬리말과 셀 경로를 동시에 지정할 수 없습니다"));
        }
        let section = self
            .document
            .sections
            .get_mut(target.sec)
            .ok_or_else(|| error("그림 변환 구역이 없습니다"))?;
        let paragraph = if let Some(hf) = &target.header_footer {
            let control = section
                .paragraphs
                .get_mut(hf.outer_para_idx)
                .and_then(|p| p.controls.get_mut(hf.outer_control_idx))
                .ok_or_else(|| error("머리말/꼬리말 경로가 없습니다"))?;
            let paragraphs = match control {
                Control::Header(h) if hf.kind == "header" => &mut h.paragraphs,
                Control::Footer(f) if hf.kind == "footer" => &mut f.paragraphs,
                _ => return Err(error("머리말/꼬리말 종류가 다릅니다")),
            };
            paragraphs
                .get_mut(target.ppi)
                .ok_or_else(|| error("머리말/꼬리말 문단이 없습니다"))?
        } else {
            let path: Vec<_> = target
                .cell_path
                .iter()
                .map(|p| (p.control, p.cell, p.paragraph))
                .collect();
            Self::resolve_cell_paragraph_mut(section, target.ppi, &path)?
        };
        match paragraph.controls.get_mut(target.ci) {
            Some(Control::Picture(p)) => Ok(p),
            Some(Control::Shape(s)) => match s.as_mut() {
                ShapeObject::Picture(p) => Ok(p),
                _ => Err(error("변환 복원 대상이 그림이 아닙니다")),
            },
            _ => Err(error("변환 복원 대상이 그림이 아닙니다")),
        }
    }

    pub fn capture_picture_transform_native(&mut self, target_json: &str) -> Result<u32, HwpError> {
        // 자동 축출로 기존 Undo를 깨뜨리지 않는다. 한도에서는 변경 전에 실패한다.
        if self.picture_transform_store.len() >= 4096 {
            return Err(error("그림 변환 보관 한도 초과"));
        }
        let next = self
            .next_picture_transform_id
            .checked_add(1)
            .ok_or_else(|| error("그림 변환 ID 소진"))?;
        let target: Target = serde_json::from_str(target_json)
            .map_err(|_| error("그림 변환 경로 JSON이 올바르지 않습니다"))?;
        let picture = self.picture_transform_target(&target)?;
        let capture = PictureTransformCapture {
            identity: identity(picture),
            common: picture.common.clone(),
            shape: picture.shape_attr.clone(),
            target,
        };
        let id = self.next_picture_transform_id;
        self.next_picture_transform_id = next;
        self.picture_transform_store.push((id, capture));
        Ok(id)
    }

    /// Undo/Redo 대칭 교환. raw를 JSON 입력으로 받지 않고 코어가 보관한 상태만 사용한다.
    pub fn swap_picture_transform_native(&mut self, id: u32) -> Result<(), HwpError> {
        let index = self
            .picture_transform_store
            .iter()
            .position(|(key, _)| *key == id)
            .ok_or_else(|| error("그림 변환 보관 ID가 없습니다"))?;
        let mut capture = self.picture_transform_store[index].1.clone();
        let picture = self.picture_transform_target(&capture.target)?;
        if identity(picture) != capture.identity {
            return Err(error("그림 변환 복원 대상이 바뀌었습니다"));
        }
        std::mem::swap(&mut picture.common, &mut capture.common);
        std::mem::swap(&mut picture.shape_attr, &mut capture.shape);
        let target = capture.target.clone();
        self.picture_transform_store[index].1 = capture;
        self.document.sections[target.sec].raw_stream = None;
        self.recompose_section(target.sec);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();
        self.event_log.push(DocumentEvent::PictureResized {
            section: target.sec,
            para: target.ppi,
            ctrl: target.ci,
        });
        Ok(())
    }

    pub fn discard_picture_transform_native(&mut self, id: u32) {
        self.picture_transform_store.retain(|(key, _)| *key != id);
    }
}
