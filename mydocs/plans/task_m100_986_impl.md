# 구현계획서 — Task #986

## 이슈

**#986** 가로 방향 문서 열 경우, 표가 밑으로 밀리는 현상

- 브랜치: `issue-986-landscape-table-flow`
- 기준 커밋: 최신 `upstream/devel` `4a76f5a1`
- 수행계획서: `mydocs/plans/task_m100_986.md`

## 최신 재현 결과

제보 첨부 `receipt.hwp` 기준, 최신 `upstream/devel` 에서 다음을 확인했다.

### 기본 경로 (`TypesetEngine`)

```bash
cargo run --release --bin rhwp -- dump-pages /private/tmp/rhwp-issue-986/receipt.hwp
```

- 결과: **3페이지**
- page 1: `ci=2`, `ci=3`, `ci=4 PartialTable rows=0..1`
- page 2: `ci=4 continuation`, `ci=5`, `ci=6 PartialTable rows=0..5`
- page 3: `ci=6 continuation`, `ci=7`, `ci=8`, 빈 문단

즉, 실제 사용 경로인 `TypesetEngine` 에서 이슈가 그대로 재현된다.

### fallback 경로 (`RHWP_USE_PAGINATOR=1`)

```bash
RHWP_USE_PAGINATOR=1 cargo run --release --bin rhwp -- dump-pages /private/tmp/rhwp-issue-986/receipt.hwp
```

- 결과: **2페이지**
- page 1: `ci=2..8` 모든 표가 `Table` 로 배치됨
- page 2: 빈 문단 1개

fallback 은 표 분할은 피하지만, 빈 문단까지 포함한 최종 1페이지 정합은 아직 아니다.
따라서 1차 수정 대상은 기본 경로인 `src/renderer/typeset.rs` 이며,
fallback `src/renderer/pagination/engine.rs` 는 회귀 확인 및 최소 동기화 대상으로 둔다.

## 원인 정리

제보 파일의 첫 문단은 빈 호스트 문단 안에 비글자취급 표 7개를 가진다.

- `ci=2`, `ci=3`: 왼쪽 큰 영역, 서로 세로로 쌓여야 함
- `ci=4`, `ci=5`: 오른쪽 첫 번째 영역, 왼쪽 표와 가로로 겹치지 않음
- `ci=6`, `ci=7`, `ci=8`: 오른쪽 두 번째 영역, 왼쪽 표와 가로로 겹치지 않음
- 모든 표는 `treat_as_char=false`, `wrap=TopAndBottom`, `vert_rel_to=Para`

현재 기본 조판 경로는 각 표의 가로 범위를 고려하지 않고 `current_height` 하나로
표를 순서대로 누적한다. 그래서 오른쪽 표가 왼쪽 표 아래로 밀리고, 실제로는
같은 페이지에 들어갈 표가 행 단위 `PartialTable` 로 분할된다.

렌더 단계도 같은 문제가 있다. `layout_table` 은 `compute_table_y_position` 에서
`raw_y.max(y_start)` 를 사용하고, 호출부는 직전 표가 증가시킨 전역 `y_offset` 을
다음 표의 `y_start` 로 넘긴다. pagination 만 고치면 render tree 위치가 다시
아래로 밀릴 수 있으므로 layout 도 함께 고쳐야 한다.

## 수정 원칙

1. `TopAndBottom` 비글자취급 표라도 **가로 범위가 겹칠 때만** 서로 밀어낸다.
2. 같은 문단/컬럼 안에서 표 점유 박스를 lane 으로 관리한다.
3. pagination 과 layout 이 같은 lane 판단을 사용한다.
4. 기존 겹치는 표, 큰 표, 실제 overflow 표의 분할 동작은 유지한다.
5. `u32` 로 저장된 음수 offset 은 반드시 `as i32` 로 signed 해석한다.

## 변경 파일

### 신규 또는 공용 helper

- `src/renderer/float_placement.rs` 신규 추가

역할:

- `is_para_topbottom_float(common)`:
  - `!treat_as_char`
  - `text_wrap == TopAndBottom`
  - `vert_rel_to == Para`
- `signed_hwpunit(value: u32) -> i32`
- `horizontal_range_for_table(...) -> (f64, f64)`
  - `HorzRelTo::Column/Page/Paper/Para` 와 `HorzAlign` 을 현재 layout 공식과 맞춤
  - 1차 구현은 `common.width` 기준, 필요 시 `MeasuredTable`/resolved column width 보강
- `FloatLaneSet`
  - 기존 lane 중 `x` 범위가 겹치는 lane 의 bottom 만 push 기준으로 사용
  - 겹치지 않는 lane 은 같은 y 영역 병렬 배치 허용

### 기본 조판 경로

- `src/renderer/typeset.rs`

수정 지점:

- `TypesetState`
  - 현재 page/column/paragraph 에 대한 float lane 상태 추가
  - column flush, page advance 시 lane 상태 초기화
- `typeset_table_paragraph`
  - 문단 시작 높이(`para_start_height`)를 각 표 조판에 전달
- `typeset_block_table`
  - `is_para_topbottom_float` 이면 별도 `typeset_para_float_table` 경로로 분기
- 신규 `typeset_para_float_table`
  - 표의 raw top = 문단 anchor + signed vertical offset
  - 같은 문단의 기존 lane 중 가로로 겹치는 lane bottom 으로만 push
  - `lane_bottom <= available` 이면 `PageItem::Table` 로 배치
  - `st.current_height` 는 현재 문단 lane bottom 의 최댓값으로만 갱신
  - 실제 overflow 일 때만 기존 row split 로직으로 fallback

### 실제 레이아웃 경로

- `src/renderer/layout.rs`
- `src/renderer/layout/table_layout.rs`

수정 지점:

- `build_single_column` 또는 `layout_column_item` 주변에 page/column 단위 lane 상태 추가
- `layout_table_item`
  - `is_para_topbottom_float` 표는 layout 호출 전 helper 로 lane top 을 계산
  - `layout_table` 에 넘기는 `y_start` 를 전역 `y_offset` 이 아니라 lane top 으로 설정
  - 반환된 표 bottom 으로 해당 lane 을 갱신
  - 전역 `y_offset` 은 모든 lane bottom 의 최댓값으로만 갱신
- `compute_table_y_position`
  - 기존 `raw_y.max(y_start)` 자체를 크게 바꾸기보다, 호출부에서 lane top 을
    `y_start` 로 넘겨 기존 클램프/캡션 보정을 최대한 재사용
  - 필요한 경우 helper 의 계산과 충돌하지 않도록 para-float 전용 override 를 추가

### fallback 페이지네이션

- `src/renderer/pagination/engine.rs`

수정 방침:

- 기본 경로 수정 후 `RHWP_USE_PAGINATOR=1` 결과를 재확인한다.
- fallback 에도 동일 helper 를 적용할 수 있으면 signed offset 과 lane 판단을 동기화한다.
- 단, fallback 은 현재 표 분할을 이미 피하고 있으므로 기본 경로 안정화 전까지
  대규모 재작성은 하지 않는다.

### debug panic 방어

- `src/renderer/composer.rs`

수정:

- `compose_lines` 에서 `text_end < text_start` 인 경우 빈 문자열로 처리하거나
  안전하게 clamp 한다.
- 목표는 debug build 에서 제보 샘플 로드 중 overflow panic 이 나지 않게 하는 것이다.
- line segment 원본을 임의 재정렬하지 않고, 비정상 range 만 방어한다.

## 구현 단계

### Stage 1 — helper + 단위 테스트

- `float_placement` helper 추가
- 가로 비겹침 lane 은 push 되지 않고, 겹침 lane 은 push 되는 단위 테스트 작성
- signed offset 해석 테스트 작성

검증:

```bash
cargo test float_placement
```

### Stage 2 — TypesetEngine pagination 수정

- `typeset.rs` 에 lane 상태 추가
- `typeset_para_float_table` 구현
- `receipt.hwp` 기본 경로에서 `PartialTable` 이 사라지는지 확인

검증:

```bash
cargo run --release --bin rhwp -- dump-pages /private/tmp/rhwp-issue-986/receipt.hwp
```

기대:

- page 1 에 `ci=2..8` 모두 `Table`
- `ci=4`, `ci=6` 의 `PartialTable` 없음
- 가능하면 page count 1

### Stage 3 — LayoutEngine 위치 동기화

- layout 에도 같은 lane helper 적용
- SVG debug overlay 로 오른쪽 표가 첫 페이지 오른쪽 영역에 남는지 확인

검증:

```bash
cargo run --release --bin rhwp -- export-svg /private/tmp/rhwp-issue-986/receipt.hwp --debug-overlay -o output/issue-986
```

### Stage 4 — debug panic 방어

- `composer.rs` range guard 추가
- debug build 에서 같은 샘플이 panic 없이 로드되는지 확인

검증:

```bash
cargo run --bin rhwp -- dump-pages /private/tmp/rhwp-issue-986/receipt.hwp
```

### Stage 5 — 회귀 테스트 추가

선호안:

- `samples/issue-986-receipt.hwp` 로 제보 fixture 추가
- `tests/issue_986.rs` 추가

테스트 내용:

- `page_count() == 1`
- pagination 결과에 `ci=4`, `ci=6` `PartialTable` 이 없어야 함
- render tree 에서 오른쪽 표의 top 이 왼쪽 큰 표 bottom 아래로 밀리지 않아야 함

대안:

- fixture 포함이 부적절하면 synthetic 문서 또는 helper 단위 테스트 + 수동 검증 기록으로 대체

### Stage 6 — fallback 및 기존 회귀 확인

검증 명령:

```bash
RHWP_USE_PAGINATOR=1 cargo run --release --bin rhwp -- dump-pages /private/tmp/rhwp-issue-986/receipt.hwp
cargo test --test issue_986
cargo test --test issue_712
cargo test --test issue_713
cargo test --test issue_775
cargo test --test svg_snapshot
cargo fmt --all --check
```

최종 전 확인:

```bash
cargo test --release --lib
```

## 완료 기준

- 기본 경로에서 제보 샘플이 1페이지에 배치된다.
- `ci=4`, `ci=6` 이 불필요하게 `PartialTable` 로 분할되지 않는다.
- 오른쪽 표는 왼쪽 큰 표 아래가 아니라 첫 페이지 오른쪽 영역에 렌더된다.
- debug build 에서 overflow panic 이 발생하지 않는다.
- 기존 표 분할/음수 offset 관련 테스트가 통과한다.

## 리스크와 대응

| 리스크 | 대응 |
|--------|------|
| 가로 겹침 판정이 넓거나 좁아 기존 문서가 회귀 | helper 단위 테스트 + `issue_712`, `issue_713`, `issue_775` 회귀 확인 |
| pagination 과 layout 의 x/y 공식이 어긋남 | helper 를 공유하고, layout 은 가능한 기존 `layout_table` 계산을 재사용 |
| 실제 overflow 표가 분할되지 않음 | lane bottom 이 available 을 넘는 경우 기존 split 로직 fallback |
| fallback Paginator 와 기본 TypesetEngine 이 다시 갈라짐 | Stage 6 에서 `RHWP_USE_PAGINATOR=1` 별도 검증 |
| fixture 추가가 부담 | 구현 전 fixture 포함 여부를 최종 확인하고, 필요 시 수동 샘플 경로 검증으로 대체 |

## 산출물

- 구현계획서: `mydocs/plans/task_m100_986_impl.md` (본 문서)
- 단계별 완료 보고서: `mydocs/working/task_m100_986_stage{N}.md`
- 최종 보고서: `mydocs/report/task_m100_986_report.md`

## 승인 요청

위 단계로 소스 수정을 진행해도 될지 승인 요청드립니다.
승인 시 Stage 1 부터 구현을 시작하고, 각 Stage 완료 후 보고서를 작성한 뒤
다음 Stage 진행 승인을 다시 요청하겠습니다.
