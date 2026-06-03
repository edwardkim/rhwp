# Task #452: 단락 마지막 줄 trailing line_spacing 누락 수정 — 구현 계획서

## 핵심 이해

### 현재 구조

**병행되는 두 y 누적 체계**:

1. **pagination/engine.rs (`current_height`)**: `current_height += para_height` (`para_height` = `height_measurer` 가 전체 줄의 `lh+ls` 합). 페이지 fitting 판정에서만 `effective_trailing` 으로 일시적으로 trailing 제외하고, **누적값에는 항상 trailing 포함**.
2. **layout.rs (`y_offset`)**: 각 단락이 `layout_paragraph` 를 호출하며 반환된 `y` 를 다음 단락 `y_start` 로 사용. 현재 마지막 줄에서 trailing ls 를 제외해 `current_height` 와 1 ls 만큼 어긋남.

→ pagination 은 trailing 포함 좌표로 페이지 분배하고, 실제 렌더링은 trailing 미포함 좌표로 그려 매 단락 1 ls 씩 위로 당겨짐. **본 수정의 본질: 두 체계의 정합 회복**.

### Task #332 의 의도

`troubleshootings/typeset_layout_drift_analysis.md` + #332 본문에 따르면, 당시 의도는 typeset 의 `height_for_fit` 모델과 layout 그리기를 **정합** 시키는 것이었음. 그러나 실제 도입된 코드는 **layout 측만 trailing 을 제외**해 pagination 누적과 어긋난 정합 — 절반의 정합. 옵션 A 는 layout 도 trailing 포함으로 통일하는 **반대 방향 정합**.

### 보존해야 하는 분기

`is_cell_last_line` (셀 내 마지막 문단의 마지막 줄): 표 셀 높이는 별도 모델로 trailing 을 제외한 채 측정·렌더링되므로 본 분기는 변경 금지.

## 구현 단계 (4 단계)

### Stage 1: Baseline 측정 + 회귀 베이스라인 캡처

**목적**: 수정 전후 정량 비교 기준 확보.

작업:
1. 현재 SVG 출력으로 다음 측정값 기록 (`mydocs/working/task_m100_452_stage1.md`):
   - `samples/exam_kor.hwp` 1페이지 좌측 단: pi=1.line9 ↔ pi=2.line0 baseline diff (현재 15.34 px)
   - 같은 페이지 pi=2.line4 ↔ pi=3.line0 baseline diff (현재 24.51 px, 회귀 감시용)
   - `samples/aift.hwp`, `samples/biz_plan.hwp`, `samples/2022년 국립국어원 업무계획.hwp`, `samples/exam_eng.hwp`, `samples/exam_math_8.hwp`, `samples/k-water-rfp.hwp`, `samples/kps-ai.hwp`, `samples/synam-001.hwp` 의 페이지 수
2. 광범위 byte 비교 baseline: 이번에 SVG export 가능한 대표 샘플들 (위 8 종 + exam_kor) 의 SVG 를 별도 폴더에 저장.
3. Task #332 가 해결한 회귀 케이스 식별:
   - `samples/21_언어_기출_편집가능본.hwp` page 1 col 1 의 pi=26 + 보기 ①②③ fit 여부 (#332 본문 기준)
   - 회귀 발생 시 어떤 형태로 나타날지 예측 (마지막 단락 trailing ls 가 페이지 하단을 넘어 다음 페이지로 밀릴 가능성)
4. 산출물: `mydocs/working/task_m100_452_stage1.md` + `/tmp/task_452_baseline/` SVG 묶음.

승인 게이트: baseline 측정값 + 검증 항목 확인 후 다음 단계 진행.

### Stage 2: 코드 수정 + 단위 검증

**목적**: 핵심 분기 변경 + 즉시 단위 검증.

작업:
1. `src/renderer/layout/paragraph_layout.rs:2511-2520` 수정:

```rust
// Before
let is_cell_last_line = is_last_cell_para && line_idx + 1 >= end;
let is_para_last_line = cell_ctx.is_none()
    && line_idx + 1 == end
    && end == composed.lines.len();
if (is_cell_last_line && cell_ctx.is_some()) || is_para_last_line {
    y += line_height;
} else {
    let line_spacing_px = hwpunit_to_px(comp_line.line_spacing, self.dpi);
    y += line_height + line_spacing_px;
}

// After
let is_cell_last_line = is_last_cell_para && line_idx + 1 >= end;
if is_cell_last_line && cell_ctx.is_some() {
    // 셀 내 마지막 문단의 마지막 줄: trailing line_spacing 제외 (셀 높이 모델 정합)
    y += line_height;
} else {
    // 본문 단락: 마지막 줄 포함 모든 줄에서 trailing line_spacing 가산.
    // pagination/engine.rs 의 current_height 누적(para_height = sum(lh+ls)) 과 정합.
    // (#452: 이전 #332 의 layout-only trailing 제외 → pagination 과 1 ls drift 발생 → 회복)
    let line_spacing_px = hwpunit_to_px(comp_line.line_spacing, self.dpi);
    y += line_height + line_spacing_px;
}
```

   주석은 Task #332 의 원래 의도 + 본 수정의 정합 방향을 함께 기록.
2. `cargo build --release --bin rhwp` 통과.
3. `cargo test --lib` 전체 통과 (기준: 직전 빌드 1117 passed 유지). 실패 테스트는 의도된 baseline 변경인지, 회귀인지 분류 → 의도된 변경이면 별도 commit 으로 baseline 갱신, 회귀이면 Stage 2 종료 전에 원인 분석.
4. `samples/exam_kor.hwp` 1페이지 SVG 재생성 → pi=1.line9 ↔ pi=2.line0 step = 24.51 px 검증, pi=2.line4 ↔ pi=3.line0 step = 24.51 px 유지 검증.
5. 산출물: `mydocs/working/task_m100_452_stage2.md`.

승인 게이트: cargo test 통과 + exam_kor 1페이지 정량 검증 통과 후 다음 단계 진행.

### Stage 3: 광범위 회귀 검증

**목적**: 페이지 수 변동·시각 회귀·#332 회귀 종합 점검.

작업:
1. Stage 1 의 8 종 샘플 SVG 재생성 → 페이지 수 비교 (변동 0 이 목표):
   - 변동 발생 시: 어느 페이지에서 어떤 단락이 이동했는지 식별. trailing ls 1 단위 이내 마진이면 cosmetic, 그 이상이면 원인 분석.
2. 광범위 byte 비교: 의도된 변경 (간격 미세 변동) 외 의외 영역 변경 여부 점검.
3. `samples/21_언어_기출_편집가능본.hwp` page 1 col 1 pi=26 + 보기 ①②③ 위치 재확인 → #332 회귀 테스트.
4. golden SVG snapshot 영향: `tests/golden_svg/` 의 어떤 파일이 변경되는지 확인 → 변경이 의도된 정합인지 시각 검토 후 baseline 갱신 PR 형태로 별도 commit.
5. 광범위 회귀 의심 시: 작업지시자 시각 판정 요청.
6. 산출물: `mydocs/working/task_m100_452_stage3.md` (페이지 수 변동표 + 광범위 비교 결과).

승인 게이트: 의외 회귀 0 + 작업지시자 시각 판정 (필요 시) 통과 후 다음 단계.

### Stage 4: 최종보고서 + 정리

**목적**: 작업 종결 + 메타 자료 갱신.

작업:
1. `mydocs/report/task_m100_452_report.md` 작성:
   - Before/After 측정값 (Stage 1 ↔ Stage 2)
   - 회귀 검증 요약 (Stage 3)
   - 잔여 위험 / 후속 작업 제안 (있다면)
2. `mydocs/orders/20260429.md` 의 #452 행 상태 갱신 (`수행계획 작성, 승인 대기` → `완료`).
3. `mydocs/troubleshootings/typeset_layout_drift_analysis.md` 에 본 수정의 영향 후속 메모 추가 (Task #332 후속이므로 동일 문서 참조 권장).
4. `local/task452` 의 모든 commit 확인 + 최종 보고서/orders 갱신을 별도 commit 으로 추가.
5. 산출물: `mydocs/report/task_m100_452_report.md` + orders 갱신 + (선택) troubleshooting 문서 갱신.

승인 게이트: 최종보고서 검토 → 승인 후 `local/task452` → `local/devel` merge.

## 위험·완화

| 위험 | 가능성 | 완화 |
|------|--------|------|
| pagination current_height 와 layout y_offset 정합 회복으로 페이지 끝 단락 trailing ls 가 col_bottom 을 살짝 넘는 cosmetic 변화 | 중 | trailing ls 는 빈 공간이므로 시각적으로 무영향. 단, golden SVG snapshot 변경 가능 → Stage 3 에서 baseline 갱신 |
| Task #332 가 해결한 21_언어 page 1 col 1 의 pi=26 fit 회귀 | 낮음 | pagination engine 의 fit 판정은 현재도 `effective_trailing` 사용 → 본 수정으로 pagination 결정 자체는 변하지 않음. layout 만 정합. Stage 3 에서 직접 검증 |
| `is_cell_last_line` 분기를 보존하지만 셀과 본문 경계의 또 다른 케이스 누락 | 낮음 | `is_para_last_line` 의 `cell_ctx.is_none()` 가드는 이미 셀과 본문을 분리했음. 셀 분기를 그대로 보존하므로 셀 동작 변화 없음 |
| 광범위 회귀 byte 차이가 의외로 클 가능성 | 중 | Stage 3 광범위 비교 후 작업지시자 시각 판정 요청 |

## 의존성 / 영향

- 단일 파일 수정 (`src/renderer/layout/paragraph_layout.rs`).
- 단일 분기 ~10 줄 변경.
- pagination/engine.rs, height_measurer.rs, web_canvas.rs, svg.rs 변경 없음.
- 머리말/꼬리말, 표 셀 내부, 각주, 머리말 표 anchor 영향 없음.

## 일정 추정

- Stage 1: 0.5h
- Stage 2: 0.5h
- Stage 3: 1h
- Stage 4: 0.5h
- 합계: 약 2.5h (회귀 발생 시 추가).
