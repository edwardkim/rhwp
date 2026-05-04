# 구현 계획서 — Task #577

**제목**: 셀 내부 단독 TopAndBottom 이미지 1라인 오프셋 제거
**브랜치**: `local/task577`
**기반 수행 계획서**: `task_m100_577.md`

---

## 1. 원인 정밀 분석

### 1.1 코드 경로

이미지 ⑤(`bin_id=17`, `tac=false`, `wrap=TopAndBottom`, `vert_rel_to=Para`)는 `table_cell_content.rs`가 아닌 `table_layout.rs`의 셀 처리 루프를 거친다:

```
src/renderer/layout/table_layout.rs
  L1413  let para_y_before_compose = para_y;     // 셀 단락 시작 y 저장
  ...
  L1500± para_y = self.layout_composed_paragraph(...);  // 텍스트(빈 라인 1줄) 레이아웃 → para_y 가 line_height 만큼 advance
  ...
  L1547  Control::Picture(pic) =>
  L1548    if pic.common.treat_as_char { ... }     // (TAC 분기 — 본 건과 무관)
  L1624    else {                                   // 비-TAC 분기 — 본 건 진입
  L1629      let cell_area = LayoutRect { y: para_y, ... };
  L1634      let (pic_x, pic_y) = self.compute_object_position(
  L1637        ..., para_y, para_alignment);
  L1646      para_y += pic_h;
```

### 1.2 좌표 산식 (현재)

`compute_object_position` (`picture_footnote.rs:162`)에서 `vert_rel_to=Para`, `vert_align=Top`, `v_offset=0` 인 경우:

```rust
y = ref_y + v_offset = para_y + 0 = para_y
```

문제는 이 시점의 `para_y`가 `layout_composed_paragraph`가 빈 anchor 라인 1줄(lh=1150 HU ≈ 15.32 px) 만큼 이미 advance 시킨 값이라는 점.

```
pic_y = cell_y + pad_top + line_height
      = 896.27 + 3.78 + 15.32 = 915.37  (관측값과 정확히 일치)
```

HWP 의도는 anchor 라인이 이미지에 의해 displaced 되므로(`TopAndBottom`), 이미지는 paragraph 시작 위치(`para_y_before_compose`)에 anchor 되어야 함.

### 1.3 근본 수정 위치

`table_layout.rs:1624..1648` 의 비-TAC Picture 분기. 여기서 `compute_object_position` 호출 시 `para_y` 대신 anchor 시점 좌표(`para_y_before_compose` 또는 `seg.vertical_pos` 보정 후)를 전달.

### 1.4 적용 범위 한정

회귀 방지를 위해 새 좌표를 적용할 조건:

1. `pic.common.text_wrap == TopAndBottom`
2. `pic.common.vert_rel_to == VertRelTo::Para`
3. (안전책) 셀 단락이 image-only 이거나 첫 anchor 라인에 anchor 됨

조건 1·2 만으로도 충분히 보수적이지만, 조건 3을 추가해 텍스트 + 그림 혼합 단락의 동작 변화를 더 보수적으로 막을지 단계 1 분석에서 확정한다.

---

## 2. 단계 분할 (4 단계)

### Stage 1 — 분석·재현·기준선 캡처

- `samples/exam_science.hwp` 1페이지 SVG 생성 → 현재 ⑤ 클리핑 재현 확인
- 회귀 검증용 baseline SVG·해시 수집 대상 샘플 선정 (최소: `exam_science.hwp` 외 1개 기존 샘플)
- 셀 안 비-TAC TopAndBottom 이미지 패턴이 다른 샘플에 어떤 형태로 존재하는지 grep / dump 확인
- 단계 1 보고서: 적용 범위(조건 1·2 만 / 1·2·3) 결정

산출물: `mydocs/working/task_m100_577_stage1.md`

### Stage 2 — 코드 수정

- `table_layout.rs:1624..1648` 의 `compute_object_position` 호출에 anchor 시점 y 전달
  - 신규 `let anchor_y = if {조건} { para_y_before_compose + lineSeg vpos 보정 } else { para_y };`
  - `cell_area.y` 와 `compute_object_position(..., para_y=anchor_y, ...)` 모두 해당 분기에서 동일한 `anchor_y` 사용
  - `para_y += pic_h;` 는 다음 단락 시작점 산출용. anchor_y 변경 후에도 cell 의 effective row height 가 충분하므로 그대로 두지만, 단계 2 에서 동작 검증 후 필요시 조정
- `cargo build --release` 통과
- 단순 회귀: `cargo test` 와 `cargo clippy --release -- -D warnings` 통과

산출물: `mydocs/working/task_m100_577_stage2.md` + 소스 커밋

### Stage 3 — 시각·자동 검증

- `rhwp export-svg samples/exam_science.hwp -o output/svg/task577_after/` 후 ① ~ ⑤ 모두 셀 클립 안에 들어가는지 좌표 비교
- `LAYOUT_OVERFLOW` 메시지가 줄어드는지 / 새 오버플로 발생 여부 확인
- 다른 샘플(스테이지 1에서 선정) SVG diff 비교 — 의도된 변화만 있는지 확인
- 필요 시 `ir-diff` 로 IR 비변경 확인 (렌더 단계만 수정이라 IR 자체는 동일해야 함)

산출물: `mydocs/working/task_m100_577_stage3.md` (오버플로 표 + diff 결과 요약)

### Stage 4 — 종결

- 최종 결과 보고서 작성: `mydocs/report/task_m100_577_report.md`
- `mydocs/orders/{오늘}.md` 갱신 (해당 타스크 상태)
- 본 계획서 → `mydocs/plans/archives/`로 이동(승인 후 정리 단계에서)

산출물: `mydocs/report/task_m100_577_report.md`

---

## 3. 변경 파일 (예상)

| 경로 | 변경 종류 |
|------|----------|
| `src/renderer/layout/table_layout.rs` | 비-TAC Picture 분기 anchor y 산식 수정 (조건부) |
| `mydocs/working/task_m100_577_stage{1..3}.md` | 단계 보고 |
| `mydocs/report/task_m100_577_report.md` | 최종 보고 |
| `mydocs/orders/{날짜}.md` | 오늘 할일 갱신 |

코드 수정은 한 파일 한 분기로 한정. `picture_footnote.rs` 의 `compute_object_position` 자체는 수정하지 않는다 (다른 호출처 회귀 방지).

---

## 4. 위험·완화

| 위험 | 완화 |
|------|------|
| 비셀(본문) TopAndBottom 이미지 회귀 | 수정은 셀 처리 루프 내부 분기(table_layout.rs)에 한정. 본문 경로(picture_footnote.rs)는 무변경 |
| 셀 내 텍스트+그림 혼합 단락 회귀 | 단계 1 분석 후, 필요 시 image-only 단락에만 한정 |
| HWP3·HWPX 회귀 | 렌더 단계 수정이라 IR 비변경. 단계 3에서 ir-diff 와 다른 샘플 SVG diff 로 검증 |
| 다른 샘플의 잠재 동일 버그가 같이 변경 | 의도된 개선이면 단계 3 보고서에 명시. 새 회귀로 판단되면 본 PR 에서 분리 |

---

## 5. 승인 요청

위 4 단계 계획대로 진행해도 좋을지 승인 요청드립니다. 승인 후 Stage 1(분석·기준선 캡처)부터 시작합니다.
