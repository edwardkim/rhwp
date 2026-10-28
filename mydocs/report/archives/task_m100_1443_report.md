# Task M100 #1443 최종 보고서

- 이슈: #1443 `rhwp-studio: 표 셀 마우스 드래그 선택 및 한컴 호환 표 편집 단축키 개선`
- 브랜치: `local/task_m100_1443`
- 기준 브랜치: `upstream/devel` `159a221d`
- 최종 HEAD: `94537560`
- 상태: PR 준비 완료

## 1. 해결 범위

이번 작업은 사용자 피드백으로 들어온 표 편집 조작을 한컴 동작에 맞추는 것을 목표로 진행했다.

- 표 셀을 마우스로 드래그해 여러 셀을 선택할 수 있게 했다.
- 보호 셀 클릭, 보호 셀 hover, 보호 셀에서 다른 셀로 이동하는 상태 전이를 보정했다.
- 표 전체 선택 뒤 셀 내부 편집 진입을 복구했다.
- 표/셀 속성 진입과 모달 크기, 탭 전환 표시를 보정했다.
- 선택 셀 기준 `셀 너비를 같게`, `셀 높이를 같게` 동작을 한컴처럼 선택 범위에만 적용되도록 했다.
- `Alt+C` 모양복사 단축키와 편집 메뉴/툴바 연결을 추가하고, 붙여넣기 후 자동 해제되는 일회성 동작으로 정리했다.
- Shift+마우스 셀 경계 조절에서 한컴처럼 개별 셀 segment만 움직이고 표 외곽 크기는 유지되도록 했다.
- Shift 개별 resize 후 일반 컬럼 resize가 기존 local segment에 막히거나 다른 행을 흔드는 회귀를 보정했다.
- `셀보호2.hwp` / `셀보호2.hwpx` 샘플과 한컴 PDF 기준으로 셀 안 여백, 표 외곽 구조, 텍스트 리플로우를 검증했다.

## 2. 주요 구현 요약

### 2.1 Studio 입력/선택

- `rhwp-studio/src/engine/input-handler-table.ts`
  - 마우스 드래그 셀 선택, Shift 개별 셀 경계 resize, 일반 컬럼/행 resize의 대상 계산을 분리했다.
  - local resize 이력이 있는 표에서 일반 resize를 수행할 때, 전체 border line이 아니라 현재 target/neighbor bbox를 기준으로 clamp를 계산하도록 했다.
- `rhwp-studio/src/engine/input-handler-mouse.ts`
  - 표 선택/셀 선택/보호 셀 클릭 후 상태 전환을 정리했다.
- `rhwp-studio/src/engine/table-resize-renderer.ts`
  - Shift 개별 resize 중 marker를 전체 표가 아니라 대상 셀 segment 범위에 맞게 표시하도록 했다.

### 2.2 표 모델/명령

- `src/document_core/commands/table_ops.rs`
  - 보상 resize에서 표 common width/height가 변하지 않도록 보존했다.
  - render width hint와 local resize row/column hint를 저장해 개별 segment 렌더링을 안정화했다.
- `src/model/table.rs`
  - local resize transient hint를 모델에 추가했다.
- `src/renderer/layout/table_layout.rs`
  - local resize hint가 있는 행/열은 셀 순서 기반 좌표 계산을 사용해 다른 행의 전역 column fallback이 침범하지 않도록 했다.
- `src/renderer/layout/border_rendering.rs`
  - 분리된 셀 segment와 외곽선 렌더링 정합을 보정했다.

### 2.3 셀 속성/렌더링

- `src/renderer/layout/table_cell_content.rs`, `src/renderer/height_measurer.rs`
  - 셀 안 여백 지정 on/off에 따른 텍스트 리플로우와 높이 측정을 맞췄다.
- `rhwp-studio/src/ui/table-cell-props-dialog.ts`
  - 한컴과 유사하게 표/셀 속성 모달 크기를 고정하고 탭 이동 시 레이아웃이 출렁이지 않게 했다.

### 2.4 모양복사

- `rhwp-studio/src/command/shortcut-map.ts`
  - 한컴 기준 `Alt+C` 모양복사 단축키를 추가했다.
- `rhwp-studio/src/command/commands/edit.ts`, `rhwp-studio/src/ui/command-palette.ts`
  - 메뉴/툴바의 모양복사 아이콘을 실제 기능과 연결했다.
  - 모양복사는 한 번 적용하면 자동 해제되는 일회성 상태로 정리했다.

## 3. 추가 샘플과 회귀 테스트

- 추가 샘플:
  - `samples/셀보호2.hwp`
  - `samples/셀보호2.hwpx`
  - `pdf/셀보호2-2024.pdf`
- 테스트 확장:
  - `tests/issue_493_cell_attrs.rs`
    - 셀 보호/필드/양식 편집 가능 속성
    - 셀 안 여백 on/off
    - 표 common size 보존
    - Shift 개별 resize와 일반 resize 조합 회귀
    - 한컴 저장 샘플 기준 표 속성 정합

## 4. 검증 결과

`upstream/devel` `159a221d` 기준 rebase 뒤 아래 검증을 완료했다.

- `cargo build --release`
  - 통과
- `cargo test --release --lib`
  - 통과: 1879 passed, 0 failed, 6 ignored
- `cargo test --profile release-test --tests`
  - 통과
  - 포함 확인: `tests/issue_493_cell_attrs.rs` 14 passed, `tests/svg_snapshot.rs` 8 passed
- `cargo fmt --check`
  - 통과
- `cargo clippy --all-targets -- -D warnings`
  - 통과
- `cd rhwp-studio && npx tsc --noEmit`
  - 통과
- `cd rhwp-studio && npm test`
  - 통과: 75 passed
- `wasm-pack build --target web --out-dir pkg`
  - 통과
  - `wasm-bindgen` prebuilt 미지원으로 cargo install fallback 경로를 사용했으나 최종 빌드는 성공

Stage별 focused 검증에서도 다음을 별도로 통과했다.

- `cargo test --profile release-test --test issue_493_cell_attrs -- --nocapture`
- `cargo test --profile release-test --test issue_1073_nested_table_split -- --nocapture`
- `cargo test --profile release-test --test svg_snapshot -- --nocapture`
- `localhost:7700` headless 실제 마우스 드래그 재현
- 작업지시자 수동 시각 검증

## 5. Git 상태

- `upstream/devel...HEAD`: behind 0, ahead 23
- 워크트리: PR 문서 생성 전 기준 clean
- PR 생성 전 권장 원격 브랜치:
  - `task_m100_1443`

## 6. PR 생성 메모

PR 본문에는 `Closes #1443`를 포함해 merge 시 issue가 자동 close되도록 한다.

