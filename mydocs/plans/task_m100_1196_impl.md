# 구현계획서 — Task M100-1196: HWPX 맞쪽 편집 여백 교대

## 설계 요약

대상 결함은 파서 결함과 레이아웃 결함을 분리해 본다.

```text
1. 파서
   최신 upstream 기준으로 HWPX pagePr@gutterType="LEFT_RIGHT"는 이미
   PageDef.binding = BindingMethod::DuplexSided 로 materialize 된다.

2. 레이아웃
   PageAreas::from_page_def() / PageLayoutInfo::from_page_def() 가
   최종 쪽번호 홀짝을 받지 않으므로, DuplexSided 짝수쪽에서도
   좌우 여백을 교대하지 않는다.
```

이번 구현은 2번만 처리한다. #1276에서 확정한 앞쪽 페이지 대응과 최종 `page_number` 기준 홀짝 정책을 유지하기 위해, 페이지네이션 중 임시 layout이 아니라 최종 쪽번호 carry 이후의 `PageContent.layout`을 보정한다.

## 설계 원칙

- `SingleSided`는 기존 결과를 바꾸지 않는다.
- `DuplexSided`는 홀수쪽에서 기존 방향을 유지하고, 짝수쪽에서 좌우 여백을 교대한다.
- `TopFlip`은 이번 이슈에서 좌우 교대 대상으로 다루지 않고 기존 좌표를 유지한다.
- 페이지네이션 중 layout 폭을 흔들지 않는다. 좌우 교대는 같은 body width에서 x 좌표만 이동하는 보정으로 처리한다.
- 최종 쪽번호 기준을 사용한다. 구역 간 page number carry 전의 section-local 홀짝을 기준으로 삼지 않는다.
- body/header/footer/column/footnote 영역은 같은 x delta를 적용해 한 페이지 안에서 일관된 레이아웃을 유지한다.

## Stage 1 — 재현 고정과 baseline 기록

**목표**: #1276 반영 상태에서 #1196만 남아 있음을 구조적으로 확인한다.

변경 후보:

- 소스 수정 없음.
- `mydocs/working/task_m100_1196_stage1.md` 작성.

진단 명령:

```text
cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -p 3
cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -p 4
cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -p 5
cargo test --lib test_parse_page_pr_gutter_type_materializes_hwp5_binding_attr
cargo test --test issue_1271_hwpx_behind_text_table -- --nocapture
```

확인 항목:

```text
1. page 4 가 section 1 본문 시작, page_num=4 상태를 유지한다.
2. page 4/5/6 의 body_area.x 가 현재는 같은 값인지 확인한다.
3. HWPX gutterType 파서 테스트가 이미 통과한다.
4. #1271 회귀 테스트가 통과한다.
```

산출물:

- `mydocs/working/task_m100_1196_stage1.md`

## Stage 2 — PageAreas/PageLayoutInfo 페이지별 여백 계산 API

**목표**: 페이지 홀짝을 반영할 수 있는 최소 레이아웃 API를 추가하고 단위 테스트로 고정한다.

변경 파일:

- `src/model/page.rs`
- `src/renderer/page_layout.rs`

변경 방향:

1. `PageAreas`에 페이지 번호를 받는 API를 추가한다.

```rust
impl PageAreas {
    pub fn from_page_def_for_page(page_def: &PageDef, page_number: u32) -> Self
}
```

2. 기존 `PageAreas::from_page_def(page_def)`는 호환을 위해 유지하고, 내부에서 `from_page_def_for_page(page_def, 1)` 또는 기존 단면 기본 계산으로 위임한다.

3. `DuplexSided` 계산 규칙:

```text
홀수쪽:
  effective_left  = margin_left + margin_gutter
  effective_right = margin_right

짝수쪽:
  effective_left  = margin_right
  effective_right = margin_left + margin_gutter
```

4. `PageLayoutInfo`에도 페이지 번호를 받는 생성 API를 추가한다.

```rust
impl PageLayoutInfo {
    pub fn from_page_def_for_page(
        page_def: &PageDef,
        column_def: &ColumnDef,
        dpi: f64,
        page_number: u32,
    ) -> Self
}
```

5. 기존 `from_page_def()`와 `from_page_def_default()`는 호환 유지한다.

테스트 후보:

- `PageAreas::from_page_def_for_page()` synthetic test:
  - `BindingMethod::SingleSided`: page 1/2 좌표 동일
  - `BindingMethod::DuplexSided`: page 1은 기존 방향, page 2는 좌우 교대
  - `margin_gutter > 0`에서도 body width가 유지되는지 확인
- `PageLayoutInfo::from_page_def_for_page()` test:
  - HWPUNIT → px 변환 후 page 1/2 `body_area.x`가 기대 방향인지 확인
  - 다단 column area가 body x 이동을 따라가는지 확인

검증 명령:

```text
cargo test --lib page_areas
cargo test --lib page_layout
```

산출물:

- `mydocs/working/task_m100_1196_stage2.md`

## Stage 3 — 최종 page_number 기준 PageContent.layout 갱신

**목표**: 페이지네이션 결과의 `PageContent.layout`이 최종 쪽번호 홀짝을 반영하도록 한다.

변경 파일:

- `src/renderer/page_layout.rs`
- `src/document_core/queries/rendering.rs`
- 필요 시 `src/renderer/typeset.rs`
- 필요 시 `src/renderer/pagination/engine.rs`

기본 구현 방향:

1. `PageLayoutInfo`에 기존 layout을 최종 page number 기준으로 이동시키는 helper를 추가한다.

후보 형태:

```rust
impl PageLayoutInfo {
    pub fn apply_page_number_margins(&mut self, page_def: &PageDef, page_number: u32)
}
```

이 helper는 현재 layout의 column 구성은 보존하고, `PageAreas::from_page_def_for_page()`가 계산한 body/header/footer x와 기존 기본 x의 차이만큼 x 좌표를 이동한다.

2. `DocumentCore::paginate()`에서 구역 간 page number carry 보정 이후, `assign_master_pages_for_section()` 호출 전에 section result 전체 layout을 갱신한다.

후보 흐름:

```text
1. section-local pagination 수행
2. page_number_pos 상속
3. 구역 간 page_number carry 보정
4. PageContent.layout 을 최종 page.page_number 기준으로 갱신
5. assign_master_pages_for_section()
6. header/footer 상속 보정
```

3. 이 방식을 기본으로 삼는 이유:

- TypesetEngine과 fallback Paginator 모두 `DocumentCore::paginate()` 결과를 거치므로 한 곳에서 정책을 맞출 수 있다.
- #1276의 masterpage 선택도 같은 위치에서 최종 `page_number` 기준으로 수행된다.
- ColumnDef 변경이 있는 페이지도 기존 `PageContent.layout.column_areas`의 상대 배치를 보존하면서 x만 이동할 수 있다.

4. 필요 시 추가 보정:

- 직접 `TypesetEngine` 또는 `Paginator`를 호출하는 테스트/외부 경로가 있으면 finalize 단계에 같은 helper를 적용한다.
- 다만 첫 구현은 DocumentCore 경로에 집중하고, 직접 호출 경로는 테스트 실패 시 좁게 확장한다.

검증 명령:

```text
cargo test --lib master_page_selection_uses_final_carried_page_number_parity
cargo test --lib page_layout
cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -p 3
cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -p 4
cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -p 5
```

산출물:

- `mydocs/working/task_m100_1196_stage3.md`

## Stage 4 — #1196 통합 회귀 테스트 추가

**목표**: 실제 대상 샘플에서 page 4/5/6의 본문 영역이 홀짝에 따라 교대됨을 회귀 테스트로 고정한다.

변경 파일:

- `tests/issue_1196_hwpx_gutter_left_right.rs`

테스트 구조:

```rust
#[test]
fn onsaemiro_left_right_gutter_alternates_body_area_by_page_parity() {
    // samples/hwpx/[2027] 온새미로 1 본교재.hwpx 로드
    // dump_page_items(Some(3/4/5)) 또는 DocumentCore pagination 접근
    // page 4: section=1, page_num=4 유지
    // page 4 body_area.x > page 5 body_area.x
    // page 6 body_area.x > page 5 body_area.x
}
```

테스트 기준:

- 정확한 px 수치 대신 방향성을 우선 검증한다.
- page 4가 section 1 본문 시작이라는 #1276 전제도 같이 확인한다.
- page 4/5/6 body width는 동일해야 한다.

검증 명령:

```text
cargo test --test issue_1196_hwpx_gutter_left_right -- --nocapture
cargo test --test issue_1271_hwpx_behind_text_table -- --nocapture
```

산출물:

- `mydocs/working/task_m100_1196_stage4.md`

## Stage 5 — SVG/좌표 수동 검증과 회귀 검증

**목표**: dump 기준뿐 아니라 SVG 좌표에서도 본문/바탕쪽 좌표 방향이 PDF 기준과 맞는지 확인한다.

검증 명령:

```text
cargo run --quiet --bin rhwp -- export-svg "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -o output/poc/task1196 --debug-overlay -p 3
cargo run --quiet --bin rhwp -- export-svg "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -o output/poc/task1196 --debug-overlay -p 4
cargo run --quiet --bin rhwp -- export-svg "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -o output/poc/task1196 --debug-overlay -p 5
```

확인 항목:

```text
1. page 4 본문 시작 x 가 page 5 보다 오른쪽에 있다.
2. page 6 본문 시작 x 가 page 5 보다 오른쪽에 있다.
3. page 4/5 바탕쪽 쪽번호 방향은 #1276 후속 보정 상태를 유지한다.
4. paper 기준 배경성 개체가 불필요하게 좌우 교대되지 않는다.
```

회귀 검증:

```text
cargo fmt --check
cargo test --lib
cargo test --tests
```

필요 시 확장:

```text
cargo clippy -- -D warnings
```

산출물:

- `mydocs/working/task_m100_1196_stage5.md`

## Stage 6 — 최종 보고와 orders 갱신

**목표**: 작업 결과와 검증 결과를 문서화하고 오늘 할일 상태를 완료로 갱신한다.

변경 파일:

- `mydocs/report/task_m100_1196_report.md`
- `mydocs/orders/20260604.md`

최종 보고 포함 항목:

- 원인:
  - parser는 이미 `DuplexSided`를 보존했으나 layout이 page parity를 쓰지 않음
- 수정:
  - 페이지별 여백 계산 API
  - 최종 `page_number` 기준 `PageContent.layout` 갱신
- 검증:
  - #1196 전용 테스트
  - #1271 회귀 테스트
  - 전체 회귀 테스트 결과
  - SVG 좌표 확인 결과

완료 전 확인:

```text
git status --short --branch
```

산출물:

- `mydocs/report/task_m100_1196_report.md`
- `mydocs/orders/20260604.md` #1196 상태 완료 갱신

## 제외 범위

- HWPX `gutterType` 파서 중복 수정
- `BindingMethod::TopFlip`의 상하 여백 교대 정책 구현
- 페이지네이션 중 reflow 폭 변경
- paper 기준 배경/전경 shape anchor 좌표 교대
- #1276의 글뒤로 표 분할 정책 재조정

## 작업지시자 승인 요청

본 구현계획이 승인되면 Stage 1부터 진행한다. 첫 단계는 소스 수정 없이 재현 dump, 기존 파서 테스트, #1271 회귀 테스트 확인 및 `mydocs/working/task_m100_1196_stage1.md` 작성이다.
