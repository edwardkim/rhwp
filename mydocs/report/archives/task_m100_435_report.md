# Task #435: exam_kor.hwp 24페이지 → 20페이지 정합 — 최종 결과보고서

> **이슈**: [#435](https://github.com/edwardkim/rhwp/issues/435)
> **관련 이슈**: [#393](https://github.com/edwardkim/rhwp/issues/393) (#435 Stage 2 로 옵션 A 적용 완료)
> **브랜치**: `local/task435` (base: `local/devel` @ `d1a4058`)
> **마일스톤**: v1.0.0 (M100)
> **작업기간**: 2026-04-29
> **결과**: **부분 완료** (24 → 22 페이지, 목표 20 미달)

---

## 요약

`samples/exam_kor.hwp` 의 페이지네이션을 한컴 PDF (20 페이지) 와 정합시키는 작업. **24 → 22 페이지** 로 2 페이지 단축 (목표 20 미달, 잔여 -2 페이지는 별도 메커니즘으로 미해결). `compute_body_wide_top_reserve_for_para` 의 Paper-rel 좌표 변환 버그 정정으로 #393 (옵션 A) 함께 처리. 회귀 없음.

## 결과 메트릭

| 메트릭 | Before | After | Δ |
|---|---|---|---|
| **exam_kor.hwp 페이지 수** | 24 | **22** | **-2** |
| Orphan 페이지 (page 2 49px, page 15 104px) | 2개 | **0** | -2 |
| pi=0.30 (page 1 단 1) split | PartialParagraph | **FullParagraph** | 해소 |
| pi=1.25 (page 14 단 1) split | PartialParagraph | **FullParagraph** | 해소 |
| col 1 reserve (Paper-rel body-wide table) | 306.1px | **94.4px** | -211.7 (정확) |

## 회귀 검증

| 문서 | Before | After | 결과 |
|---|---|---|---|
| exam_kor.hwp | 24 | **22** | 목표 진행 (20 미달) |
| exam_eng.hwp | 8 | 8 | ✓ 유지 |
| k-water-rfp.hwp | 28 | 28 | ✓ 유지 |
| hwpspec.hwp | 177 | 177 | ✓ 유지 |
| synam-001.hwp | 35 | 35 | ✓ 유지 |

`cargo test --release`: **1062 passed, 0 failed, 1 ignored** (전체 lib + integration tests 통과)

`cargo clippy --release -- -D warnings`: pre-existing 에러 (`table_ops.rs:1007`, `object_ops.rs:298` 의 panicking_unwrap) 2건 — 본 task 와 무관하며 devel base 에서도 동일 발생.

## 코드 변경 (Stage 2)

### `src/renderer/typeset.rs:2127-2174` `compute_body_wide_top_reserve_for_para`

**버그**: `VertRelTo::Paper` 일 때 `vertical_offset` (paper-rel) 을 body-rel 변환 없이 reserve 에 직접 누적.

**수정 전**:
```rust
if matches!(common.vert_rel_to, VertRelTo::Paper) {
    let shape_top_abs = ...vertical_offset...;
    let shape_bottom_abs = shape_top_abs + ...height...;
    if shape_bottom_abs <= body_top { continue; }
}
let shape_y_offset = ...vertical_offset...;
if shape_y_offset > body_h / 3.0 { continue; }
let bottom = shape_y_offset + shape_h + outer_bottom;  // ❌ Paper-rel 좌표
```

**수정 후**:
```rust
let shape_h = ...height...;
let raw_v_offset = ...vertical_offset...;

let (body_y, body_bottom) = if matches!(common.vert_rel_to, VertRelTo::Paper) {
    let shape_top_abs = raw_v_offset;
    let shape_bottom_abs = shape_top_abs + shape_h;
    if shape_bottom_abs <= body_top { continue; }
    ((shape_top_abs - body_top).max(0.0), shape_bottom_abs - body_top)
} else {
    (raw_v_offset, raw_v_offset + shape_h)
};

if body_y > body_h / 3.0 { continue; }
let bottom = body_bottom + outer_bottom;  // ✅ body-rel
```

### 변경 효과 검증

exam_kor.hwp 의 body-wide 표 (`pi=0` `wrap=TopAndBottom` `vert=Paper(38mm)`):

| 항목 | 값 | 계산 |
|---|---|---|
| `body_top` | 211.7 px | margin_top 56mm |
| `shape_top_abs` (paper-rel) | 143.6 px | v_offset 38mm |
| `shape_bottom_abs` (paper-rel) | 291.0 px | top + h(11057 HU) = 290.9 |
| `body_y` (수정 후) | 0 | max(0, 143.6 - 211.7) |
| `body_bottom` (수정 후) | 79.2 px | 291.0 - 211.7 |
| `outer_bottom` | 15.1 px | margin.bottom 4mm |
| **reserve (수정 후)** | **94.3 px** | body_bottom + outer_bottom |
| HWP 실제 col 1 시작 | 94.5 px | first_vpos=7085 HU |

오차 0.2px 이내로 정확.

## 단계별 진행

| 단계 | 결과 | 코드 변경 |
|---|---|---|
| Stage 1 | 베이스라인 측정 + 진단 데이터 수집 | - |
| Stage 2 | #393 (1) col 1 reserve 정정 | `typeset.rs:2127-2174` (32 lines) |
| Stage 3 | 일반 페이지 누적 부족 조사 | (없음 — 가설 재검토로 보류) |
| Stage 4 | (조건부 — 진입 안 함) | - |
| Stage 5 | 최종 검증 + 보고서 | - |

## Stage 3 핵심 발견

원래 가설 ("표/도형 후 컬럼 잔여 공간 산정 부족") 은 **메트릭 오해**:

- `diff = used_height - hwp_last_line_bottom` 은 **typeset cur_h vs layout last-line bottom 의 좌표계 차이**
- **추가로 채울 수 있는 공간이 아님**

증거: 페이지 3 col 0 used=957.8 / hwp_used=1208.0 / diff=-250.1
- typeset budget 957 까지, layout 은 vpos 1208 까지 그림 (이미 본문 1211 한계 근접)
- 추가 paragraph 넣으면 layout 상 overflow

### 실제 22→20 페이지 단축 장애물

| 위치 | 메커니즘 |
|---|---|
| 섹션 1 페이지 14 | Square wrap 표 4개 + col 0 over-fill (1225>1211) → col 1 under-use (64px) |
| 섹션 1 페이지 15 | 단정의 2단인데 단 1 누락 (단일 컬럼 출력) — pagination column 처리 별도 버그 |
| 섹션 2 페이지 18 | pi=11 split + pi=13 [단나누기] orphan-like — col 0 cur_h 가 HWP 대비 ~100px over-advance |

## 잔여 작업 (별도 task 권고)

본 task 종료 후 다음 후속 작업이 22→20 페이지 단축에 필요:

1. **신규 이슈 (Square wrap 표 over-fill)**: `paginate_text_lines` (`engine.rs:702-711`) 의 표 직후 trailing line_spacing 제외 로직이 col 0 over-fill 을 통과시킴. 페이지 14 col 0 1225.8 → 본문 1211.3 이내로 정정 필요.

2. **신규 이슈 (단일 컬럼 출력 버그)**: 섹션 1 페이지 15 가 단정의 2단임에도 단일 컬럼으로 출력. [단나누기] 후 새 페이지 생성 시 column_contents 초기화 누락 추정.

3. **신규 이슈 (col 0 cur_h over-advance)**: 섹션 2 페이지 18 col 0 cur_h 가 HWP vpos 보다 ~100px 앞섬. pi=2 (Square wrap 표 9-line partial) 처리에서 cur_h 누적 과대 추정.

## 산출물

| 파일 | 내용 |
|---|---|
| `mydocs/plans/task_m100_435.md` | 수행계획서 |
| `mydocs/plans/task_m100_435_impl.md` | 구현 계획서 (5단계) |
| `mydocs/working/task_m100_435_stage1.md` | Stage 1 베이스라인 측정 |
| `mydocs/working/task_m100_435_stage2.md` | Stage 2 col 1 reserve 정정 (실제 코드 변경) |
| `mydocs/working/task_m100_435_stage3.md` | Stage 3 일반 페이지 누적 부족 조사 |
| `mydocs/report/task_m100_435_report.md` | 본 최종 보고서 |
| `mydocs/orders/20260429.md` | 오늘할일 갱신 |
| `output/debug/task435/exam_kor_baseline.csv` | Stage 1 베이스라인 데이터 (24 페이지) |
| `output/debug/task435/exam_kor_stage2.csv` | Stage 2 결과 데이터 (22 페이지) |
| `output/debug/task435/regression_baseline.txt` | 회귀 대상 5문서 페이지 수 |
| `output/debug/task435/typeset_drift_baseline.txt` | RHWP_TYPESET_DRIFT 출력 (748 lines) |
| `output/debug/task435/typeset_drift_key.txt` | 핵심 paragraph 추출 |

## 커밋

| Hash | 단계 | 변경 |
|---|---|---|
| `e0cdc2e` | Stage 1 | 수행/구현 계획서 + Stage 1 보고서 (590+ lines docs) |
| `0879f11` | Stage 2 | typeset.rs 32 lines + Stage 2 보고서 (165 lines) |
| `bd08d6b` | Stage 3 | Stage 3 조사 보고서 (167 lines, 코드 변경 없음) |

## 결론

#393 의 옵션 A (col 1 reserve 정정) 를 정확히 구현하여 **24→22 페이지 (2 페이지 단축)** 달성. 회귀 0건. 그러나 한컴 PDF (20 페이지) 정합은 미달 — 잔여 2 페이지 단축은 Square wrap, 단일 컬럼 출력 버그, col 0 cur_h over-advance 의 3 가지 별도 메커니즘 필요. 본 task 는 옵션 A (현 상태 종료) 로 마무리하고 잔여 작업은 별도 이슈로 분리 권장.

#393 은 본 task 의 Stage 2 수정으로 옵션 A 가 적용 완료되었으므로 **close 가능** (작업지시자 결정 필요).
