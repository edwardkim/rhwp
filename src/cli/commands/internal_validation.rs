//! Internal document mutation and round-trip validation command adapters.

use crate::{validate_internal_positionals, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

pub(crate) fn run(args: &[String]) -> i32 {
    // 인자를 생략하면 저장소에 없는 하드코딩 경로("hwp_webctl/bsbc01_10_000.hwp")를
    // `.expect()` 로 읽어 패닉(exit 101)했다 — 계약(cli_commands.md)에 없는 종료 코드라
    // CI 게이트가 분류할 수 없다. 형제 명령 test-caption 과 같은 모양으로 맞춘다
    // (tests/issue_cli_test_caption_no_panic.rs 가 그쪽을 이미 고정하고 있다).
    if args.is_empty() {
        eprintln!("사용법: rhwp test-field <파일.hwp> [출력.hwp]");
        return EXIT_USAGE;
    }
    if let Err(code) = validate_internal_positionals("test-field", args, 2) {
        return code;
    }
    let input = args[0].as_str();
    let output = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or("output/field_test.hwp");

    let data = match std::fs::read(input) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일 읽기 실패 - {}: {}", input, e);
            return EXIT_RUNTIME;
        }
    };
    let mut core = match rhwp::document_core::DocumentCore::from_bytes(&data) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("오류: 문서 파싱 실패 - {}: {:?}", input, e);
            return EXIT_RUNTIME;
        }
    };

    // 1. 필드 목록 출력
    let fields = core.collect_all_fields();
    println!("=== 필드 목록 ({}개) ===", fields.len());
    for fi in &fields {
        let name = fi.field.field_name().unwrap_or("(이름없음)");
        println!("  {} = \"{}\"", name, fi.value);
    }

    // 2. 필드에 값 설정
    let test_data = [
        ("mbizNm", "청소년 자립지원사업"),
        ("newCtnuTxt", "계속"),
        ("chargerNm", "홍길동"),
        ("telno", "02-1234-5678"),
        ("sFisYear", "2026"),
        // 셀 필드
        ("bizPurps", "청소년 자립 역량 강화"),
        ("bizPrdTxt", "2026.01 ~ 2026.12"),
        ("insttNm", "시청 복지과"),
    ];

    println!("\n=== 필드 값 설정 ===");
    for (name, value) in &test_data {
        match core.set_field_value_by_name(name, value) {
            Ok(r) => println!("  ✓ {} = \"{}\" → {}", name, value, r),
            Err(e) => println!("  ✗ {} = \"{}\" → {}", name, value, e),
        }
    }

    // 3. 설정 후 확인
    println!("\n=== 설정 후 확인 ===");
    let fields2 = core.collect_all_fields();
    for fi in &fields2 {
        let name = fi.field.field_name().unwrap_or("(이름없음)");
        println!("  {} = \"{}\"", name, fi.value);
    }

    // 3.5 pi=0 문단 텍스트 직접 확인
    let para0 = &core.document().sections[0].paragraphs[0];

    // 4. 직렬화 → 저장
    let saved = match core.export_hwp_native() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("오류: 직렬화 실패 - {:?}", e);
            return EXIT_RUNTIME;
        }
    };
    if let Err(e) = std::fs::write(output, &saved) {
        eprintln!("오류: 저장 실패 - {}: {}", output, e);
        return EXIT_RUNTIME;
    }
    println!("\n저장: {} ({}바이트)", output, saved.len());

    // 5. 재로딩 → 필드 확인
    let core2 = match rhwp::document_core::DocumentCore::from_bytes(&saved) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("오류: 재로딩 실패 - {:?}", e);
            return EXIT_RUNTIME;
        }
    };
    let fields3 = core2.collect_all_fields();
    println!("\n=== 재로딩 후 확인 ===");
    for fi in &fields3 {
        let name = fi.field.field_name().unwrap_or("(이름없음)");
        println!("  {} = \"{}\"", name, fi.value);
    }
    EXIT_OK
}
