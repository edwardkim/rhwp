# PR: Task #321 — 페이지네이션 LINE_SEG vpos-reset 강제 분할 (1페이지 우측하단 겹침 수정)

## 제목

```
Task #321: TypesetEngine 문단간 vpos-reset 강제 분할 (21_언어 1페이지 우측하단 겹침 수정)
```

## 본문

### 배경

작업지시자 제보: `samples/21_언어_기출_편집가능본.hwp` 1페이지 우측하단이 겹쳐 보임. 덤프 결과 pi=29와 pi=30이 동일 y(=1421.5)에 클램프되어 중첩 렌더 중임을 확인. #313(TypesetEngine main 전환) 후에도 잔존하던 별개 이슈.

선행:
- #310 (LINE_SEG vpos 분석 + 진단 도구) — vpos-reset 패턴 식별
- #311 (Paginator 경로에서 vpos-reset 강제 분할 시도) — 21_언어 19→20쪽 악화로 부정
- #313 (TypesetEngine main 전환) — 21_언어 19→15쪽 (PDF 일치) 달성, 잔존 9.5px 오버플로우는 미해결

### 원인

HWP LINE_SEG 데이터에서 pi=30의 `first_vpos == 0`. HWP 5.0 LINE_SEG는 페이지 내 흐름의 y 좌표를 기록하므로, vpos가 0으로 리셋되는 것은 HWP 원본이 해당 문단을 새 페이지/단의 첫 줄로 배치할 의도임을 의미한다.

그러나 TypesetEngine의 `typeset_section` 문단 순회는 `column_type` (`break_byte`)과 `ParaShape.page_break_before` 만 검사하고 LINE_SEG vpos-reset 신호를 사용하지 않는다. 그 결과 pi=30이 col 1에 계속 쌓이고, 누적 높이는 `available` 이내(1223.1 ≤ 1226.4)이지만 layout 단계의 vpos 보정 + 순차 y 진행 사이 드리프트로 pi=29와 pi=30이 col 하단에 동일 y로 클램프되어 텍스트가 겹친다.

### 변경

#### 1. `src/renderer/typeset.rs::typeset_paragraph` — Stage 1 진단 훅

`RHWP_TYPESET_DRIFT=1` env 활성화 시 문단별 `(current_height, available, fmt.total_height, vpos_h, diff)` 출력. 추후 포맷터 vs vpos 정합 디버깅용으로 보존.

#### 2. `src/renderer/typeset.rs::typeset_section` — 문단간 vpos-reset 강제 분할

```rust
// 다단 나누기 / 단 나누기 / 쪽 나누기 검사 직후
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

가드 조건:
- `cv == 0` — HWP가 vpos를 0으로 리셋한 첫 seg만 대상 (일반 문단의 우연 vpos 제외)
- `pv > 5000 HU (≈ 1.76mm)` — 직전 문단이 실제 내용이 있을 것
- `!st.current_items.is_empty()` — 단 최상단 분할 방지

### 검증

#### 21_언어 시각 효과

| 항목 | Before | After |
|------|--------|-------|
| pi=28 ② y | 1407.8 | 1420.28 |
| pi=29 ③ y | 1421.5 | 1433.97 |
| pi=30 ④ y | **1421.5 (pi=29와 동일)** | 페이지 2 col 0 상단 |
| 시각 텍스트 겹침 | ❌ 있음 | ✅ 해소 |
| LAYOUT_OVERFLOW 건수 | 5 | 4 |

페이지 2 col 0의 `hwp_used≈848.1px, diff=-6.8px` — HWP 원본과 7px 이내 일치.

#### 4 샘플 무회귀

| 샘플 | 기대 | 측정 |
|------|------|------|
| 21_언어_기출_편집가능본 | 15 | 15 ✓ |
| exam_math | 20 | 20 ✓ |
| exam_kor | 24 | 24 ✓ |
| exam_eng | 9 | 9 ✓ |

#### 전체 테스트

```
cargo test --release
992 passed; 0 failed; 1 ignored
```

`samples/` 146개 export-svg 대량 실행 정상 완료. LAYOUT_OVERFLOW 분포는 21_언어 5→4로 1건 감소, 다른 샘플 변동 없음.

### Task #311과의 차이

#311은 Paginator 경로에서 vpos-reset 강제 분할 시도, 21_언어 19→20쪽 회귀로 가설 부정. 본 task는 다음 차이로 회귀 없이 적용 가능:

1. **TypesetEngine 경로** — column 가용 공간을 더 정확히 사용 (#313에서 검증).
2. **inter-paragraph reset만 대상** — intra-paragraph reset은 기존 `detect_column_breaks_in_paragraph`가 처리.
3. **타이트한 가드** — `cv == 0 && pv > 5000` 조합으로 오탐 최소화.

### 영향 범위

- HWP LINE_SEG에 `vpos == 0` 리셋이 있고 직전 문단이 실제 내용을 가진 경우만 분기 진입.
- HWPX는 vpos 정보가 동일 형식으로 제공되지 않을 수 있으나 `line_segs.first()` 가 None이면 무동작.
- 합성 Paragraph (테스트용)는 보통 `line_segs == []` 이므로 무영향.

### 후속 사안 (선택)

잔존 pi=28/pi=29의 9.5px LAYOUT_OVERFLOW 경고는 다음 불일치에 기인:

- 포맷터 `total_height = sb + Σ(lh + ls) + sa` — trailing line_spacing 포함
- vpos 실측 `last.vpos + last.lh - first.vpos` — trailing line_spacing 미포함

문단당 ~9.5px 차이가 누적되어 col 경계 근처에서 경고 발생. 실제 body text는 layout의 clamp 로직으로 col_bottom 이내에 렌더되어 **시각 영향 없음**. 별도 이슈로 분리 가능.

### 단계별 진행

| 단계 | 내용 | 보고서 |
|------|------|--------|
| 1 | Stage 1 — 드리프트 정량화 (`RHWP_TYPESET_DRIFT` 진단 훅) | `mydocs/working/task_m100_321_stage1.md` |
| 2 | Stage 2 — 문단간 vpos-reset 강제 분할 + 시각 검증 | `mydocs/working/task_m100_321_stage2.md` |
| 3 | (Stage 2에 통합) | — |
| 4 | 회귀 검증 + 최종 보고서 | `mydocs/report/task_m100_321_report.md` |

수행계획서: `mydocs/plans/task_m100_321.md`
구현 계획서: `mydocs/plans/task_m100_321_impl.md`
최종 보고서: `mydocs/report/task_m100_321_report.md`

### Test plan

- [x] `cargo test --release` 992 passed, 0 failed
- [x] 21_언어 1페이지 pi=29/pi=30 시각 겹침 해소 (SVG 좌표 검증)
- [x] 21_언어 LAYOUT_OVERFLOW 5→4 (pi=30 분 1건 해소)
- [x] 4샘플 페이지 수 무변화 (15/20/24/9)
- [x] `samples/` 146개 대량 export-svg 정상 완료
- [x] HWPX 어댑터 회귀 0

closes #321
