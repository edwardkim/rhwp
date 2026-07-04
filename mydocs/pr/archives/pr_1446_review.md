# PR #1446 검토 - 표 셀 드래그 선택과 한컴 호환 표 편집 보정

- PR: https://github.com/edwardkim/rhwp/pull/1446
- 제목: `task 1443: 표 셀 드래그 선택과 한컴 호환 표 편집 보정`
- 작성일: 2026-06-20
- 작성자: `jangster77`
- 관련 이슈: #1443 `rhwp-studio: 표 셀 마우스 드래그 선택 및 한컴 호환 표 편집 단축키 개선`
- base: `devel`
- head: `task_m100_1443`
- 처리 상태: CI 대기

## 1. 요약 판단

로컬 검증 기준으로는 PR 진행 가능 상태다.

PR #1446은 한컴에서 자주 쓰는 표 셀 선택, 셀 크기 조절, 셀 너비/높이 균등화, 모양복사 흐름을
rhwp-studio에 맞춘다. 작업 중 `셀보호2.hwp` 샘플을 기준으로 Shift 개별 셀 resize와 일반 컬럼
resize가 섞일 때 발생한 렌더링 회귀까지 함께 보정했다.

PR 생성 직전 `upstream/devel` `159a221d` 기준으로 rebase했고, macOS 로컬 PR 검증 명령은 모두
통과했다. GitHub Actions 결과는 아직 대기 중이다.

## 2. PR 정보

| 항목 | 값 |
|---|---|
| PR 상태 | open |
| draft | false |
| base | `devel` |
| head | `task_m100_1443` |
| author | `jangster77` |
| mergeable | `MERGEABLE` |
| 변경 파일 | 55 |
| 추가/삭제 | +5071 / -258 |
| 커밋 수 | 24 |

## 3. 변경 범위

주요 변경은 네 갈래로 나뉜다.

| 범위 | 주요 파일 | 내용 |
|---|---|---|
| Studio 표 입력 | `rhwp-studio/src/engine/input-handler-table.ts`, `input-handler-mouse.ts`, `input-handler.ts` | 셀 드래그 선택, 보호 셀 상태 전환, 표 선택 뒤 셀 진입, Shift 개별 resize와 일반 resize 분리 |
| 표 명령/모델 | `src/document_core/commands/table_ops.rs`, `src/model/table.rs` | 보상 resize, local resize row/column hint, 표 common size 보존 |
| 렌더링 | `src/renderer/layout/table_layout.rs`, `border_rendering.rs`, `height_measurer.rs` | local segment 좌표, 셀 안 여백 on/off, 한컴 PDF 기준 외곽 구조 보정 |
| 모양복사 | `rhwp-studio/src/command/commands/edit.ts`, `shortcut-map.ts`, `command-palette.ts` | `Alt+C` 모양복사, 툴바 연결, 일회성 적용 후 자동 해제 |

추가 샘플과 기준 파일:

- `samples/셀보호2.hwp`
- `samples/셀보호2.hwpx`
- `pdf/셀보호2-2024.pdf`

## 4. 핵심 확인 사항

### 4.1 한컴 호환 표 조작

- 마우스로 여러 셀을 드래그 선택할 수 있다.
- 보호 셀은 진입 불가 상태를 표시하고, 보호 셀 클릭 뒤 다른 셀로 이동해도 선택 상태가 남지 않는다.
- 전체 표 선택 뒤 다시 셀을 클릭하면 셀 내부 편집으로 진입한다.
- `셀 너비를 같게`, `셀 높이를 같게`는 전체 표가 아니라 선택된 셀 범위 기준으로 적용된다.

### 4.2 Shift 개별 셀 resize

- Shift+마우스로 개별 셀 segment만 조절한다.
- 한컴처럼 대상 셀 segment가 다른 행의 경계를 넘어갈 수 있다.
- 표 전체 common width/height는 보존한다.
- Shift 개별 resize 뒤 일반 컬럼 resize를 해도 기존 local segment와 영향을 받지 않는 행이 흔들리지 않는다.

### 4.3 셀 안 여백과 표 속성

- `셀보호2.hwp` / `셀보호2.hwpx` / 한컴 PDF 기준으로 셀 안 여백 on/off 표시와 실제 텍스트 리플로우를 맞췄다.
- 표 이동 뒤 raw common object attr과 `table.common` offset이 함께 갱신된다.

### 4.4 모양복사

- Windows/한컴 기준 `Alt+C` 단축키를 사용한다.
- macOS에서도 동일 물리 의도를 유지하도록 Studio shortcut map에 반영했다.
- 툴바의 모양복사 아이콘이 실제 command와 연결된다.
- 일회성 모양복사로, 한 번 적용하면 자동 해제된다.

## 5. 로컬 검증

`upstream/devel` `159a221d` 기준 rebase 후 아래 명령을 수행했다.

| 명령 | 결과 |
|---|---|
| `cargo build --release` | 통과 |
| `cargo test --release --lib` | 통과, 1879 passed / 0 failed / 6 ignored |
| `cargo test --profile release-test --tests` | 통과 |
| `cargo fmt --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `cd rhwp-studio && npx tsc --noEmit` | 통과 |
| `cd rhwp-studio && npm test` | 통과, 75 passed |
| `wasm-pack build --target web --out-dir pkg` | 통과 |

추가 focused 검증:

- `cargo test --profile release-test --test issue_493_cell_attrs -- --nocapture`
- `cargo test --profile release-test --test issue_1073_nested_table_split -- --nocapture`
- `cargo test --profile release-test --test svg_snapshot -- --nocapture`
- `localhost:7700` headless 실제 마우스 드래그 재현
- 작업지시자 수동 시각 검증

## 6. 리스크

| 리스크 | 판단 |
|---|---|
| 변경 규모 | 큼. 표 입력/렌더링/모델/테스트가 함께 변경됨 |
| 표 렌더링 회귀 | Stage 18~20에서 별도 회귀 보정 완료. 통합 테스트와 svg snapshot 통과 |
| Studio 조작 회귀 | headless 실제 드래그와 작업지시자 수동 시각 검증 완료 |
| 샘플 파일 추가 | `셀보호2` 기준 검증에 필요. 테스트와 보고서에서 사용 |

## 7. 권고

GitHub Actions가 통과하면 merge 가능으로 판단한다.

merge 전 확인:

1. GitHub Actions `Build & Test`, CodeQL 계열 체크가 모두 통과하는지 확인한다.
2. PR diff에 의도하지 않은 generated 산출물이 없는지 확인한다.
3. merge 후 #1443이 자동 close되지 않으면 수동 close한다.

