//! 캡션 mutation과 SVG 출력으로 round-trip 경계를 확인하는 내부 CLI command.

use std::fs;
use std::path::Path;

use rhwp::model::control::Control;
use rhwp::model::shape::{CaptionDirection, CaptionVertAlign};

use crate::{EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

#[derive(Clone, Copy)]
struct CaptionExpectation {
    para: usize,
    control: usize,
    direction_name: &'static str,
    vert_align_name: &'static str,
    direction: CaptionDirection,
    vert_align: CaptionVertAlign,
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
    let expectations = [
        CaptionExpectation {
            para: 0,
            control: 2,
            direction_name: "Bottom",
            vert_align_name: "Top",
            direction: CaptionDirection::Bottom,
            vert_align: CaptionVertAlign::Top,
        },
        CaptionExpectation {
            para: 0,
            control: 3,
            direction_name: "Top",
            vert_align_name: "Top",
            direction: CaptionDirection::Top,
            vert_align: CaptionVertAlign::Top,
        },
        CaptionExpectation {
            para: 1,
            control: 0,
            direction_name: "Left",
            vert_align_name: "Center",
            direction: CaptionDirection::Left,
            vert_align: CaptionVertAlign::Center,
        },
        CaptionExpectation {
            para: 1,
            control: 1,
            direction_name: "Right",
            vert_align_name: "Center",
            direction: CaptionDirection::Right,
            vert_align: CaptionVertAlign::Center,
        },
    ];

    let mut mutation_succeeded = [false; 4];
    let mut validation_failed = false;
    for (i, expected) in expectations.iter().enumerate() {
        let json = format!(
            r#"{{"hasCaption":true,"captionDirection":"{}","captionVertAlign":"{}","captionWidth":8504,"captionSpacing":850}}"#,
            expected.direction_name, expected.vert_align_name
        );
        println!(
            "[{}] para={}, ci={}, dir={}, va={}",
            i, expected.para, expected.control, expected.direction_name, expected.vert_align_name
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
    for (i, expected) in expectations.iter().enumerate() {
        if !mutation_succeeded[i] {
            continue;
        }
        let Some(section) = doc.document().sections.first() else {
            eprintln!("문서 오류: 캡션을 검사할 section이 없습니다.");
            return EXIT_RUNTIME;
        };
        let Some(paragraph) = section.paragraphs.get(expected.para) else {
            validation_failed = true;
            eprintln!(
                "[{}] 캡션 검증 오류: para={} 가 문서 범위를 벗어남(문단 {}개)",
                i,
                expected.para,
                section.paragraphs.len()
            );
            continue;
        };
        let Some(control) = paragraph.controls.get(expected.control) else {
            validation_failed = true;
            eprintln!(
                "[{}] 캡션 검증 오류: para={} ci={} 가 범위를 벗어남(컨트롤 {}개)",
                i,
                expected.para,
                expected.control,
                paragraph.controls.len()
            );
            continue;
        };
        let Control::Picture(picture) = control else {
            validation_failed = true;
            eprintln!(
                "[{}] 캡션 검증 오류: para={} ci={} 가 그림 컨트롤이 아님",
                i, expected.para, expected.control
            );
            continue;
        };
        let Some(caption) = picture.caption.as_ref() else {
            validation_failed = true;
            eprintln!(
                "[{}] 캡션 검증 오류: para={} ci={} 에 캡션이 없음",
                i, expected.para, expected.control
            );
            continue;
        };
        if caption.direction != expected.direction
            || caption.vert_align != expected.vert_align
            || caption.width != 8504
            || caption.spacing != 850
        {
            validation_failed = true;
            eprintln!(
                "[{}] 캡션 검증 오류: para={} ci={} 기대=(dir={:?}, va={:?}, width=8504, spacing=850) 실제=(dir={:?}, va={:?}, width={}, spacing={})",
                i,
                expected.para,
                expected.control,
                expected.direction,
                expected.vert_align,
                caption.direction,
                caption.vert_align,
                caption.width,
                caption.spacing
            );
            continue;
        }
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
