// CLI capabilities 자기서술은 `test-caption` 을 <파일.hwp> 를 받는 일반 명령처럼
// 소개하지만, 실제로는 특정 fixture 전용 하드코딩 인덱스((0,2),(0,3),(1,0),(1,1))를
// 경계검사 없이 인덱싱해 임의 문서로 호출하면 패닉(exit 101)했다. "죽지 않는다"는
// CLI 계약을 지키는지 실제 문서로 계약 테스트를 고정한다.
use std::process::Command;

fn rhwp_bin() -> std::path::PathBuf {
    let mut p = std::env::current_exe().unwrap();
    p.pop();
    if p.ends_with("deps") {
        p.pop();
    }
    p.push(if cfg!(windows) { "rhwp.exe" } else { "rhwp" });
    p
}

#[test]
fn test_caption_does_not_panic_on_arbitrary_document() {
    let bin = rhwp_bin();
    if !bin.exists() {
        eprintln!("skip: {} 없음(먼저 release-test 빌드 필요)", bin.display());
        return;
    }
    // fixture 전용 하드코딩 인덱스((0,2)/(0,3)/(1,0)/(1,1))가 없는 임의의 실문서.
    let sample = "samples/2022년 국립국어원 업무계획.hwp";
    if !std::path::Path::new(sample).exists() {
        eprintln!("skip: {sample} 없음");
        return;
    }
    let out = Command::new(&bin)
        .args(["test-caption", sample])
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
}
