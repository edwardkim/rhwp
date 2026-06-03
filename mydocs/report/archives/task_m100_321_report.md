# Task #321 최종 보고서 — 페이지네이션 LINE_SEG vpos-reset 강제 분할

상위 Epic: (해당 없음 — 독립 이슈)
관련 선행 작업: Task #310 (vpos 분석), Task #311 (Paginator에서 동일 접근 시도·부정), Task #313 (TypesetEngine 전환)
브랜치: `task321` ← `task318`
이슈: [#321](https://github.com/edwardkim/rhwp/issues/321)

## 증상

`samples/21_언어_기출_편집가능본.hwp` 1페이지 우측단 하단에서 문단 pi=29와 pi=30이 동일 y 좌표(y=1421.5)에 클램프되어 **시각적 텍스트 겹침**. `LAYOUT_OVERFLOW` 경고 3건(pi=28/29/30).

## 결과 요약

**시각적 겹침 해소.** pi=30이 HWP 원본 의도대로 페이지 2 col 0 상단에 배치되며 pi=29와의 중첩 제거. `cargo test --release`: 992 passed, 0 failed.

| 항목 | Before | After |
|------|--------|-------|
| 21_언어 페이지 수 | 15 | 15 |
| 21_언어 LAYOUT_OVERFLOW 건수 | 5 | 4 |
| pi=29/pi=30 시각 겹침 | ❌ 있음 | ✅ 해소 |
| exam_math/kor/eng 페이지 수 | 20/24/9 | 20/24/9 |

잔존 pi=28/pi=29 경고(9.5px)는 포맷터의 trailing line_spacing 포함 계산과 layout의 vpos 진행 사이 드리프트에 의한 것이며, 실제 body text는 col_bottom 이내에 렌더되어 **시각 영향 없음**. 별도 후속 이슈로 분리 가능.

## 구현

### Stage 1 — 드리프트 origin 정량화 (커밋 `348c617`)

`src/renderer/typeset.rs::typeset_paragraph` 진입부에 `RHWP_TYPESET_DRIFT=1` env 진단 훅 추가. 결과:

- 포맷터 `fmt.total_height`가 모든 문단에 trailing line_spacing 포함 (~9.5px/문단 과다).
- pi=30의 `first_vpos=0`이 HWP 원본의 "pi=30은 새 페이지/단 시작" 신호임을 확인.

### Stage 2 — 문단간 vpos-reset 강제 분할 (커밋 `3cea672`)

`src/renderer/typeset.rs::typeset_section` 문단 순회 루프에 다음 검사 추가:

```rust
if para_idx > 0 && !st.current_items.is_empty() {
    let curr_first_vpos = para.line_segs.first().map(|s| s.vertical_pos);
    let prev_last_vpos = paragraphs[para_idx - 1].line_segs.last().map(|s| s.vertical_pos);
    if let (Some(cv), Some(pv)) = (curr_first_vpos, prev_last_vpos) {
        if cv == 0 && pv > 5000 {
            st.advance_column_or_new_page();
        }
    }
}
```

핵심 가드:
- `cv == 0`: HWP가 vpos=0으로 리셋한 첫 seg만 대상 (일반 빈 문단의 우연 vpos 제외)
- `pv > 5000 HU`: 직전 문단이 실제 내용이 있어야 함 (5000 HU ≈ 1.76mm)
- `!st.current_items.is_empty()`: 단 최상단에서는 불필요한 분할 방지

## 21_언어 상세 효과

### Before
- 페이지 1 단 1: 22 items (pi=9 partial, pi=10..pi=30)
- pi=30 LINE_SEG vpos=0 무시, col 1 하단에 배치 → pi=29와 동일 y로 클램프

### After
- 페이지 1 단 1: 21 items (pi=9 partial, pi=10..pi=29)
- 페이지 2 단 0: pi=30 첫 항목 배치 (HWP 원본 의도 복원, hwp_used 오차 -6.8px)

### SVG 좌표 검증
- Before: pi=29 ③ y=1421.5, pi=30 ④ y=1421.5 (동일 위치 겹침)
- After: pi=28 ② y=1420.28, pi=29 ③ y=1433.97 (13.7px 분리), pi=30 ④ → 페이지 2

## Task #311과의 대비

Task #311은 Paginator 경로에서 유사한 vpos-reset 강제 분할을 시도하였으나 21_언어가 19→20쪽으로 회귀. 본 Task #321은 TypesetEngine 경로에서 적용하였고 페이지 수 불변 유지.

차이 원인:
1. TypesetEngine이 column 가용 공간을 더 정확히 사용 (#313에서 검증됨).
2. inter-paragraph reset만 대상으로 함 (intra-paragraph reset은 기존 `detect_column_breaks_in_paragraph`가 처리).
3. 가드 조건 `pv > 5000 HU`로 미미한 vpos만 있는 경우 제외.

## 잔존 이슈 (별도 후속 후보)

`pi=28/pi=29`의 9.5px LAYOUT_OVERFLOW 경고는 다음 불일치에 기인:
- 포맷터: `total_height = spacing_before + sum(line_height + line_spacing) + spacing_after` → trailing line_spacing 포함
- vpos 실측: `last.vpos + last.lh - first.vpos` → trailing line_spacing 미포함

두 지표 차이가 문단당 ~9.5px. 누적되어 column 경계 근처에서 경고 발생. 실제 렌더는 layout의 `is_column_top` 등 보정 로직으로 col_bottom 이내에 clamp되므로 **시각 영향 없음**. 포맷터와 vpos 정합이 필요하다면 별도 이슈로 다룬다.

## 산출물

- 코드: `src/renderer/typeset.rs` (+ 진단 훅 + vpos-reset 검출 로직)
- 문서:
  - `mydocs/plans/task_m100_321.md` (수행계획서)
  - `mydocs/plans/task_m100_321_impl.md` (구현 계획서)
  - `mydocs/working/task_m100_321_stage1.md` (Stage 1 정량화)
  - `mydocs/working/task_m100_321_stage2.md` (Stage 2 효과 분석)
  - 본 최종 보고서

## 보존된 부속물

| 부속물 | 보존 이유 |
|--------|-----------|
| `RHWP_TYPESET_DRIFT` env 진단 훅 | 추후 fmt vs vpos 정합 디버깅용 |

## 회귀 검증

- `cargo test --release`: 992 passed, 0 failed.
- 4개 핵심 샘플: 페이지 수 불변.
- `samples/` 146개 파일 export-svg 대량 실행 정상 완료 (배치 스크립트).

## 본 Task 종료 절차

1. Stage 4 커밋 (본 보고서 포함).
2. 작업지시자 승인 후:
   - `gh issue close 321`
   - task321 → task318 (또는 local/devel) merge 결정.
3. 병합 후 잔존 9.5px 드리프트는 별도 후속 이슈로 검토 (선택).

## 학습

1. **Task #311의 부정 결과를 교훈으로 보존**한 덕분에 본 Task는 "같은 접근이 엔진만 바뀌면 성공할 수 있다"는 가설로 좁혀 즉시 검증 가능했다.
2. **경고 ≠ 시각 버그**: LAYOUT_OVERFLOW 경고가 남더라도 실제 렌더 결과가 허용 가능한지는 SVG를 직접 검증해야 한다.
3. **좁은 가드**가 광범위 회귀를 막았다: `cv == 0 && pv > 5000` 조합으로 오탐 최소화.
