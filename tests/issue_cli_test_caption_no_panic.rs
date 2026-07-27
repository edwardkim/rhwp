// CLI capabilities 자기서술은 `test-caption` 을 <파일.hwp> 를 받는 일반 명령처럼
// 소개하지만, 실제로는 특정 fixture 전용 하드코딩 인덱스((0,2),(0,3),(1,0),(1,1))를
// 경계검사 없이 인덱싱해 임의 문서로 호출하면 패닉(exit 101)했다. "죽지 않는다"는
// CLI 계약을 지키는지 실제 문서로 계약 테스트를 고정한다.
use std::path::PathBuf;
use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn unique_temp_dir() -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let dir =
        std::env::temp_dir().join(format!("rhwp-test-caption-{}-{nonce}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("임시 출력 폴더 생성 실패");
    dir
}

#[test]
fn test_caption_does_not_panic_on_arbitrary_document() {
    // fixture 전용 하드코딩 인덱스((0,2)/(0,3)/(1,0)/(1,1))가 없는 임의의 실문서.
    let sample = std::fs::canonicalize("samples/2022년 국립국어원 업무계획.hwp")
        .expect("회귀 샘플이 저장소에 있어야 합니다");
    let output_dir = unique_temp_dir();
    let out = Command::new(rhwp_bin())
        .args(["test-caption", sample.to_str().expect("UTF-8 샘플 경로")])
        .args(["--output", output_dir.to_str().expect("UTF-8 출력 경로")])
        .output()
        .expect("test-caption 실행 실패");
    let code = out.status.code();
    assert_ne!(
        code,
        Some(101),
        "Rust panic(exit 101) 발생 — 범위 밖 인덱싱 회귀. stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        code,
        Some(0),
        "예기치 않은 종료 코드. stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        std::fs::read_dir(&output_dir)
            .expect("출력 폴더 읽기 실패")
            .next()
            .is_some(),
        "정상 종료했다면 SVG가 하나 이상 생성되어야 합니다"
    );
    std::fs::remove_dir_all(output_dir).expect("임시 출력 폴더 정리 실패");
}
