// [Task #2230] 그림 미지정 placeholder 선택 가능화 회귀 테스트.
//
// studio 개체 선택(findPictureAtClick)의 데이터 소스는 get_page_control_layout
// 이다. #2225 에서 MissingPicture placeholder 를 렌더 트리에 도입했지만 컨트롤
// 레이아웃에는 방출되지 않아(Placeholder 분기가 kind=="ole" 만 방출) 클릭
// 선택이 불가능했다. #2230 1단계: placeholder 에 문서 좌표(control_ref
// kind="picture")와 cell_context 를 배선하고, 컨트롤 레이아웃에
// type:"image" + missing:true 로 방출한다.
//
// 검증 문서: 36389312 결재문서 — 1페이지 상단 결재 표 "심볼" 셀에 bin_id=0
// (BinData 미첨부) Picture 컨트롤이 있어 MissingPicture placeholder 로
// 렌더된다 (좌표 실측 x≈646, y≈55, 75.6×75.6px).

use std::fs;
use std::path::Path;

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "samples/hwpx/opengov/36389312_결재문서본문_특정소방대상물 화재발생 알림(화재번호 2026-177).hwpx",
    );
    let bytes = fs::read(&path).expect("샘플 읽기 실패");
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("파싱 실패")
}

/// 컨트롤 레이아웃 JSON 에서 개별 컨트롤 오브젝트 문자열을 분리한다.
/// (수집기는 flat 배열 + 중첩 없는 스칼라/cellPath 필드만 방출하므로
/// "{\"type\":" 경계 분리로 충분하다. 표 컨트롤의 cells 중첩은 type 키가
/// 앞에 오는 경계 규칙 때문에 분리 대상에 걸리지 않는다.)
fn control_chunks(json: &str) -> Vec<&str> {
    let mut chunks = Vec::new();
    let mut rest = json;
    while let Some(pos) = rest.find("{\"type\":") {
        let tail = &rest[pos + 1..];
        let end = tail.find("{\"type\":").unwrap_or(tail.len());
        chunks.push(&rest[pos..pos + 1 + end]);
        rest = &rest[pos + 1..];
        rest = &rest[end.min(rest.len())..];
    }
    chunks
}

/// 미지정 그림 placeholder 가 클릭 선택 가능한 image 컨트롤로 방출된다.
#[test]
fn missing_picture_placeholder_emitted_as_selectable_image_control() {
    let doc = load_doc();
    let json = doc
        .get_page_control_layout(0)
        .expect("컨트롤 레이아웃 조회 실패");

    let missing: Vec<&str> = control_chunks(&json)
        .into_iter()
        .filter(|c| c.starts_with("{\"type\":\"image\"") && c.contains("\"missing\":true"))
        .collect();

    assert_eq!(
        missing.len(),
        1,
        "심볼 placeholder 가 missing image 컨트롤 1건으로 방출되어야 한다. json={json}"
    );

    let ctrl = missing[0];
    // 실측 좌표(x≈646.2, y≈54.9) — hit-test bbox 성립 확인
    assert!(
        ctrl.contains("\"x\":646.") && ctrl.contains("\"y\":54."),
        "심볼 placeholder bbox 좌표 불일치: {ctrl}"
    );
    // 문서 좌표 + 셀 경로 — enterPictureObjectSelectionDirect/커맨드 대상 특정에 필요
    for key in [
        "\"secIdx\":",
        "\"paraIdx\":",
        "\"controlIdx\":",
        "\"cellPath\":[",
    ] {
        assert!(ctrl.contains(key), "{key} 누락: {ctrl}");
    }
}

/// 그림이 실존하는 일반 image 컨트롤에는 missing 마커가 붙지 않는다.
#[test]
fn normal_image_control_has_no_missing_marker() {
    let doc = load_doc();
    let json = doc
        .get_page_control_layout(0)
        .expect("컨트롤 레이아웃 조회 실패");

    let normal: Vec<&str> = control_chunks(&json)
        .into_iter()
        .filter(|c| c.starts_with("{\"type\":\"image\"") && !c.contains("\"missing\":true"))
        .collect();

    // 좌측 기관 로고 1건 (실측 x≈84.1)
    assert_eq!(
        normal.len(),
        1,
        "일반 image 컨트롤 1건이어야 한다. json={json}"
    );
    assert!(
        normal[0].contains("\"x\":84."),
        "로고 컨트롤 좌표 불일치: {}",
        normal[0]
    );
}

/// 1×1 투명 PNG (object_ops/picture.rs 테스트 픽스처와 동일).
fn minimal_png() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
        0x44, 0xAE, 0x42, 0x60, 0x82,
    ]
}

/// [2단계] 그림 지정 커맨드: placeholder → 실그림 전환 + 컨트롤 레이아웃 정합.
#[test]
fn assign_picture_image_converts_placeholder_to_image() {
    let mut doc = load_doc();

    // 1단계 방출 실측 좌표: sec=0, parentParaIdx=0, cellPath=[{2,3,0}], ci=0
    // (wasm 래퍼는 오류 경로에서 JsValue 를 생성해 비-wasm 타겟에서 abort
    // 하므로 native 를 직접 호출한다.)
    let cell_path: &[(usize, usize, usize)] = &[(2, 3, 0)];
    let result = doc
        .assign_picture_image_native(0, 0, cell_path, 0, &minimal_png(), 1, 1, "png")
        .expect("그림 지정 실패");
    assert!(
        result.contains("\"ok\":true") && result.contains("\"binDataId\":"),
        "지정 결과 형식 불일치: {result}"
    );

    // 컨트롤 레이아웃: missing 마커 소멸 + 일반 image 2건(로고 + 심볼)
    let json = doc
        .get_page_control_layout(0)
        .expect("컨트롤 레이아웃 조회 실패");
    assert!(
        !json.contains("\"missing\":true"),
        "지정 후에도 missing 마커가 남아 있다: {json}"
    );
    let images: Vec<&str> = control_chunks(&json)
        .into_iter()
        .filter(|c| c.starts_with("{\"type\":\"image\""))
        .collect();
    assert_eq!(
        images.len(),
        2,
        "지정 후 image 컨트롤 2건이어야 한다: {json}"
    );

    // 지정된 그림이 placeholder 자리(실측 x≈646.2, 틀 크기 유지)에 배치된다
    assert!(
        images
            .iter()
            .any(|c| c.contains("\"x\":646.") && c.contains("\"w\":75.6")),
        "심볼 자리의 image 컨트롤 부재(틀 크기 유지 실패): {json}"
    );
}

/// [2단계] 대상 검증 실패 시 문서 무변형 (BinData 미등록).
#[test]
fn assign_picture_image_invalid_target_leaves_document_unchanged() {
    let mut doc = load_doc();
    let before = doc
        .get_page_control_layout(0)
        .expect("컨트롤 레이아웃 조회 실패");

    // 존재하지 않는 컨트롤 인덱스 → 오류
    let cell_path: &[(usize, usize, usize)] = &[(2, 3, 0)];
    assert!(
        doc.assign_picture_image_native(0, 0, cell_path, 99, &minimal_png(), 1, 1, "png")
            .is_err(),
        "잘못된 컨트롤 인덱스가 성공 처리됐다"
    );

    // 빈 이미지 데이터 → 오류
    assert!(
        doc.assign_picture_image_native(0, 0, cell_path, 0, &[], 1, 1, "png")
            .is_err(),
        "빈 이미지 데이터가 성공 처리됐다"
    );

    let after = doc
        .get_page_control_layout(0)
        .expect("컨트롤 레이아웃 조회 실패");
    assert_eq!(before, after, "실패 호출 후 레이아웃이 변형됐다");
}

/// [4단계] 그림 지정 후 HWPX 저장 왕복 — 재파싱에서 그림(BinData) 유지.
#[test]
fn assign_then_hwpx_roundtrip_preserves_image() {
    let mut doc = load_doc();
    let cell_path: &[(usize, usize, usize)] = &[(2, 3, 0)];
    doc.assign_picture_image_native(0, 0, cell_path, 0, &minimal_png(), 1, 1, "png")
        .expect("그림 지정 실패");

    let bytes = doc.export_hwpx_native().expect("HWPX 직렬화 실패");
    let reloaded = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("재파싱 실패");

    let json = reloaded
        .get_page_control_layout(0)
        .expect("컨트롤 레이아웃 조회 실패");
    assert!(
        !json.contains("\"missing\":true"),
        "HWPX 왕복 후 그림이 placeholder 로 되돌아갔다(BinData 소실): {json}"
    );
    let images = control_chunks(&json)
        .into_iter()
        .filter(|c| c.starts_with("{\"type\":\"image\""))
        .count();
    assert_eq!(
        images, 2,
        "HWPX 왕복 후 image 컨트롤 2건이어야 한다: {json}"
    );
}

/// [4단계] 그림 지정 후 HWP(5.0 CFB) 저장 왕복 — 재파싱에서 그림(BinData) 유지.
#[test]
fn assign_then_hwp_roundtrip_preserves_image() {
    let mut doc = load_doc();
    let cell_path: &[(usize, usize, usize)] = &[(2, 3, 0)];
    doc.assign_picture_image_native(0, 0, cell_path, 0, &minimal_png(), 1, 1, "png")
        .expect("그림 지정 실패");

    let bytes = doc.export_hwp_with_adapter().expect("HWP 직렬화 실패");
    let reloaded = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("재파싱 실패");

    let json = reloaded
        .get_page_control_layout(0)
        .expect("컨트롤 레이아웃 조회 실패");
    assert!(
        !json.contains("\"missing\":true"),
        "HWP 왕복 후 그림이 placeholder 로 되돌아갔다(BinData 소실): {json}"
    );
    let images = control_chunks(&json)
        .into_iter()
        .filter(|c| c.starts_with("{\"type\":\"image\""))
        .count();
    assert_eq!(images, 2, "HWP 왕복 후 image 컨트롤 2건이어야 한다: {json}");
}
