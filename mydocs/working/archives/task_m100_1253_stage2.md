# Task #1253 Stage2

## 목적

Stage0/Stage1에서 확인한 `미주 사이`와 `구분선 아래` 렌더 불일치를 실제 조판 공통 로직에서 해결한다. Stage1은 UI 정합만 처리했으므로, 이번 단계에서는 한컴 UI 값이 파서/API/렌더에서 같은 의미로 소비되는지 확인하고 보정한다.

## 작업 항목

1. `get_endnote_shape_native`/`apply_endnote_shape_native`의 Studio JSON 필드와 내부 `FootnoteShape` 필드 매핑을 재확인한다.
2. HWP5/HWPX 파서의 `betweenNotes`, `belowLine`, `aboveLine` 매핑이 Stage0 판단과 테스트 기준을 유지하는지 확인한다.
3. `src/renderer/typeset.rs`와 `src/renderer/height_cursor.rs`의 미주 간격 정책이 `미주 사이`와 `구분선 아래`를 케이스별 예외가 아니라 공통 규칙으로 처리하도록 보정한다.
4. PR #1232 공통 로직 이후에도 한컴 기준 기본값 `미주 사이 7mm`, `구분선 아래 2mm`와 변형 샘플 `미주 사이 20mm`, `구분선 아래 20mm`가 같은 경로로 처리되는지 확인한다.

## 검증 예정

- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- 필요 시 `wasm-pack build --target web --out-dir pkg`
- 한컴오피스 2024 스크린샷 기준으로 작업지시자 시각 확인

## 진행 기록

- 2026-06-02: Stage1 커밋 `cbd68395` 이후 새 스테이지 문서를 만들고, 남은 렌더 간격 문제 분석을 시작했다.
- 2026-06-02: `get_endnote_shape_native`/`apply_endnote_shape_native`의 매핑을 재확인했다. Studio JSON의 `noteSpacing`은 내부 `raw_unknown`(`미주 사이`), `separatorMarginBottom`은 내부 `note_spacing`(`구분선 아래`)으로 유지한다.
- 2026-06-02: HWPX `<hp:noteLine width>`만 일반 선 굵기 코드표와 다르게 `mm * 10`으로 파싱하던 문제를 발견했다. `0.7mm`가 한컴/Studio UI 기준 raw `9`가 아니라 raw `7`로 읽힐 수 있으므로 공통 `parse_hwpx_line_width()`를 사용하도록 보정했다.
- 2026-06-02: 한컴 UI처럼 `길이` 항목에 `사용자` 선택 콤보를 추가했다. 현재 저장 계약은 사용자 길이 수치만 사용하므로 콤보는 표시 정합용으로 고정한다.
- 2026-06-03: PR 준비 전 전체 테스트를 수행했다. Chrome extension 세션 확인은 제외하고 rhwp-studio와 Rust/WASM 검증에 집중했다.

## 검증 결과

- `npm run build --prefix rhwp-studio`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 43개 통과
- `wasm-pack build --target web --out-dir pkg`: 통과. prebuilt `wasm-bindgen`이 없는 플랫폼이라 cargo install fallback 경고가 있었지만 최종 `pkg` 산출물 생성은 성공했다.
- `cargo test --tests`: 통과
- rhwp-studio UI 최종 정합은 작업지시자 시각 확인이 필요하다.
