//! 문서 전체 구조 (Document, Section, SectionDef)

use super::*;
use super::bin_data::{BinData, BinDataContent};
use super::footnote::FootnoteShape;
use super::header_footer::MasterPage;
use super::page::{PageDef, PageBorderFill};
use super::paragraph::Paragraph;
use super::style::{CharShape, ParaShape, Style, BorderFill, Font, TabDef, Numbering, Bullet};

/// 파서가 모델링하지 않는 원시 레코드 (라운드트립 보존용)
#[derive(Debug, Clone, Default)]
pub struct RawRecord {
    /// 태그 ID
    pub tag_id: u16,
    /// 레벨 (트리 깊이)
    pub level: u16,
    /// 레코드 데이터
    pub data: Vec<u8>,
}

/// HWP 문서 전체를 나타내는 최상위 구조체
#[derive(Debug, Clone, Default)]
pub struct Document {
    /// 파일 헤더 정보
    pub header: FileHeader,
    /// 문서 속성
    pub doc_properties: DocProperties,
    /// 문서 정보 (DocInfo 스트림에서 파싱)
    pub doc_info: DocInfo,
    /// 본문 구역 리스트
    pub sections: Vec<Section>,
    /// 미리보기 데이터 (PrvImage, PrvText)
    pub preview: Option<Preview>,
    /// 바이너리 데이터 콘텐츠 (이미지, OLE 등)
    pub bin_data_content: Vec<BinDataContent>,
    /// 파서가 모델링하지 않는 추가 CFB 스트림 (라운드트립 보존용)
    /// (스트림 경로, 데이터)
    pub extra_streams: Vec<(String, Vec<u8>)>,
}

/// 미리보기 데이터 (PrvImage, PrvText 스트림)
#[derive(Debug, Clone, Default)]
pub struct Preview {
    /// 썸네일 이미지 (BMP 또는 GIF)
    pub image: Option<PreviewImage>,
    /// 미리보기 텍스트
    pub text: Option<String>,
}

/// 썸네일 이미지
#[derive(Debug, Clone)]
pub struct PreviewImage {
    /// 이미지 포맷
    pub format: PreviewImageFormat,
    /// 이미지 바이너리 데이터
    pub data: Vec<u8>,
}

/// 썸네일 이미지 포맷
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PreviewImageFormat {
    /// PNG 이미지
    Png,
    /// BMP 이미지
    Bmp,
    /// GIF 이미지
    Gif,
    /// 알 수 없는 포맷
    Unknown,
}

/// 파일 헤더 정보 (FileHeader 스트림)
#[derive(Debug, Clone, Default)]
pub struct FileHeader {
    /// HWP 버전
    pub version: HwpVersion,
    /// 파일 속성 비트 플래그
    pub flags: u32,
    /// 압축 여부
    pub compressed: bool,
    /// 암호 설정 여부
    pub encrypted: bool,
    /// 배포용 문서 여부
    pub distribution: bool,
    /// 원본 FileHeader 256바이트 (직렬화 시 원본 복원용)
    pub raw_data: Option<Vec<u8>>,
}

/// HWP 파일 버전
#[derive(Debug, Clone, Default)]
pub struct HwpVersion {
    pub major: u8,
    pub minor: u8,
    pub build: u8,
    pub revision: u8,
}

/// 문서 속성 (HWPTAG_DOCUMENT_PROPERTIES)
#[derive(Debug, Clone, Default)]
pub struct DocProperties {
    /// 원본 레코드 바이트 (라운드트립 보존용)
    pub raw_data: Option<Vec<u8>>,
    /// 구역 개수
    pub section_count: u16,
    /// 페이지 시작 번호
    pub page_start_num: u16,
    /// 각주 시작 번호
    pub footnote_start_num: u16,
    /// 미주 시작 번호
    pub endnote_start_num: u16,
    /// 그림 시작 번호
    pub picture_start_num: u16,
    /// 표 시작 번호
    pub table_start_num: u16,
    /// 수식 시작 번호
    pub equation_start_num: u16,
    /// 캐럿 위치: 리스트 아이디
    pub caret_list_id: u32,
    /// 캐럿 위치: 문단 아이디
    pub caret_para_id: u32,
    /// 캐럿 위치: 문단 내 글자 위치
    pub caret_char_pos: u32,
}

/// 문서 정보 (DocInfo 스트림의 ID 매핑 데이터)
#[derive(Debug, Clone, Default)]
pub struct DocInfo {
    /// 바이너리 데이터 목록
    pub bin_data_list: Vec<BinData>,
    /// 글꼴 목록 (언어별: 한글, 영어, 한자, 일어, 기타, 기호, 사용자)
    pub font_faces: Vec<Vec<Font>>,
    /// 테두리/배경 목록
    pub border_fills: Vec<BorderFill>,
    /// 글자 모양 목록
    pub char_shapes: Vec<CharShape>,
    /// 탭 정의 목록
    pub tab_defs: Vec<TabDef>,
    /// 문단 번호 정의 목록
    pub numberings: Vec<Numbering>,
    /// 글머리표 정의 목록
    pub bullets: Vec<Bullet>,
    /// 문단 모양 목록
    pub para_shapes: Vec<ParaShape>,
    /// 스타일 목록
    pub styles: Vec<Style>,
    /// 파서가 모델링하지 않는 추가 레코드 (DOC_DATA, FORBIDDEN_CHAR 등)
    pub extra_records: Vec<RawRecord>,
    /// 원본 DocInfo 레코드 스트림 바이트 (직렬화 시 원본 복원용)
    pub raw_stream: Option<Vec<u8>>,
    /// Bullet 개수 (ID_MAPPINGS에 포함, bullets.len()과 동기화)
    pub bullet_count: u32,
    /// MemoShape 개수 (ID_MAPPINGS에 포함, 현재 파싱 미지원이지만 보존 필요)
    pub memo_shape_count: u32,
    /// DISTRIBUTE_DOC_DATA 레코드 제거 플래그 (serializer에서 raw_stream surgical remove 수행)
    pub distribute_doc_data_removed: bool,
    /// raw_stream이 model 변경과 동기화되지 않음 (serializer에서 재생성 필요)
    pub raw_stream_dirty: bool,
}

/// 본문의 구역 (Section)
#[derive(Debug, Clone, Default)]
pub struct Section {
    /// 구역 정의
    pub section_def: SectionDef,
    /// 문단 리스트
    pub paragraphs: Vec<Paragraph>,
    /// 원본 BodyText 레코드 스트림 바이트 (직렬화 시 원본 복원용)
    /// 편집 시 None으로 초기화하여 재직렬화 유도
    pub raw_stream: Option<Vec<u8>>,
}

/// 구역 정의 (HWPTAG_CTRL_HEADER - 'secd')
#[derive(Debug, Clone, Default)]
pub struct SectionDef {
    /// 속성 비트 플래그
    pub flags: u32,
    /// 단 간격
    pub column_spacing: HwpUnit16,
    /// 기본 탭 간격
    pub default_tab_spacing: HwpUnit,
    /// 쪽 번호 (0이면 앞 구역에 이어서)
    pub page_num: u16,
    /// 쪽 번호 종류 (flags bit 20-21): 0=이어서, 1=홀수, 2=짝수
    /// (사용자 지정은 page_num > 0 으로 구분)
    pub page_num_type: u8,
    /// 그림 번호
    pub picture_num: u16,
    /// 표 번호
    pub table_num: u16,
    /// 수식 번호
    pub equation_num: u16,
    /// 용지 설정
    pub page_def: PageDef,
    /// 각주 모양
    pub footnote_shape: FootnoteShape,
    /// 미주 모양
    pub endnote_shape: FootnoteShape,
    /// 쪽 테두리/배경
    pub page_border_fill: PageBorderFill,
    /// 머리말 감추기
    pub hide_header: bool,
    /// 꼬리말 감추기
    pub hide_footer: bool,
    /// 바탕쪽 감추기
    pub hide_master_page: bool,
    /// 테두리 감추기
    pub hide_border: bool,
    /// 배경 감추기
    pub hide_fill: bool,
    /// 빈 줄 감추기 (bit 19): 페이지 시작 부분의 빈 줄 2개까지 높이 0 처리
    pub hide_empty_line: bool,
    /// 텍스트 방향 (0: 가로, 1: 세로)
    pub text_direction: u8,
    /// 개요 번호 ID (SectionDef 바이트 14-15, Numbering 테이블 참조, 1-based)
    pub outline_numbering_id: u16,
    /// CTRL_HEADER 데이터의 파싱된 필드 이후 추가 바이트 (라운드트립 보존용)
    pub raw_ctrl_extra: Vec<u8>,
    /// 추가 쪽 테두리/배경 (2번째, 3번째 등)
    pub extra_page_border_fills: Vec<PageBorderFill>,
    /// 파서가 인식하지 못한 자식 레코드 (바탕쪽 등, 라운드트립 보존용)
    pub extra_child_records: Vec<RawRecord>,
    /// 바탕쪽 (extra_child_records에서 파싱, 렌더링 전용)
    pub master_pages: Vec<MasterPage>,
}

impl Document {
    /// 배포용(읽기전용) 문서를 편집 가능한 일반 문서로 변환한다.
    ///
    /// 변환 내용:
    /// - FileHeader: distribution 플래그(bit 2) 제거, raw_data 초기화
    /// - DocInfo: DISTRIBUTE_DOC_DATA 레코드 제거, raw_stream 초기화
    ///
    /// 이미 일반 문서인 경우 아무 작업도 하지 않는다.
    /// 반환값: 변환이 실제로 수행되었으면 true
    pub fn convert_to_editable(&mut self) -> bool {
        if !self.header.distribution {
            return false;
        }

        // 1. FileHeader: distribution 플래그 제거
        self.header.distribution = false;
        self.header.flags &= !0x04; // bit 2 제거
        self.header.raw_data = None; // 플래그 변경 반영을 위해 재생성 유도

        // 2. DocInfo: DISTRIBUTE_DOC_DATA 레코드 제거
        // HWPTAG_BEGIN(0x010) + 12 = 0x01C
        const TAG_DISTRIBUTE_DOC_DATA: u16 = 0x01C;
        self.doc_info.extra_records.retain(|r| r.tag_id != TAG_DISTRIBUTE_DOC_DATA);
        // raw_stream의 surgical remove는 serializer 계층에서 처리
        self.doc_info.distribute_doc_data_removed = true;

        // 3. BodyText: 파싱 시 이미 복호화되어 있으므로 추가 처리 불필요

        true
    }

    /// 기존 CharShape를 복제하고 수정사항을 적용한 후, 동일한 것이 있으면 재사용한다.
    ///
    /// 반환값: 적용된 CharShape의 ID (기존 또는 새로 생성)
    pub fn find_or_create_char_shape(
        &mut self,
        base_id: u32,
        mods: &super::style::CharShapeMods,
    ) -> u32 {
        let base = self.doc_info.char_shapes
            .get(base_id as usize)
            .cloned()
            .unwrap_or_default();
        let modified = mods.apply_to(&base);

        // 동일한 CharShape가 이미 있는지 검색
        for (i, cs) in self.doc_info.char_shapes.iter().enumerate() {
            if *cs == modified {
                return i as u32;
            }
        }

        // 없으면 새로 추가
        let new_id = self.doc_info.char_shapes.len() as u32;
        self.doc_info.char_shapes.push(modified);
        self.doc_info.raw_stream_dirty = true;
        new_id
    }

    /// 기존 ParaShape를 복제하고 수정사항을 적용한 후, 동일한 것이 있으면 재사용한다.
    ///
    /// 반환값: 적용된 ParaShape의 ID (기존 또는 새로 생성)
    pub fn find_or_create_para_shape(
        &mut self,
        base_id: u16,
        mods: &super::style::ParaShapeMods,
    ) -> u16 {
        let base = self.doc_info.para_shapes
            .get(base_id as usize)
            .cloned()
            .unwrap_or_default();
        let modified = mods.apply_to(&base);

        // 동일한 ParaShape가 이미 있는지 검색
        for (i, ps) in self.doc_info.para_shapes.iter().enumerate() {
            if *ps == modified {
                return i as u16;
            }
        }

        // 없으면 새로 추가
        let new_id = self.doc_info.para_shapes.len() as u16;
        self.doc_info.para_shapes.push(modified);
        self.doc_info.raw_stream_dirty = true;
        new_id
    }

    /// 동일한 TabDef가 이미 있으면 그 ID를, 없으면 새로 추가하고 ID를 반환한다.
    pub fn find_or_create_tab_def(&mut self, new_td: super::style::TabDef) -> u16 {
        // 동일한 TabDef가 이미 있는지 검색
        for (i, td) in self.doc_info.tab_defs.iter().enumerate() {
            if *td == new_td {
                return i as u16;
            }
        }

        // 없으면 새로 추가
        let new_id = self.doc_info.tab_defs.len() as u16;
        self.doc_info.tab_defs.push(new_td);
        self.doc_info.raw_stream_dirty = true;
        new_id
    }

    /// [Task #741 후속] 외부 file path 그림 (HWP3 영역 영역 절대 경로 영역 저장된 image)
    /// 영역 의 binary 영역 영역 base_dir 영역 영역 자동 load.
    ///
    /// HWP3 파일 영역 의 image 영역 영역 영역 원본 절대 경로 (예: "D:\\Work\\...\\rdb02.gif")
    /// 영역 저장 영역. 본 환경 영역 영역 영역 path 영역 영역 access 부재 영역 영역 영역,
    /// 본 helper 영역 영역 path 영역 영역 basename 영역 영역 추출 (`rdb02.gif`) → `base_dir`
    /// 영역 영역 영역 file 영역 load → `bin_data_content` 영역 push 영역 → 기존 renderer
    /// 영역 (svg / web_canvas / skia) 영역 영역 영역 image 영역 표시 영역.
    ///
    /// 반환: load 영역 image 영역.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn populate_external_images_from_dir(&mut self, base_dir: &std::path::Path) -> usize {
        use crate::model::control::Control;
        use crate::model::shape::ShapeObject;

        let mut loaded = 0;
        // (storage_id, basename, extension) 영역 영역 image 영역 수집
        let mut to_load: Vec<(u16, String, String)> = Vec::new();
        for section in &self.sections {
            for para in &section.paragraphs {
                for ctrl in &para.controls {
                    let pic = match ctrl {
                        Control::Picture(p) => p,
                        Control::Shape(s) => match s.as_ref() {
                            ShapeObject::Picture(p) => p,
                            _ => continue,
                        },
                        _ => continue,
                    };
                    if let Some(ref path) = pic.image_attr.external_path {
                        let id = pic.image_attr.bin_data_id;
                        // 이미 load 영역 (bin_data_content 영역 영역 entry 보유) 영역 skip
                        let already_loaded = self.bin_data_content.iter()
                            .any(|c| c.id == id && !c.data.is_empty());
                        if already_loaded { continue; }

                        // path 영역 영역 basename 추출 (Windows / Unix 영역 모두 대응)
                        let basename = path.rsplit(|c| c == '/' || c == '\\').next().unwrap_or(path);
                        let ext = std::path::Path::new(basename)
                            .extension().and_then(|e| e.to_str()).unwrap_or("").to_string();
                        to_load.push((id, basename.to_string(), ext));
                    }
                }
            }
        }

        for (id, basename, ext) in to_load {
            let full_path = base_dir.join(&basename);
            if let Ok(data) = std::fs::read(&full_path) {
                // 본질: bin_data_content 영역 영역 (id-1) index 영역 영역 access (utils.rs:23).
                let idx = (id as usize).saturating_sub(1);
                if idx < self.bin_data_content.len() {
                    self.bin_data_content[idx].id = id;
                    self.bin_data_content[idx].data = data;
                    self.bin_data_content[idx].extension = ext;
                } else {
                    self.bin_data_content.push(crate::model::bin_data::BinDataContent {
                        id, data, extension: ext,
                    });
                }
                loaded += 1;

                // [한컴 viewer 정합] 원본 절대 경로 영역 영역 access 부재 시 HWP file 영역
                // 영역 같은 dir 영역 image 영역 발견 영역 영역 dialog 영역 영역 영역 의 path 영역
                // resolved local path 영역 영역 갱신 (basename 영역만 부재 영역).
                let resolved = full_path.to_string_lossy().to_string();
                for section in &mut self.sections {
                    for para in &mut section.paragraphs {
                        for ctrl in &mut para.controls {
                            let pic = match ctrl {
                                crate::model::control::Control::Picture(p) => p,
                                crate::model::control::Control::Shape(s) => match s.as_mut() {
                                    crate::model::shape::ShapeObject::Picture(p) => p,
                                    _ => continue,
                                },
                                _ => continue,
                            };
                            if pic.image_attr.bin_data_id == id
                                && pic.image_attr.external_path.is_some() {
                                pic.image_attr.external_path = Some(resolved.clone());
                            }
                        }
                    }
                }
            }
        }
        loaded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_default() {
        let doc = Document::default();
        assert_eq!(doc.sections.len(), 0);
        assert_eq!(doc.doc_properties.section_count, 0);
    }

    #[test]
    fn test_hwp_version() {
        let ver = HwpVersion { major: 5, minor: 0, build: 3, revision: 0 };
        assert_eq!(ver.major, 5);
    }

    #[test]
    fn test_convert_to_editable_clears_distribution() {
        let mut doc = Document {
            header: FileHeader {
                flags: 0x05, // compressed + distribution
                distribution: true,
                compressed: true,
                ..Default::default()
            },
            doc_info: DocInfo {
                extra_records: vec![
                    RawRecord { tag_id: 28, level: 0, data: vec![0u8; 256] }, // HWPTAG_DISTRIBUTE_DOC_DATA = HWPTAG_BEGIN(0x10=16) + 12 = 28
                    RawRecord { tag_id: 99, level: 0, data: vec![1, 2, 3] }, // 다른 레코드
                ],
                ..Default::default()
            },
            ..Default::default()
        };

        let converted = doc.convert_to_editable();
        assert!(converted);
        assert!(!doc.header.distribution);
        assert_eq!(doc.header.flags, 0x01); // compressed만 남음
        assert!(doc.header.raw_data.is_none());
        assert!(doc.doc_info.raw_stream.is_none());
        // DISTRIBUTE_DOC_DATA 레코드 제거, 다른 레코드는 보존
        assert_eq!(doc.doc_info.extra_records.len(), 1);
        assert_eq!(doc.doc_info.extra_records[0].tag_id, 99);
    }

    #[test]
    fn test_convert_to_editable_noop_for_normal() {
        let mut doc = Document {
            header: FileHeader {
                flags: 0x01, // compressed only
                distribution: false,
                compressed: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let converted = doc.convert_to_editable();
        assert!(!converted);
        assert_eq!(doc.header.flags, 0x01); // 변경 없음
    }

    #[test]
    fn test_find_or_create_char_shape_reuse() {
        use super::style::{CharShape, CharShapeMods};
        let mut doc = Document::default();
        // 기본 CharShape 하나 추가
        let mut cs = CharShape::default();
        cs.bold = false;
        doc.doc_info.char_shapes.push(cs.clone());

        // bold=true로 수정 → 새 ID 생성
        let mods = CharShapeMods { bold: Some(true), ..Default::default() };
        let id1 = doc.find_or_create_char_shape(0, &mods);
        assert_eq!(id1, 1);
        assert_eq!(doc.doc_info.char_shapes.len(), 2);

        // 동일한 수정 → 기존 ID 재사용
        let id2 = doc.find_or_create_char_shape(0, &mods);
        assert_eq!(id2, 1);
        assert_eq!(doc.doc_info.char_shapes.len(), 2); // 추가 생성 없음
    }

    #[test]
    fn test_find_or_create_para_shape_reuse() {
        use super::style::{ParaShape, ParaShapeMods, Alignment};
        let mut doc = Document::default();
        doc.doc_info.para_shapes.push(ParaShape::default());

        let mods = ParaShapeMods { alignment: Some(Alignment::Center), ..Default::default() };
        let id1 = doc.find_or_create_para_shape(0, &mods);
        assert_eq!(id1, 1);

        // 동일한 수정 → 재사용
        let id2 = doc.find_or_create_para_shape(0, &mods);
        assert_eq!(id2, 1);
        assert_eq!(doc.doc_info.para_shapes.len(), 2);
    }
}
