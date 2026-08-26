# 구현계획서 — task_m100_3211 LayoutFrame 기반 LineSeg 재조판

- **이슈**: [#3211](https://github.com/edwardkim/rhwp/issues/3211)
- **선행 검증 PR**: [#4315](https://github.com/edwardkim/rhwp/pull/4315)
- **구현 PR**: [#4755](https://github.com/edwardkim/rhwp/pull/4755)
- **브랜치**: `renderer/lineseg-frame-reflow`
- **기준**: `upstream/devel` `fbca0aa6c`
- **작성일**: 2026-08-13
- **최종 정합**: 2026-08-14
- **정본 병합 후속**: `renderer/frame-canonical-rebase`, 2026-08-22

## 1. 목표

단일 가용 폭으로 문단 전체의 `LineSeg`를 먼저 확정하던 경로를 물리 행 단위의
`LayoutFrame` 재조판으로 바꾼다. 그림 회피 영역이나 표 셀처럼 한 물리 행이 여러 가로 구간으로
나뉘어도 각 구간의 텍스트 진행과 공통 세로 지표를 한 행의 계약으로 유지한다.

저장 `LineSeg`는 새 행의 가로 geometry를 대신하지 않는다. 완전한 단일-segment 행만 보존할 수 있고,
FIRST부터 LAST까지 나뉜 행은 하나의 물리 행으로 다시 계산한다.

### 1.1 정본 우선순위 — cacheability first

저장 `LineSeg`는 과거 조판 결과의 cache다. 저장 record가 있다는 사실이나 텍스트가 저장 폭을 크게
넘지 않는다는 사실만으로 재사용할 수 없다. 재사용 여부는 현재 문단이 실제로 놓일 `LayoutFrame`이
먼저 계산한 물리 행 geometry로 판정한다.

```text
ParagraphBox
  → LayoutFrame 생성
  → 현재 band와 interval을 COMPUTE
  → 저장 FIRST..LAST 행의 cache key와 exact 비교
  → 일치하고 text provenance도 유효: REUSE
  → 불일치하거나 stale: 같은 Frame에서 fresh fill
```

cache key는 물리 행마다 다음 세 값뿐이다.

```text
(interval count, column_start, segment_width)
```

`vertical_pos`, `line_height`, `text_height`, `baseline_distance`, `line_spacing`, overhang은
cache key가 아니다. 이 값들은 현재 metric context에서 다시 계산한다. 따라서
`stored_rows_are_stale()`는 오래된 텍스트를 검출하는 추가 invalidator일 뿐 cacheability의 소유자가
아니다.

Frame이 기존 split 행을 만든 exclusion geometry를 갖지 못한 경우는 mismatch가 아니라
`unmodelled`다. 이 경우 single-slot Frame으로 저장 FIRST..LAST 행을 평탄화하지 않고 기존 전문
owner에게 처리를 돌려준다.

`unmodelled`은 FIRST..LAST 다중 slot에만 한정되지 않는다. exclusion이 한쪽 slot만 남긴 채 여러
행을 지나고 band 바닥에서 끝나면, 저장 결과는 `narrow, narrow, ..., full-width`의 완전한
single-slot 행들로 나타난다. exclusion 없는 scalar Frame의 수평 범위는 행 사이에 바뀌지 않으므로,
서로 다른 `(column_start, segment_width)`를 가진 single-slot 행 sequence 역시 현재 Frame이 갖지
못한 per-band geometry의 증거다. 이 경우에도 exact mismatch 뒤 fresh fill로 평탄화하지 않고
`unmodelled`로 기존 wrap owner에게 돌려준다. 모든 행이 같은 단일 extent인데 현재 Frame과만 다른
경우는 이 증거가 없으므로 strict admission mismatch와 fresh fill 대상에 그대로 남긴다.

## 2. `LayoutFrame` 계약

`LayoutFrame`은 RenderTree 산출물이 아니라 레이아웃이 소유하는 가변 flow geometry다.

```rust
pub(crate) struct LayoutFrame {
    pub(crate) horizontal: Range<i32>,
    pub(crate) top: i32,
    pub(crate) exclusions: Vec<FrameExclusion>,
    pub(crate) current_intervals: Vec<Range<i32>>,
    pub(crate) next_geometry_event: Option<i32>,
    pub(crate) minimum_width: i32,
    rows: Vec<PhysicalRow>,
}
```

### `LayoutFrame::carve()`

```rust
pub(crate) fn carve(&mut self, band_height: i32) -> &[Range<i32>];
```

호출자는 내용에서 계산한 후보 행 높이만 전달한다. 현재 가로 범위, 세로 위치, exclusion과 최소 폭은
Frame이 이미 소유하므로 `Paragraph`나 물리 행 index를 인자로 다시 전달하지 않는다.

`carve()`는 다음 상태만 계산한다.

- 현재 미확정 행의 세로 band
- 왼쪽에서 오른쪽으로 정렬된 가로 구간
- 다음으로 이동할 수 있는 정확한 geometry event

고정 조건:

- band와 exclusion은 half-open 구간으로 교차한다.
- `BothSides`는 양쪽 구간을, `LargestSide`는 선택된 한쪽 구간을 남긴다.
- 사용할 수 있는 구간이 없으면 임의의 `+1` 없이 다음 geometry event에서 재시도한다.
- 최소 폭 미만 구간은 두 개 이상일 때만 제거하여 마지막 구간 하나를 보존한다.
- 더 높은 후보 행으로 다시 호출할 수 있도록 commit 이전 상태만 바꾼다.

`carve()`는 텍스트를 채우거나 `LineSeg`를 생성하지 않고, 행을 commit하거나 Frame을 다음 행으로
전진시키지 않는다.

## 3. 물리 행 transaction

`PhysicalRow`는 한 번 기록되는 `FrameRowMetrics`와 왼쪽에서 오른쪽으로 정렬된 `RowSegment`를
소유한다. `LineSeg`의 FIRST/LAST 경계 비트는 완성된 행을 투영할 때 만든다.

```rust
pub(crate) fn commit_carved_row(
    &mut self,
    metrics: FrameRowMetrics,
    segments: Vec<RowSegment>,
) -> Option<usize>;
```

commit은 segment 수와 각 가로 구간이 마지막 `carve()` 결과와 정확히 같은지 확인한다. 수용된 행의
모든 segment는 같은 `vertical_pos`, `line_height`, `text_height`, `baseline_distance`,
`line_spacing`을 가지며, Frame의 세로 위치는 행 전체에 대해 정확히 한 번만 전진한다.

문단 채움은 caller-owned Frame에서 다음 상태 전이로 수행한다.

```rust
let frame_checkpoint = frame.clone();
let row_frame_checkpoint = frame.clone();
let cursor_checkpoint = cursor.clone();

loop {
    frame.restore_checkpoint(row_frame_checkpoint.clone());
    cursor = cursor_checkpoint.clone();

    let intervals = frame.carve(candidate_height).to_vec();
    let segments = fill_each_interval(intervals, &mut cursor)?;
    let metrics = resolve_row_metrics(&segments);

    if metrics.line_height != candidate_height {
        candidate_height = metrics.line_height;
        continue;
    }

    frame.commit_carved_row(metrics, segments)?;
    break;
}

if transaction_failed {
    frame.restore_checkpoint(frame_checkpoint);
}
```

위 의사 코드는 `layout_paragraph_in_frame()`의 소유권과 rollback 순서를 고정한다. 작은 내부 계산을
그 이름의 새 함수로 추출하라는 요구는 아니다.

### 3.1 저장 cache admission transaction

저장 행의 재사용도 fresh fill과 같은 Frame transaction이다.

```rust
let entry = frame.clone();
let geometry_matches = frame.try_admit_stored_rows(stored, recompute_metrics);

if geometry_matches && !stale {
    return Stored;
}

frame.restore_checkpoint(entry);
return reflow_without_stored_rows(para, frame);
```

고정 조건:

- `try_admit_stored_rows()`는 저장 `column_start`나 `segment_width`를 Frame 입력으로 채택하지 않는다.
  `carve()` 결과와 exact equality만 검사한다.
- accept는 Frame이 계산한 interval과 metric으로 행을 commit한 상태다.
- reject는 문단 진입 checkpoint를 정확히 복원한 뒤 fresh fill로 진행한다.
- geometry가 일치해도 stale text면 admitted 행을 되돌리고 fresh fill한다.
- fresh fill이 실패하면 반쯤 계산된 행을 게시하지 않는다.
- exact miss 뒤의 결론은 호출자 jurisdiction에 속한다. 본문처럼 현재 Frame 입력이 정본인 경로는
  fresh fill하지만, clean imported cell은 padding/owner geometry를 common Frame이 아직 전부 모델링하지
  못하므로 `unmodelled`로 남긴다. 같은 cell도 text/style/geometry mutation owner가 dirty provenance를
  세운 뒤에는 miss를 fresh fill할 수 있다. 즉 clean mismatch를 허용하는 tolerance가 아니라
  “이 Frame이 재계산할 권한이 증명됐는가”의 차이다.
- dirty provenance는 `Paragraph::insert_text_at` 같은 좌표 유지 primitive가 자동 추정하지 않는다.
  field replacement처럼 reflow 없이 저장 partition을 남기는 mutation owner가 명시적으로 세우고,
  model-writing reflow는 `replace_line_segs()`로 새 행과 current 상태를 한 번에 게시한다. load-time 안내문
  정규화와 paint-only 서식은 저장 row를 stale로 만들지 않는다.

### 3.2 Frame 경계와 표시 run 경계

Frame reflow가 소유하는 것은 source text range의 줄 partition과 물리 행 metric이다. 호출자가 이미
해결한 표시 run 의미까지 다시 만들지 않는다. 특히 머리말·꼬리말 field처럼 model 1자가 표시 N자인
run은 `text`에 marker를 유지하고 `display_text`에 문서 context 값을 가진다.

따라서 cache miss 뒤에는 다음 두 결과를 reconcile한다.

- Frame 결과: 새 `char_start` 경계와 행 metric, 현재 `CharShapeRef`와 language lane으로 다시 만든
  일반 run partition
- 기존 composition: 문서 context에서만 해결할 수 있어 `Paragraph` 재구성으로 재현되지 않는
  `display_text`·footnote marker·overlap payload와 그 payload가 소유한 model span

reconcile의 단위는 문단 전체 display surface나 source run 전체가 아니라 **context-owned model span**이다.
기존 composition에서 `display_text`·footnote marker·overlap 중 하나를 가진 span만 찾고, Frame run을 그
span 경계에서 필요한 만큼 나눈 뒤 payload만 덮어쓴다. 나뉜 조각의 `char_style_id`와 `lang_index`는 항상
Frame 결과를 유지한다. source의 stale fallback style/language partition을 이식하는 경로는 없다.
model 1자/display N자 payload는 그 1자 span 전체에만 보존하고, 일반 text span은 Frame이 현재
`CharShapeRef`와 language lane으로 만든 결과를 그대로 쓴다. 따라서 filename/page field의 표시 문자열을
raw control marker로 되돌리지 않으면서도, 한 field의 display 차이가 문단 나머지의 정상 style run을
stale fallback으로 되돌릴 수 없다.

## 4. 연결 범위

### `src/renderer/layout_frame.rs`

- `LayoutFrame`, `PhysicalRow`, `RowSegment`, `FrameRowMetrics`를 레이아웃 계층에 둔다.
- `carve()`와 `commit_carved_row()`를 분리한다.
- 완성된 물리 행만 `project_line_segs()`에서 평탄한 `LineSeg`로 투영한다.

### `src/renderer/composer/line_breaking.rs`

- `FillCursor`가 가로 구간 사이의 token과 UTF-16 진행을 보존한다.
- `fill_one_interval()`은 Frame이 제공한 한 구간만 채운다.
- `layout_paragraph_in_frame()`은 행 높이가 달라지면 cursor와 Frame을 복원해 다시 carve한다.
- 일반 scalar 문단도 지원 범위 안에서는 같은 Frame transaction을 사용한다.

### 표와 그림 band

- 표 셀은 표가 소유한 실제 내용 폭과 padding으로 Frame을 만든다.
- 그림 회피 영역은 host부터 영향받는 마지막 문단까지 하나의 Frame에서 계산한다.
- 문서 편집은 영향받는 그림 band 전체를 shadow 상태에서 계산하고, 성공한 완성 결과만 한 번에 게시한다.
- 열 이동 뒤 geometry가 달라지면 새 열에서 다시 계산하며, 수렴하지 않으면 기존 공개 상태를 유지한다.

RenderTree는 완성된 `LineSeg`만 소비하며 `carve()`나 미확정 행 상태를 소유하지 않는다.

## 5. 검증 계획

핵심 계약은 실제 구현과 같은 모듈의 focused test로 고정한다.

```text
taller_candidate_recarves_before_the_row_is_committed
committed_row_projects_one_complete_lineseg_group_and_advances_once
one_physical_row_projects_each_carved_interval_with_shared_metrics
frame_reflow_projects_two_intervals_as_one_physical_row
frame_reflow_retries_a_taller_row_without_consuming_the_cursor
real_p325_picture_band_matches_the_stored_seven_paragraph_geometry
picture_frame_body_edit_publishes_complete_band_before_recompose
picture_frame_transaction_rejects_shadow_failure_without_publication
a_frame_that_expects_the_stored_rows_admits_them_and_skips_the_reflow
frame_rejected_rows_reflow_without_propagating_cached_source_flags
```

cacheability-first 후속은 다음 순서로 검증한다.

1. debugger에서 `stale=false` stored 경로가 `Stored`를 반환하기 전에
   `LayoutFrame::try_admit_stored_rows()`를 반드시 통과하는지 확인한다.
2. exact match는 저장 text partition을 유지하고, mismatch는 checkpoint 복원 뒤 fresh fill로 가는
   양방향 단위 테스트를 고정한다.
3. context-resolved run 회귀(`#1144`, `#3216`)를 focused test로 고정한다.
4. release-test 전체 실패 집합을 변경 전 기준과 비교한다. exact key 활성화로 새로 드러난 실패는
   cache key를 느슨하게 하지 않고 COMPUTE 결함으로 분류해 한 종류씩 닫는다.

2026-08-22 최초 활성화 측정:

```text
변경 전: 7992 run, 7979 passed, 13 failed, 40 skipped
exact admission 최초 활성화: 7992 run, 7955 passed, 37 failed, 40 skipped
추가 노출: 24 failures
```

첫 COMPUTE 분류(`#4090`, debugger):

```text
HeightMeasurer pi=45 Frame: horizontal=0..48188, exclusions=[]
stored rows: 0+26319 x 6, then 0+48188, all SINGLE
TypesetState: wrap_around_table_para=44, cs=0, sw=26319,
              square band=62.6133..244.4933px
```

즉 저장 cache는 단순히 현재 full-width Frame과 불일치한 것이 아니라, 선행 Square 표가 소유한
exclusion의 단일-slot prefix와 band 종료를 기록하고 있었다. exclusion을 전달받지 못한 측정 Frame의
fresh fill은 이 geometry를 계산할 수 없으므로 `unmodelled` 반환이 정본이다. 이 분류 뒤
`frame_reflow_tests` 11/11, `issue_4090_square_table_left_wrap` 1/1,
`issue_4090_hwpx_tail_page_break` 1/1을 확인했다. 전체 gate 변화량은 다음 전수 실행에서 기록한다.

첫 분류 적용 뒤 전체 gate는 `7960 passed / 32 failed / 40 skipped`였다. 최초 exact admission의
37 failures에서 5건을 닫았고, 변경 전 13 failures를 제외한 추가 노출은 19건이다.

두 번째 COMPUTE 분류(run reconciliation, debugger): 일반 NO_LS 문단에서 Frame composition은
`style 0 × 5, style 1 × 5` 두 run이었지만, context reconciliation 뒤에는 저장 fallback의
`style 0 × 10` 한 run으로 되돌아갔다. 양쪽 모두 `display_text`·footnote marker·overlap이 없었다.
따라서 reconciliation은 문단 전체 display surface 차이로 source run을 허용하지 않는다. source에서
context payload(`display_text`·footnote marker·overlap)를 가진 model span만 골라 Frame run 경계에
payload를 overlay하고, `char_style_id`·`lang_index`는 Frame 계산값을 유지한다. 일반 NO_LS span은 항상
Frame의 `style 0 × 5, style 1 × 5` partition을 유지한다. 한 field가 model 1자/display N자라 해도 그
payload가 소유한 1자 밖의 source fallback style은 이식되지 않는다.

두 번째 분류 적용 뒤 전체 gate는 `7963 passed / 29 failed / 40 skipped`였다. 이 중
`document_core::text_security::tests::scan_cost_stays_linear_as_input_grows` 1건은 10.4초 실행에서 발생한
비결정적 timing threshold 실패이며 Frame 기능 집합과 분리한다. 기능 실패는 28건, 변경 전 13건을
제외한 추가 노출은 15건이다.

세 번째 COMPUTE 분류(column solver quantization, debugger): `#1440`의 최초 mismatch는 exclusion이 없는
`HeightMeasurer` 문단에서 다음과 같았다.

```text
para_shape_id=61, stale=false
stored interval: 850..37418
ParagraphBox input: 850..37418
Frame COMPUTE: 852..37416
ParaShape attr1=0x10000080 (snapToGrid bit 8 clear)
```

차이는 glyph나 cache tolerance가 아니라 4-HWPUNIT quantization을 잘못된 단계에 적용한 결과였다.
column-solver trace는 `÷4×4`가 paragraph `snapToGrid`보다 위에서, **full column width에
먼저** 적용되는 독립 quantization임을 고정한다. 따라서 올바른 순서는 `column width 38268 유지 →
left/right margin 850 적용 → 850..37418`이며, post-margin edge를 각각 ceil/floor하면 안 된다.

한 차례 `snapToGrid` bit 8을 4-HU gate로 연결한 시도는 `#1440`만 통과했지만, 중간 전수 실행
5,422건 시점에 이미 68 failures와 정책연구 215→216쪽 회귀를 만들었다. 이는 같은 문서의 “third
quantization is unconditional and above snapToGrid” 보강과 충돌하는 구현이어서 즉시 철회했다. 최종
구현은 `ParagraphBox::body()`에서 full column width만 4-HU 단위로 내림한 뒤 paragraph margins를
적용하고, `LayoutFrame::carve()`는 전달받은 exact horizontal range를 다시 snap하지 않는다.
`SectionDef::char_grid`/paragraph `snapToGrid` builder·fill pitch는 별도 미구현 입력으로 남긴다. exact
cache key는 그대로다.

세 번째 분류의 정정 구현 뒤 전체 gate는 `7973 passed / 19 failed / 40 skipped`였다. 변경 전 13건을
제외한 추가 노출은 native HWP3 4건과 HWPX Square-OLE 편집 2건으로 좁혀졌다.

네 번째 COMPUTE 분류(legacy/empty carrier origin, debugger): native HWP3의 첫 mismatch는 비어 있는
문단의 `stored=8464..51024` 대 `Frame=0..51024`였고, 다음 mismatch는 실제 텍스트 문단의
`stored=2500..50024` 대 `Frame=5000..50024`였다. 후자는 ParaShape raw `margin_left=10000`,
`indent=-5000`에서 common style이 margin 5000을 만들지만 HWP3 저장 `column_start`는 hanging indent를
포함한 2500을 쓰는 legacy lane이다. common HWP5 `LayoutFrame`에는 그 origin 계약이 없다.

따라서 stored cache route에서 다음 두 경우를 `unmodelled`로 반환한다.

- HWP3 계보의 non-stale stored rows. native HWP3뿐 아니라 HWP3→HWP5 변환본도 legacy stored row
  origin을 보존하므로 `LayoutCompatibilityProfile::hwp3_native_layout() || hwp3_layout()`을
  `ResolvedStyleSet::legacy_hwp3_stored_geometry`에 전달한다.
- text/control이 없는 empty carrier의 stored extent가 exclusion 없는 Frame extent와 다른 경우. 이는
  Square-OLE 편집 뒤 빈 문단처럼 외부 owner가 준 narrowed origin을 보존한다.

NO_LS는 이 guard보다 먼저 Frame fresh fill을 타고, HWP3 계보라도 stale text는 guard를 통과해 fresh
fill한다. nonempty HWP5/HWPX의 uniform mismatch는 계속 exact rejection과 fresh fill 대상이다. Debug
focused에서 HWP3 #1105가 통과했고, Square-OLE #2069의 HWPX 4건도 통과했다.

이 분류의 첫 전체 gate는 `7977 passed / 15 failed / 40 skipped`였다. 기존 13 failures와 독립적인
timing threshold 1건 외에 `#1892` HWP3→HWP5 drawing-group round-trip 1건이 남았다. LLDB에서 재열람한
HWP5는 `provenance={format:Hwp5,hwp3_lineage:true}`였고, non-stale row는 `stored=0..42520`,
`Frame=1200..42520`, exact comparison은 `false`였다. 원본 HWP3만 legacy gate를 탔고 변환본은 exact
rejection 뒤 fresh fill로 들어간 비대칭이 8px 변위를 만들었다. 위 provenance 범위 정정 뒤 `#1892`
4/4가 통과했고, `#1105`는 변경 전 기준과 같은 9/14 pass(기존 5 failures)를 유지했다.

최종 release-test 전수 gate는 `7992 run: 7979 passed / 13 failed / 40 skipped`로 변경 전 실패
집합과 정확히 일치했다. exact cache key를 유지하면서 최초 노출 24건을 모두 닫았고, timing threshold
실패도 재실행에서는 발생하지 않았다.

Gestell lifecycle review에서 section edit 뒤 provenance가 사라지는 한 경로가 발견됐다.
`DocumentCore::rebuild_section()`이 `DocInfo`만 받는 `resolve_styles()`로 `self.styles`를 교체하면서
`legacy_hwp3_stored_geometry=false`로 되돌리고 있었다. 초기 load와 full rebuild에만 mutable
post-patch를 두는 방식은 이 수명주기를 닫지 못한다.

정정 구현은 `resolve_styles_for_document(document, dpi)`를 style-resolution의 document-aware 단일
constructor로 두고, `DocumentCore::rebuild_resolved_styles()`가 모든 `self.styles` 교체를 소유한다.
초기 load, DPI 변경, full derived-state rebuild, section rebuild, HTML/table/style cache 갱신도 같은
경계를 사용한다. HWP3 formatting 회귀 테스트는 paint-only underline 편집이 실제
`rebuild_section()`을 통과한 뒤 stored row가 불변이고 provenance flag가 유지되는지 확인한다. 같은
row에서 flag만 지운 reversal probe는 `Reflowed`, 유지한 production probe는 `unmodelled/None`을
반환해 테스트 신호도 고정한다.

이 lifecycle 정정 뒤 release-test 전수 gate는 새 회귀 테스트 1건을 포함해
`7993 run: 7980 passed / 13 failed / 40 skipped`였다. 실패 집합은 정정 전 baseline 13건과
동일하며, 새 HWP3 edit-lifecycle 테스트를 포함한 Frame focused test는 12/12다.

다섯 번째 분류(`#5765`, sample16 HWP5 9 failures)는 **downstream pagination/flow**다. LLDB의
동일 문단 pi=460 비교 결과는 다음과 같다.

```text
native HWP3 stored rows: 6
text_start: 0,54,107,159,211,261
metrics: line_height=1300, line_spacing=780, baseline=1105 (전 행 동일)
geometry: column_start=4000, segment_width=46024 (전 행 동일)
vpos: 65116,67196,69276,0,2080,4160  -> line 3 physical-page reset

missing-LineSeg HWP5 Frame rows: 6
text_start: 0,53,106,159,211,262
metrics: 1300/780/1105, geometry: 4000+46024
pagination operands: current=797.973, available=967.253,
                     5/6 threshold=806.044, trailing spacing=10.4
```

즉 glyph advance는 양쪽 모두 6행이며 source fragment 경계(line 3)를 바꾸지 않고, typography/row
metrics와 Frame geometry도 native 저장값과 일치한다. 변환본에서 저장 LineSeg 자체가 빠지면서
`69276→0` reset만 사라졌고, `missing_lineseg_trailing_line_break()`가 current height를 threshold보다
8.07px 작다고 보아 `forced_page_break_line=None`을 반환한 것이 최초 분기다. LLDB에서 그 Option의
메모리만 `Some(3)`으로 바꾸자 실패하던 page-count/페이지 항목 테스트 전체가 통과해 인과를 확인했다.

정정은 HWP3-converted missing-LineSeg inference의 fill 비교에 page-end fit이 이미 제외하는 마지막
`line_spacing`을 같은 credit으로 더한다. 위 operands에서는 `797.973+10.4=808.373`으로 threshold를
넘고 균등 fragment 계산이 native와 같은 line 3을 돌려준다. 일반 HWPX inference와 exact Frame cache
admission은 바꾸지 않는다. focused 결과는 `issue_1105` 14/14, `#2158` 2/2, `#1035` 4/4,
`#1086` 4/4, 인접 `#1116` 13/13이다.

최종 release-test 전수 gate는 `7993 run: 7989 passed / 4 failed / 40 skipped`였다. `#5765`
sample16 page-count 9건이 모두 실패 집합에서 제거됐고, 남은 4건은 기존의 `overflow_cell`, `#3128`,
`#4956`, `#2308`뿐이다.

여섯 번째 분류(`#5705`, `overflow_cell_baseline`)는 **downstream nested-table fragment viewport
ownership**이다. 이전 handoff의 `eu=MAX`, 271.3px/483.5px 종료 조각은 현재 코드에서 세 번째
조각으로 남아 있지만, live LLDB의 최초 26줄 소실은 그 직전 nonterminal 조각에서 발생했다.

```text
host: section 0, parent paragraph 172, RowBreak table row 27, cell 80
outer cell cuts:
  0..19    inner=327.213
  19..68   inner=887.707   <- 최초 소실 조각
  68..MAX  inner=271.347   terminal

mixed_nested_split_from_cut(19..68):
  total=1444.933, offset=327.213, visible=863.707
  terminal=false
  terminal_rowbreak_source_cursor=true
  computed row_offset_within_start=327.213
  returned offset_within_start=0
```

`terminal_rowbreak_source_cursor` 예외는 주석·도메인 이름대로 종료 child에서 이미 소비한 source
cursor를 재생하지 않기 위한 규칙인데, 반환식에 `terminal` 조건이 없어 nonterminal `19..68`에도
적용됐다. 그 결과 consumer가 앞 조각 327.213px prefix를 다시 배치했고, 깊이 2·3 셀 줄 26개가
page bottom 밖으로 밀렸다. #4889의 `compensation_would_consume_fragment`는 이 경로의
`single_cell_nested_continuation=false`라 발화하지 않는 것이 맞다.

LLDB에서 consumer가 받는 split의 `offset_within_start`만 0→327.213으로 바꾸자 page 28의 26개
`LAYOUT_OVERFLOW_CELL` 방출이 전부 사라졌다. 수정은 cursor-zero 예외에 `terminal`을 추가해 실제
종료 조각의 #3128/76076 계약은 유지하고, nonterminal은 계산한 source offset을 보존한다. glyph
advance, line metrics, Frame geometry, exact cache admission은 바꾸지 않는다.

focused 결과: `86712_regulatory_analysis.hwp` overflow 27→1,
`issue1891/86712_regulatory_analysis.hwpx` 27→1(baseline 허용 2), 전체
`overflow_cell_baseline` 698문서 pass(0 아닌 15종, 총 592줄). 인접 `issue_1891` 4/4,
`issue_1486` 6/6, `issue_3820` 4/4, `issue_4326` 2/2가 통과했다.

최종 release-test 전수 gate는 `7993 run: 7990 passed / 3 failed / 40 skipped`였다.
`overflow_cell_baseline`이 실패 집합에서 제거됐고, 남은 것은 `#3128`, `#4956`, `#2308`이다.

일곱 번째 분류(`#5703`/`#3128`)는 두 개의 **typography row-advance / downstream
fragment-flow** 소유권 결함이고, glyph advance나 Frame horizontal geometry 결함이 아니다.
현재 tree의 LLDB trace는 p34 outer table owner 325를 다음처럼 재확인했다.

```text
layout_partial_table_cells:
  start_row=6, end_row=7, start_cut=[1,37], end_cut=[]
  row_heights=[23.28×5,44.08,374.986667]
  resolved row 6=1038.773333

child terminal paragraph 12, line 1:
  line_height=1300HU, line_spacing=520HU, baseline=1105HU
  projected CellUnit height=1300HU (520HU lost at child-local terminal fold)

outer terminal empty-host paragraph:
  stored LineSeg line_height=1300HU, line_spacing=260HU
```

먼저 glyph/style provenance를 별도로 판정했다. 문제의 219자 child 문단은 parser 직후 이미
`(0,18),(2,128),(4,129),(28,128),(29,130),(33,131),(73,128),(105,132),(140,128),
(185,130),(218,128)` CharShape 원장을 가진다. 이 run을 style 18 하나로 강제해 7행을 만드는 것은
source를 버리는 보상이다. 현재 Frame의 6행은 공식 PDF와 p33 행 수가 같고, p34의 모든 보이는
줄 경계도 PDF와 일치한다. token/prefix 누적의 0--3HU 차이는 경계를 하나도 바꾸지 않았다.

실제 첫 결함은 `nested_table_mixed_fragment_heights()`가 child 내부에서는 마지막인 줄의 520HU
spacing을 버린 뒤, outer terminal continuation에서도 그 줄을 전체 flow의 마지막으로 취급한 것이다.
`projected_terminal_line_spacing()`은 같은 child source paragraph의 직전 line unit이 보존한
`height - content_height`만 복원한다. content height가 다르거나 한 줄뿐이면 추정하지 않고 0을
반환한다. long-terminal source cursor는 exact 상태로 유지하고 short-child와 같은 4px mixed clip
guard만 더한다. LLDB mutation에서 520HU만 복원하면 bottom 452→459로 이동했고, spacing+guard는
continuation bottom을 462.02px(PDF 463±2)로 옮겼다.

두 번째 결함은 child 뒤의 outer-cell empty host Enter가 가진 260HU line spacing이 terminal
continuation 완료 시 사라진 것이다. 이것은 표 border 높이가 아니라 다음 body item까지의 flow
advance다. `native_terminal_child_host_line_spacing()`이 동일한 native HWP5 구조 gate에서 저장값을
읽고, typeset과 layout이 terminal fragment 뒤에서 한 번만 소비한다. `last_item_content_bottom` 뒤에
적용하므로 continuation bbox는 462.02px에 남고, direct-benefit table만 PDF 허용대에 들어간다.

focused 결과는 `#3128` 2/2, `#1891` 4/4, `#2439` 4/4, `#2308` guard 1/1이다. `#2308`
derived-state의 기존 PDF geometry failure는 그대로 남아 별도 분류다. 최종 release-test 전수 gate는
`7993 run: 7991 passed / 2 failed / 40 skipped`였고, 남은 실패는 기존 `#4956`, `#2308`뿐이다.

여덟 번째 분류(`#2308` p34 nested fragment)는 **downstream child paint viewport / parent bbox
aggregation**이다. #3128에서 복원한 row advance는 outer table flow를 고쳤지만 child 1×1 table의
paint viewport는 여전히 짧았다. 현재 tree LLDB의 exact operands는 다음과 같다.

```text
mixed_nested_split_from_cut(cell r6/c1, cut 37..MAX, para 0):
  total=991.493333, offset=636.813333, terminal=true
  flow_visible=354.680000
  first visible advance=24.266667 (paint core=17.333333)
  last visible paint core=17.333333
  current visible_height=354.68 + 24.266667 - 4 = 374.946667
  PDF child viewport=388.3
```

PDF값은 독립 상수의 이식이 아니라 기존 원장의 두 노출 edge에서 산출된다.
long native terminal child는 exact source cursor 때문에 선두 advance를 복원하는 동시에 terminal paint
core도 flow slice 밖에서 보존해야 한다. 각 exposed edge의 mixed-flow allowance 4px를 제외하면
`354.68 + 24.266667 - 4 + 17.333333 - 4 = 388.28px`다. LLDB에서 `visible_height` 하나만
388.28로 바꾸자 실패 테스트가 통과했고 text/row cut/pagination은 불변이었다. 구현은 이 식을
`terminal && terminal_rowbreak_source_cursor && !native_short_terminal_child`에만 적용한다.

child paint clip이 388.28로 늘어난 뒤 partial-table bbox aggregator가 clipped `TableCell`의 확장된
clip bottom을 새 parent flow로 합쳐 outer table을 465.4까지 키웠다. 이는 해당 함수 주석의
"clipped general flow descendant는 RowBreak viewport를 확대하지 않는다" 계약과 모순이었다. 다만
일반 clipped-cell 전체를 logical bottom으로 cap한 첫 시도는 #2007 p14 ancestor clip을 4.2px
잘라 focused test가 즉시 기각했다. 최종 구현은 동일한 long-terminal native 구조와 실제 terminal
continuation에만 paint clip과 logical parent bbox를 분리하고, direct drawing의 기존 current-page
확장은 유지한다. 이로써 nested child bbox는 388.28, outer continuation bbox는 #3128의 462.02를
각자 보존한다.

attribution은 glyph advance/typography/Frame geometry가 아니라 downstream fragment paint 및 bbox
ownership이다. focused 결과는 `#2308` 5 pass/1 ignored, `#3128` 2/2, `#2007/#4159` 15/15,
`#4326` 2/2, `#3820` 4/4다. 최종 release-test 전수 gate는
`7993 run: 7992 passed / 1 failed / 40 skipped`; 남은 실패는 `#4956` 하나다.

아홉 번째 분류(`#4956`)는 **downstream synthetic-row justification / derived-state** 소유권
결함이다. glyph advance, typography row metric, Frame geometry, page pagination은 원인이 아니다.
현재 tree LLDB에서 실제 오른쪽 초과를 만든 첫 노드는 머리말·꼬리말이 아니라 body `para_index=31`
첫 줄의 두 번째 run `"검색엔진을 호출할 때 "`였다. 편집 직후 IR은 이미 새 폭으로 두 줄을 발행했고
첫 줄 geometry도 `column_start=0`, `segment_width=17576HU=234.346667px`로 새 body box와 같았다.

```text
current corrected fill, para 31 line 0:
  text = "search\t\t검색엔진을 호출할 때 사용하는 형식"
  projected boundary = before `사`; first rendered Korean run bytes=30, trailing separator present
  available_width=234.346667, tab advance before Korean run=70.0
  extra_word_spacing=24.673333
  run right=427.5, body right=396.9

known passing pre-canonical path (b14db80da), same line box/tabs/style:
  boundary = after `사`; first rendered Korean run bytes=33, no trailing separator
  extra_word_spacing=10.448889
```

이 차이는 canonical fill을 되돌릴 근거가 아니다. 옛 경로는 fit predicate가 거절한 `사`를 별도
break-point recorder가 다시 받아 줄에 넣던 이중 판정이고, 현재 단일 predicate는 `76076` 공식 PDF의
행 경계를 맞춘다. Frame 경로만 끄고 scalar fallback으로 보내도 현재 실패가 그대로였고, candidate
letter-spacing trim을 LLDB에서 0으로 만들어도 경계/실패는 불변이었다.

실제 producer/consumer 어긋남은 soft-wrap이 다음 줄 앞 separator를 소비하지만 `LineSeg` projection은
다음 줄 시작점만 저장할 수 있어 그 공백이 앞 synthetic row의 run 끝에 남는다는 점이다.
`compute_line_extra_spacing()`은 후행 공백을 분배 slot과 natural width에서 제외했지만 TextRun painter는
보존된 모든 공백에 `extra_word_spacing`을 적용했다. `TAG_IMPLEMENTATION_PROPERTY`인 비마지막 synthetic
row가 실제 후행 공백을 렌더할 때만 그 공백을 slot/natural width 양쪽에 포함하도록 producer 계약을
맞췄다. 저장 row와 마지막 줄의 기존 trailing-space 규칙은 바꾸지 않는다. 수정 후 같은 LLDB operand는
`extra_word_spacing=14.448889`이고 #4956 6/6이 통과한다.

현재 source로 새로 저장한 동일 narrow-document probe를 다시 읽어도 수정 전에는 동일한 para31 operands와
427.5px 초과를 재현했다. 따라서 기존 handoff의 “save/reload는 good” 가설은 현재 tree에서는 기각됐고,
cache invalidation이나 serialization normalization을 고치는 문제가 아니다. focused neighbor는 #1891 4/4,
#2308 5 pass/1 ignored, #4326 2/2, #3820 4/4이고, 기존 justification unit 4/4도 통과했다.
최종 release-test 전수 gate는 `7993 run: 7993 passed / 0 failed / 40 skipped`다. 시작 시 남아 있던
13건과 exact admission이 처음 드러낸 24건을 모두 닫았고 exact key는 그대로다.

이 24건은 저장 cache를 무검증으로 다시 허용할 근거가 아니다. primary 완료 조건은 exact admission을
유지한 채 추가 실패를 0으로 만들고, 기존 13건의 원인 추적을 재개하는 것이다. 첫 번째 공통 결함은
Frame reflow 뒤 context-resolved `display_text`를 잃던 문제였고, 해결 계약은 context-owned span에
payload만 overlay하고 Frame의 style/language partition을 보존하는 것이다. `#1144` 4건과 `#3216`
5건, mixed CharShapeRef + model-one/display-many field 교차 test로 고정한다. 전체 gate 수치는 다음 전수
실행에서 다시 기록한다.

### 5.3 upstream/devel 동기화와 최종 검증

`upstream/devel` `61e439043`을 기준으로 branch를 restack했다. 첫 고유 commit은
`try_admit_stored_rows()`와 exact-admission/rollback 행동 test 하나만 담아 Gestell first gate를
통과한다. 이후 upstream이 추가한 fallback paginator도 Typeset과 같은
`missing_lineseg_fragment_boundary()`를 사용하며, HWPX reset provenance와 HWP3-converted provenance를
private pagination context로 각각 전달한다. public `PaginationOpts`와 `ResolvedStyleSet`에는 provenance
필드를 추가하지 않았다.

Gestell review에서 확인·수정한 lifecycle은 다음과 같다.

- context reconciliation은 context-owned span의 payload만 overlay하고 Frame style/language partition을
  보존한다.
- text/style/geometry mutation owner만 저장 partition을 dirty로 만들거나 model-writing reflow로 현재 행을
  게시한다. field replacement는 obsolete rows를 제거해 live/save가 같은 NO_LS Frame 경로를 쓴다.
- clean imported cell의 exact miss는 common Frame의 입력 jurisdiction이 증명되지 않아 `unmodelled`, 같은
  cell의 proven mutation miss는 fresh fill이다.
- picture-band 결과는 fresh implementation rows이며 저장 row flag/vpos reset provenance를 상속하지 않는다.
- bulk replace와 auto-number/field/path edit는 reflow 뒤 body/cell vpos ladder, recomposition, pagination,
  serialization까지 닫는다.

최종 Gestell 결과는 `PASS — No remaining material finding`이다. 최종 unchanged code candidate
`187cf5519`의 release-test는 run ID `52e86831-05b9-4248-8321-9f608721c114`,
`7964 run: 7964 passed / 0 failed / 41 skipped`다. `cargo fmt --all -- --check`와
`git diff --check`도 통과했다. 이 gate는 사전부터 존재하던 HFT/font-metric/golden 작업을 stash로 격리한
상태에서 실행했으므로 그 별도 변경의 검증으로 해석하지 않는다.

PR 후보는 다음 저장소 gate를 순차 통과시킨다.

```text
focused LayoutFrame, line-breaking, table-owner와 picture-band tests
release-test 전체 회귀
Native Skia 3종
cargo fmt --all -- --check
git diff --check
cargo clippy --all-targets -- -D warnings
cargo test --doc
rhwp-studio TypeScript와 unit tests
wasm-pack build --target web --out-dir pkg
```

## 6. 범위 밖

- `reflow_linesegs_on_demand()`의 persistence 소유권 변경
- 지원되지 않는 복수 그림 topology의 추정 처리
- Shape, Tight, Through 전체 geometry 지원
- 글꼴 shaping과 kerning에 따른 작은 줄 경계 차이
- accept arm의 recomputed vertical metric을 저장 IR에 write-back하는 단계

저장 `LineSeg` cache의 재사용 판정은 더 이상 범위 밖이 아니다. 이 계획의 primary 접근은
`LayoutFrame` exact admission을 모든 지원 stored 경로의 첫 gate로 두는 것이다. 반면 accept arm의
metric write-back은 cacheability와 분리한다. 두 변경을 한 번에 열면 geometry key mismatch와 metric
settlement 회귀를 구분할 수 없기 때문이다.

마지막 항목은 구조적 Frame 정확성과 섞지 않고 #4439에서 별도로 추적한다.
