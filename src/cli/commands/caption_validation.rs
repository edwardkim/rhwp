//! 캡션 mutation과 SVG 출력으로 round-trip 경계를 확인하는 내부 CLI command.

use std::fs;
use std::path::Path;

use rhwp::model::control::Control;
use rhwp::model::document::Section;
use rhwp::model::image::Picture;
use rhwp::model::shape::{Caption, CaptionDirection, CaptionVertAlign, ShapeObject};

use crate::{EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

const CAPTION_WIDTH: u32 = 8504;
const CAPTION_SPACING: i16 = 850;

struct CaptionExpectation {
    para: usize,
    control: usize,
    direction: CaptionDirection,
    vert_align: CaptionVertAlign,
}

impl CaptionExpectation {
    fn direction_name(&self) -> &'static str {
        match self.direction {
            CaptionDirection::Left => "Left",
            CaptionDirection::Right => "Right",
            CaptionDirection::Top => "Top",
            CaptionDirection::Bottom => "Bottom",
        }
    }

    fn vert_align_name(&self) -> &'static str {
        match self.vert_align {
            CaptionVertAlign::Top => "Top",
            CaptionVertAlign::Center => "Center",
            CaptionVertAlign::Bottom => "Bottom",
        }
    }

    fn properties_json(&self) -> String {
        format!(
            r#"{{"hasCaption":true,"captionDirection":"{}","captionVertAlign":"{}","captionWidth":{CAPTION_WIDTH},"captionSpacing":{CAPTION_SPACING}}}"#,
            self.direction_name(),
            self.vert_align_name()
        )
    }
}

const EXPECTATIONS: [CaptionExpectation; 4] = [
    CaptionExpectation {
        para: 0,
        control: 2,
        direction: CaptionDirection::Bottom,
        vert_align: CaptionVertAlign::Top,
    },
    CaptionExpectation {
        para: 0,
        control: 3,
        direction: CaptionDirection::Top,
        vert_align: CaptionVertAlign::Top,
    },
    CaptionExpectation {
        para: 1,
        control: 0,
        direction: CaptionDirection::Left,
        vert_align: CaptionVertAlign::Center,
    },
    CaptionExpectation {
        para: 1,
        control: 1,
        direction: CaptionDirection::Right,
        vert_align: CaptionVertAlign::Center,
    },
];

/// `set_picture_properties_native`의 좌표·그림 해석과 같은 범위를 읽는다.
///
/// 본문 뒤의 para 인덱스는 DocumentCore와 동일하게 Endnote 문단을 이어 붙인 가상
/// 인덱스로 해석한다. HWP3 파서는 그림을 `Shape(Picture)`로 올릴 수 있으므로 두
/// 그림 표현을 모두 받아야 mutation 성공 뒤 verification만 실패하는 오탐이 없다.
fn resolve_picture<'a>(
    section: &'a Section,
    expected: &CaptionExpectation,
) -> Result<&'a Picture, String> {
    let body_len = section.paragraphs.len();
    let paragraph = if expected.para < body_len {
        &section.paragraphs[expected.para]
    } else {
        let mut virtual_idx = expected.para - body_len;
        let mut found = None;
        'outer: for body_para in &section.paragraphs {
            for control in &body_para.controls {
                if let Control::Endnote(endnote) = control {
                    if virtual_idx < endnote.paragraphs.len() {
                        found = endnote.paragraphs.get(virtual_idx);
                        break 'outer;
                    }
                    virtual_idx -= endnote.paragraphs.len();
                }
            }
        }
        found.ok_or_else(|| {
            format!(
                "para={} 가 문서 범위를 벗어남(본문 문단 {}개)",
                expected.para, body_len
            )
        })?
    };

    let control = paragraph.controls.get(expected.control).ok_or_else(|| {
        format!(
            "para={} ci={} 가 범위를 벗어남(컨트롤 {}개)",
            expected.para,
            expected.control,
            paragraph.controls.len()
        )
    })?;
    match control {
        Control::Picture(picture) => Ok(picture),
        Control::Shape(shape) => match shape.as_ref() {
            ShapeObject::Picture(picture) => Ok(picture),
            _ => Err(format!(
                "para={} ci={} 의 Shape 컨트롤이 그림이 아님",
                expected.para, expected.control
            )),
        },
        _ => Err(format!(
            "para={} ci={} 가 그림 컨트롤이 아님",
            expected.para, expected.control
        )),
    }
}

fn verify_caption<'a>(
    picture: &'a Picture,
    expected: &CaptionExpectation,
) -> Result<&'a Caption, String> {
    let caption = picture.caption.as_ref().ok_or_else(|| {
        format!(
            "para={} ci={} 에 캡션이 없음",
            expected.para, expected.control
        )
    })?;
    if caption.direction != expected.direction
        || caption.vert_align != expected.vert_align
        || caption.width != CAPTION_WIDTH
        || caption.spacing != CAPTION_SPACING
    {
        return Err(format!(
            "para={} ci={} 기대=(dir={:?}, va={:?}, width={CAPTION_WIDTH}, spacing={CAPTION_SPACING}) 실제=(dir={:?}, va={:?}, width={}, spacing={})",
            expected.para,
            expected.control,
            expected.direction,
            expected.vert_align,
            caption.direction,
            caption.vert_align,
            caption.width,
            caption.spacing
        ));
    }
    Ok(caption)
}

/// 캡션 방향별 테스트: 4개 이미지에 각각 Bottom/Top/Left/Right 캡션을 설정하고 SVG 출력
pub(crate) fn run(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("사용법: rhwp test-caption <파일.hwp> [-o <출력 폴더>]");
        return EXIT_USAGE;
    }
    if args[0].starts_with('-') {
        eprintln!(
            "오류: test-caption 입력 파일 자리에 옵션을 쓸 수 없습니다 - {}",
            args[0]
        );
        return EXIT_USAGE;
    }

    let input = &args[0];
    let mut output_dir = Path::new("output/caption-test");
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("오류: {} 뒤에 출력 폴더 경로가 필요합니다.", args[i]);
                    return EXIT_USAGE;
                };
                if value.starts_with('-') {
                    eprintln!("오류: {} 뒤에 출력 폴더 경로가 필요합니다.", args[i]);
                    return EXIT_USAGE;
                }
                output_dir = Path::new(value);
                i += 2;
            }
            option => {
                eprintln!("오류: 알 수 없는 test-caption 옵션입니다 - {option}");
                return EXIT_USAGE;
            }
        }
    }

    let data = match fs::read(input) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("파일 읽기 오류: {}", e);
            return EXIT_RUNTIME;
        }
    };

    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("파싱 오류: {}", e);
            return EXIT_RUNTIME;
        }
    };

    if doc.document().sections.is_empty() {
        eprintln!("문서 오류: 캡션을 검사할 section이 없습니다.");
        return EXIT_RUNTIME;
    }

    // 문단 0: 컨트롤 2,3 / 문단 1: 컨트롤 0,1
    let mut mutation_succeeded = vec![false; EXPECTATIONS.len()];
    let mut validation_failed = false;
    for (i, expected) in EXPECTATIONS.iter().enumerate() {
        let json = expected.properties_json();
        println!(
            "[{}] para={}, ci={}, dir={}, va={}",
            i,
            expected.para,
            expected.control,
            expected.direction_name(),
            expected.vert_align_name()
        );
        match doc.set_picture_properties_native(0, expected.para, expected.control, &json) {
            Ok(result) => {
                mutation_succeeded[i] = true;
                println!("  결과: {}", result);
            }
            Err(error) => {
                validation_failed = true;
                eprintln!(
                    "[{}] 캡션 설정 오류: para={} ci={}: {:?}",
                    i, expected.para, expected.control, error
                );
            }
        }
    }

    // mutation 성공만으로는 round-trip 검증이 아니다. 네 대상의 실제 캡션 값까지
    // 모두 일치해야 렌더 단계로 이동한다. setter 실패 대상은 이미 진단했으므로
    // 중복 오류 대신 성공한 mutation만 확인한다.
    let section = &doc.document().sections[0];
    for (i, expected) in EXPECTATIONS.iter().enumerate() {
        if !mutation_succeeded[i] {
            continue;
        }
        let caption = match resolve_picture(section, expected)
            .and_then(|picture| verify_caption(picture, expected))
        {
            Ok(caption) => caption,
            Err(error) => {
                validation_failed = true;
                eprintln!("[{i}] 캡션 검증 오류: {error}");
                continue;
            }
        };
        // 기존 사람용 stdout은 Option<String>의 Debug 표현을 포함한다. 내부 진단
        // 소비자가 문자열을 비교하므로 이 래핑을 단순화하지 않는다.
        println!(
            "[{}] caption={:?}",
            i,
            Some(format!(
                "dir={:?}, paras={}, text={:?}",
                caption.direction,
                caption.paragraphs.len(),
                caption.paragraphs.first().map(|p| &p.text)
            ))
        );
    }

    if validation_failed {
        eprintln!("캡션 검증 실패: 네 대상의 mutation과 verification이 모두 성공해야 합니다.");
        return EXIT_RUNTIME;
    }

    // SVG 출력
    let page_count = doc.page_count();
    if page_count == 0 {
        eprintln!("SVG 렌더링 오류: 문서에 출력할 페이지가 없습니다.");
        return EXIT_RUNTIME;
    }
    if let Err(e) = fs::create_dir_all(output_dir) {
        eprintln!("출력 폴더 생성 오류: {}: {}", output_dir.display(), e);
        return EXIT_RUNTIME;
    }
    println!("페이지 수: {}", page_count);
    for p in 0..page_count {
        let svg = match doc.render_page_svg(p) {
            Ok(svg) => svg,
            Err(e) => {
                eprintln!("SVG 렌더링 오류(page {}): {:?}", p, e);
                return EXIT_RUNTIME;
            }
        };
        let path = output_dir.join(format!("caption-test-p{}.svg", p));
        if let Err(e) = fs::write(&path, &svg) {
            eprintln!("SVG 저장 오류: {}: {}", path.display(), e);
            return EXIT_RUNTIME;
        }
        println!("  → {}", path.display());
    }
    println!("완료");
    EXIT_OK
}
