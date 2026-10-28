# Task 1274 Stage 17 - PR 준비 CI 및 SVG snapshot 정리

## 목적

Task 1274 PR 준비 단계에서 전체 CI급 검증을 수행하고, `cargo test --verbose` 중
확인된 실패를 분류한다.

## 확인된 실패

### `tests/issue_241.rs`

- 실패 테스트: `issue_241_hwpx_stamp_host_paragraph_keeps_flow_line_height`
- 상태: `upstream/devel` 기준에서도 재현된 baseline 실패로 분류한다.
- 이번 task 1274 변경과 직접 관련된 실패로 보지 않는다.

### `tests/svg_snapshot.rs`

- 실패 테스트: `issue_677_bokhakwonseo_page1`
- 관찰: actual SVG와 golden SVG의 차이는 `cell-clip-*` ID 번호가 2씩 당겨진
  snapshot 문자열 차이였다.
- 정규화 검증:
  - `cell-clip-[0-9]+`를 같은 토큰으로 치환한 뒤 diff하면 차이가 없다.
- 판단: 좌표, 텍스트, 도형 배치의 시각 변화가 아니라 clipPath ID drift이다.
  PR 준비를 위해 golden SVG를 현재 출력으로 갱신한다.

## 진행 예정

- `UPDATE_GOLDEN=1 cargo test --test svg_snapshot issue_677_bokhakwonseo_page1`
  로 golden을 갱신한다.
- 생성된 `.actual.svg` 임시 산출물은 제거한다.
- `cargo test --test svg_snapshot` 및 baseline skip 전체 테스트로 재검증한다.

## 수행 결과

### 사전 검증

- `cargo fmt --all -- --check`: 통과
- `cargo build --verbose`: 통과
- `cargo check --target wasm32-unknown-unknown --lib`: 통과
- `cargo test --features native-skia skia --lib --verbose`: 통과
- `cargo clippy -- -D warnings`: 통과

`wasm-pack build --target web --out-dir pkg`는 작업지시자가 수동 WASM build와
시각 검증을 수행하는 규칙에 따라 실행하지 않았다.

### 전체 테스트

- `cargo test --verbose`
  - 실패: `tests/issue_241.rs`
    `issue_241_hwpx_stamp_host_paragraph_keeps_flow_line_height`
  - 이 실패는 `upstream/devel`에서도 재현된 baseline 실패로 이미 기록된 항목이다.

### baseline 제외 재검증

- `cargo test --verbose -- --skip issue_241_hwpx_stamp_host_paragraph_keeps_flow_line_height`
  - 최초 실행: `svg_snapshot::issue_677_bokhakwonseo_page1` snapshot mismatch 확인
  - 원인: `cell-clip-*` ID 번호 drift만 존재했고, ID 정규화 후 diff는 비어 있었다.
  - 조치: `UPDATE_GOLDEN=1 cargo test --test svg_snapshot issue_677_bokhakwonseo_page1`
    로 golden SVG 갱신
  - 확인: `cargo test --test svg_snapshot issue_677_bokhakwonseo_page1 -- --nocapture` 통과
    - 기존처럼 `LAYOUT_OVERFLOW ... overflow=2.5px` 로그는 출력되지만 snapshot은 통과한다.
  - 최종 실행: `cargo test --verbose -- --skip issue_241_hwpx_stamp_host_paragraph_keeps_flow_line_height`
    통과

## PR 준비 판단

Task 1274 변경분 기준으로 추가 실패는 확인되지 않았다. 단, CI에서 `cargo test`를
그대로 실행하면 `issue_241_hwpx_stamp_host_paragraph_keeps_flow_line_height` baseline
실패 때문에 전체 체크는 실패할 수 있다. 이 항목은 task 1274 PR 본문에 baseline
실패로 명시한다.
