# Task M100 #717 구현 계획서

## 타이틀

rhwp-studio 표 셀 빈 영역 클릭 hitTest 컨텍스트 고정

## 대상 이슈

- Issue: #717 — `rhwp-studio: 표 셀 빈 영역 클릭 시 커서가 다른 위치로 이동`
- 브랜치: `local/task717`
- 상위 계획서: `mydocs/plans/task_m100_717.md`

## 구현 원칙

본 작업은 클릭 좌표가 표 셀 bbox 내부에 있을 때, 해당 표/셀 컨텍스트를 잃지 않도록 `hit_test_native()`의 셀 hit 경로를 정정한다.

- 좌표 → 문서 위치 변환의 권위는 Rust/WASM `hit_test_native()`에 둔다.
- frontend에서 결과를 보정하는 방식은 피한다.
- 셀 내부 클릭으로 판정된 뒤에는 본문 전체 fallback으로 빠지지 않게 한다.
- 표 경계선 클릭과 표 객체 선택 UX는 변경하지 않는다.
- 렌더링 배치, 페이지네이션, 표 크기 계산은 변경하지 않는다.

## 변경 후보 파일

| 파일 | 목적 |
|------|------|
| `src/document_core/queries/cursor_rect.rs` | `hit_test_native()` 셀 bbox 메타/컨텍스트 정정 |
| `tests/issue_717_table_cell_hit_test.rs` | `exam_social.hwp` 기반 native hitTest 회귀 테스트 |
| `rhwp-studio/e2e/issue-717-table-cell-click.test.mjs` | 필요 시 web 클릭 e2e 검증 |
| `mydocs/working/task_m100_717_stage{N}.md` | 단계별 완료보고서 |
| `mydocs/report/task_m100_717_report.md` | 최종 결과보고서 |
| `mydocs/orders/20260508.md` | 상태 갱신 |

## 단계 1 — 재현 테스트와 현재 hitTest 결과 고정

### 목표

`samples/exam_social.hwp` 1/4쪽 자료 표 제목 행 빈 영역 클릭 좌표가 현재 어떤 `hitTest` 결과를 반환하는지 코드 레벨에서 고정한다.

### 작업

1. `HwpDocument::from_bytes()`로 `samples/exam_social.hwp`를 로드하는 테스트 파일을 추가한다.
2. 이슈 좌표 후보를 직접 호출한다.
   - 페이지: `0`
   - 좌표: `x≈191, y≈356`
   - 기대 대상: `sectionIndex=0`, `parentParaIndex=1`, `controlIndex=0`
3. 실제 반환값의 `parentParaIndex/controlIndex/cellIndex/cellPath/cursorRect`를 확인한다.
4. 같은 페이지의 보조 좌표를 추가로 측정한다.
   - `<보기>` 표(`s0:pi=6 ci=0 3x3`) 빈 영역
   - 페이지 하단 번호 표 영역으로 오인되지 않아야 하는 좌표
5. 실패하는 회귀 테스트 또는 진단 출력을 확보한다.

### 완료 기준

- 현재 코드에서 #717 증상 또는 컨텍스트 이탈 가능성을 드러내는 테스트/진단이 존재한다.
- `mydocs/working/task_m100_717_stage1.md` 작성 후 승인 요청.

## 단계 2 — 셀 bbox 메타 보완 로직 정정

### 목표

`CellBboxInfo`가 다른 표의 동일 `cell_index` TextRun으로 덮이지 않도록 한다.

### 작업

1. `cell_bboxes` 보완 로직의 `cell_index` 단독 매칭을 제거하거나 제한한다.
2. Table 노드가 제공하는 `section_index/para_index/control_index` 메타를 우선 신뢰한다.
3. 보완이 필요한 경우 다음 조건을 모두 만족할 때만 TextRun 메타를 사용한다.
   - 동일 `parent_para_index`
   - 동일 `control_index`
   - 동일 `cell_index`
   - 가능하면 bbox 포함/교차 관계가 성립
4. 중첩 표에서 필요한 경우 `CellBboxInfo`에 `CellContext` 또는 `cellPath` 기반 정보를 추가한다.
5. 클릭 좌표가 여러 셀 bbox에 포함될 경우 가장 안쪽 또는 가장 작은 bbox를 우선 선택한다.

### 완료 기준

- 단계 1의 대상 좌표가 `s0:pi=1 ci=0` 표 컨텍스트로 귀속된다.
- 기존 표 셀 hitTest 테스트가 깨지지 않는다.
- `mydocs/working/task_m100_717_stage2.md` 작성 후 승인 요청.

## 단계 3 — 셀 내부 fallback 차단과 caret clamp 정합화

### 목표

셀 내부 클릭으로 판정된 뒤 해당 셀의 직접 TextRun을 찾지 못해도, 본문 전체나 바탕쪽 표로 fallback하지 않게 한다.

### 작업

1. 클릭한 셀에 TextRun이 있으면 같은 셀의 가장 가까운 run 시작/끝 또는 문자 위치로 이동한다.
2. 클릭 y 범위에 직접 맞는 run이 없으면 같은 셀의 첫/마지막 run으로 clamp한다.
3. 해당 셀에 TextRun이 없지만 표 메타가 있으면 `cellParaIndex=0`, `charOffset=0`으로 셀 내부 진입 결과를 반환한다.
4. 셀 내부 클릭 분기에서 결과를 만들 수 없는 경우에도 본문 전체 `same_line_runs` / `closest line` fallback으로 넘어가지 않게 한다.
5. 표 경계선 클릭은 기존 `input-handler-mouse.ts`의 `isTableBorderClick()` 분기가 계속 처리하도록 반환 형태를 유지한다.

### 완료 기준

- #717 대상 좌표와 보조 표 좌표가 모두 같은 표/셀 내부로 귀속된다.
- 페이지 하단 번호 표(`32`) 쪽 결과가 반환되지 않는다.
- `cargo test --test issue_717_table_cell_hit_test` 통과.
- `mydocs/working/task_m100_717_stage3.md` 작성 후 승인 요청.

## 단계 4 — 회귀 검증, WASM/web 확인, 최종 보고

### 목표

Rust native hitTest 정정을 전체 검증하고, `rhwp-studio` web 경로에서 같은 결과가 유지되는지 확인한다.

### 작업

1. Rust 회귀 테스트를 실행한다.
2. 관련 hitTest 회귀 테스트를 실행한다.
3. 필요 시 WASM 빌드와 `rhwp-studio` 빌드를 수행한다.
4. 가능하면 `rhwp-studio` e2e 또는 dev server에서 대상 좌표 클릭을 확인한다.
5. 최종 보고서와 오늘할일을 갱신한다.

### 검증 명령 후보

```bash
cargo test --test issue_717_table_cell_hit_test
cargo test --test issue_658_text_selection_rects
cargo test --test issue_595
cargo test --lib --release
cd rhwp-studio && npm run build
```

WASM 확인이 필요한 경우:

```bash
docker compose --env-file .env.docker run --rm wasm
```

E2E 확인이 필요한 경우:

```bash
cd rhwp-studio
npx vite --host 0.0.0.0 --port 7700
node e2e/issue-717-table-cell-click.test.mjs
```

### 완료 기준

- 테스트와 빌드가 통과한다.
- web editor에서 대상 표 빈 영역 클릭 시 커서가 해당 표/셀 내부에 진입한다.
- 작업지시자 시각 판정 대기 또는 통과 상태를 최종 보고서에 명시한다.
- `mydocs/report/task_m100_717_report.md` 작성.

## 위험도와 대응

| 위험 | 대응 |
|------|------|
| `cell_index` 반복 사용으로 다른 표 컨텍스트와 충돌 | `parent_para_index/control_index/cellPath` 기준으로 매칭 강화 |
| 중첩 표 bbox가 외곽 표 bbox와 동시에 hit | 가장 작은 bbox 또는 가장 깊은 cellPath 우선 |
| 빈 셀/TextRun 없는 셀에서 커서 위치 계산 실패 | 셀 bbox 기반 `cursorRect`와 `cellParaIndex=0` fallback 반환 |
| 표 경계선 클릭 객체 선택 회귀 | 반환 JSON의 `parentParaIndex/controlIndex/cellIndex` 형태 유지 |
| #595/#658 hitTest 관련 회귀 | 기존 issue 테스트와 관련 e2e 검증 실행 |

## 승인 요청

본 구현 계획서 승인 후 Stage 1부터 진행한다. Stage 1에서는 회귀 테스트/진단 코드 작성과 완료보고서 작성까지만 수행하고, 완료 후 다음 단계 진행 승인을 요청한다.
