# Task #1280 2단계 완료보고서 — Rust 단위 테스트 (백엔드 계약 고정)

## 목표

글상자 생성 백엔드 계약(글상자=text_box 있음, 사각형=없음, 글상자에 텍스트 입력 보존)을 단위 테스트로
고정하여, 프런트 수정과 함께 회귀를 막는다.

## 변경 내용

**파일**: `src/document_core/commands/object_ops.rs` (`#[cfg(test)]` 모듈 `issue_1280_textbox_creation_tests` 추가)

기존 `issue_1151_cell_picture_insert_tests`의 `make_test_core()`/`parse_idx()` 헬퍼 패턴을 재사용하고,
`Control::Shape(s) => …(s.as_ref())` 접근은 `helpers.rs:56`의 기존 코드와 동일하게 작성했다.

추가 테스트 3종:

| 테스트 | 검증 |
|--------|------|
| `create_textbox_has_textbox` | `create_shape_control_native(shape_type="textbox")` → `get_textbox_from_shape(...).is_some()` |
| `create_rectangle_has_no_textbox` | `shape_type="rectangle"` → `get_textbox_from_shape(...).is_none()` (글상자/사각형 경로 분리) |
| `insert_text_into_created_textbox` | 생성한 글상자에 `insert_text_in_cell_native(0, para, ctrl, 0, 0, 0, "테스트")` → 내부 첫 문단 텍스트 `"테스트"` 보존 |

## 검증

```
test document_core::commands::object_ops::issue_1280_textbox_creation_tests::create_rectangle_has_no_textbox ... ok
test document_core::commands::object_ops::issue_1280_textbox_creation_tests::create_textbox_has_textbox ... ok
test document_core::commands::object_ops::issue_1280_textbox_creation_tests::insert_text_into_created_textbox ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1582 filtered out
```

| 항목 | 결과 |
|------|------|
| `cargo test issue_1280` (msvc 툴체인) | **3 passed; 0 failed** ✓ |
| `cargo fmt --check` (신규 코드 내용 포맷) | content diff **0건** ✓ |
| 컴파일 경고/에러 | 없음 |

> 잔여 `cargo fmt --check` 보고는 저장소 전역 CRLF newline 아티팩트(575건, 본 작업과 무관)뿐이며,
> CLAUDE.md에 따라 무관한 전역 newline diff는 만들지 않는다.

## 빌드 환경 메모

본 컨트리뷰터 윈도우 환경에 MSVC C++ 빌드 도구(`link.exe`)와 Windows SDK가 미설치되어 네이티브 빌드가
불가했고, GNU 툴체인도 번들 MinGW가 불완전(`as.exe` 부재)했다. 작업지시자가 VS 2022 Professional에
"C++ 데스크톱 개발" 워크로드(MSVC v143 + Windows 11 SDK)를 설치하여 `x86_64-pc-windows-msvc` 기본
툴체인으로 네이티브 빌드/테스트가 정상화되었다.

## 다음 단계

3단계: e2e 회귀 테스트(`issue-1280-textbox-text-input.test.mjs`) 작성 + WASM 빌드 + 검증.

## 승인 대기

본 보고서와 소스 커밋 후 승인을 요청한다. 승인 후 3단계로 진행한다.
