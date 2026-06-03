# Stage 2 완료보고서 — #1049 VPOS_CORR lazy_base 수정

- 타스크: #1049 (M100), 브랜치 `local/task1049`
- 단계: Stage 2 — VPOS_CORR lazy_base trailing-ls bridge 정합 (최소 침습)
- 작성일: 2026-05-21
- 대상: 비공개 185p HWPX sec0 pi=781 본문 4.6px overflow

## 1. 수정 (1파일·1지점)

`src/renderer/height_cursor.rs::vpos_adjust` lazy_base 산출의 Task #1022 v2
**trailing-ls bridge** 조건 강화. (줄높이 모델·렌더러 다른 경로 무변경.)

```rust
// 직전이 실텍스트 본문 문단이고 vpos 연속(curr_first_vpos == prev_vpos_end)이면
// 그 trailing 줄간격은 이미 연속 vpos 흐름·sequential y 에 포함 → bridge 끔.
let prev_has_text = prev_para.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}');
let vpos_continuous = matches!(curr_first_vpos, Some(v) if v <= prev_vpos_end);
let trailing_ls_hu = if vpos_continuous && prev_has_text { 0 } else { /* 종전 */ };
```

### 근거 (Stage 1 격리 + 회귀 케이스 데이터)
- **#1049 pi=760**: prev=pi=759(제목, 실텍스트), curr_first_vpos(5969537) **==** prev_vpos_end
  (5969537) → 연속. 렌더러가 pi=759 trailing_ls(960HU)를 이미 advance 에 포함했는데
  bridge 가 base 에서 또 빼 lazy_base 가 960HU 과소(5959663) → pi=760 +12.8px 전진 →
  pi=781 4.6px 초과. **bridge off → lazy_base 5960623, +12.8 점프 소멸**.
- **footnote-01 p1 (보존)**: curr_first_vpos(24869) **>** prev_vpos_end(23869), 1000HU gap →
  bridge 유지(#1022 v2 top-box 후 본문 정합).
- **복학원서 page1 (보존)**: 연속이나 prev=빈 문단(pi=1, text_len=0) → 빈줄 높이 억제로
  trailing_ls 가 sequential y 에 미반영 가능 → bridge 유지. `prev_has_text` 게이트로 구분.

## 2. 검증 결과

| 지표 | 수정 전 | 수정 후 |
|------|--------|--------|
| pi=781 본문 overflow | 4.6px | **0 (해소)** |
| pi=760 시작 vpos_adjust 점프 | +12.80px | **+0.0 (no jump)** |
| lazy_base (pi=760) | 5959663 | 5960623 |
| 대상 overflow 총건 | 3 | **2** (pi=323/567 page-larger만 잔존) |
| `cargo test --release` | 1516 / 0 | **1516 passed / 0 failed** |
| footnote-01 (issue_598) | pass | pass (무회귀) |
| 복학원서 (issue_677 골든) | pass | pass (무회귀) |

- 대상 185p → **184p**: page 111 의 +12.8 드리프트 제거로 후속 콘텐츠가 당겨져 1쪽 축소.
  한컴 2022 PDF(권위) = **179p** → 184 가 185 보다 1쪽 더 근접(회귀 아님, 개선 방향).
- aift.hwp 74p 불변.
- 잔여 드리프트 +8.9px(pi=758 인라인 TAC 표 과대렌더분)는 본문 내, 시각 무영향·overflow 무관.

## 3. 무회귀 핵심 (vpos 연속 + prev 실텍스트 게이트)
VPOS_CORR 은 #412/#643/#874/#991/#1022/#1027 누적 핵심 경로. 본 수정은 bridge 적용 범위를
**좁히기만** 하며(연속+실텍스트 조건에서만 off), gap/빈문단/미상 케이스는 종전 동작 유지.
전체 골든 SVG 전수(svg_snapshot 포함) 1516건 무회귀로 범위 확인.

## 4. 변경 파일
- `src/renderer/height_cursor.rs` (lazy_base bridge 조건, +22/-5, fmt-clean)
- 무관한 rustfmt diff 없음(layout.rs/typeset.rs 미변경).

## 5. 다음 (Stage 3)
- 한글 2022 PDF(`pdf/2. ...-2022.pdf`) 와 page 112(보안 서약서 폼) 시각 정합 대조.
- VPOS 회귀 케이스 골든 재확인(footnote-01·복학원서·exam_kor·#874/#991) — 최종 결과보고서.
