---
PR: #723
제목: Task #722 — hwp3-sample5.hwp wrap=Square 그림 paragraph 시각 정합
컨트리뷰터: @jangster77 (Taesup Jang) — 16번째 사이클 (HWP 3.0 파서 영역 핵심)
처리: 옵션 A — 1 commit cherry-pick + no-ff merge
처리일: 2026-05-10
머지 commit: 6ced74b0
후속: PR #732 (Task #724) 영역 영역 exam_science 회귀 정정 통합 권장
---

# PR #723 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (1 commit cherry-pick + no-ff merge `6ced74b0`)

| 항목 | 값 |
|------|-----|
| 머지 commit | `6ced74b0` (--no-ff merge) |
| closes | #722 |
| 시각 판정 | 페이지 8/27/48 PDF 권위본 정합 (작업지시자 판정) |
| 회귀 발견 | exam_science p1/p2 5번/8번/12번 문항 지문 왼쪽 경계 클립핑 (작업지시자 시각 판정) |
| 후속 처리 결정 | PR #732 (Task #724) 영역 영역 exam_science 회귀 정정 통합 — **rollback 미진행** |
| 자기 검증 | lib 1173 + 통합 ALL GREEN + clippy 0 + sweep 168/170 same |

## 2. 정정 본질 (3 영역)

### 2.1 `src/renderer/pagination.rs` (+6 LOC)
`WrapAnchorRef` 영역 의 `anchor_image_margin_right: i32` 필드 추가.

### 2.2 `src/renderer/typeset.rs` (+56 LOC)
- anchor 다음 paragraph 등록 영역 의 image margin_right 추출 (Picture/Shape::Picture, treat_as_char=false, wrap=Square)
- anchor host paragraph 자체 self-register (case 가드):
  - LINE_SEG ≥ 2 → wrap zone (multi-line)
  - LINE_SEG 1 + caption_room ≤ line_height → wrap zone (강제)
  - LINE_SEG 1 + caption_room > line_height → caption-style (자기 미등록)

### 2.3 `src/renderer/layout/paragraph_layout.rs` (+10/-3)
wrap_anchor 영역 의 cs/sw 영역 inter-image-text gap (3mm = 852 HU) 보정 — `cs += mr / sw -= mr`.

## 3. 본 환경 cherry-pick + 검증

### 3.1 cherry-pick (1 commit)
```
58ef1362 Task #722: hwp3-sample5.hwp wrap=Square 그림 paragraph 시각 정합 (closes #722)
```
충돌 0건.

### 3.2 결정적 검증

| 검증 | 결과 |
|------|------|
| `cargo build --release` | ✅ 통과 |
| `cargo test --release` | ✅ lib **1173** + 통합 ALL GREEN, failed 0 |
| `cargo clippy --release` | ✅ 신규 경고 0 |
| 광범위 sweep | ⚠️ 7 fixture / 170 페이지 / **168 same / 2 diff** (exam_science_001/002.svg) |

광범위 sweep 영역 의 byte diff 발견 영역 — 작업지시자 시각 판정 게이트 영역 으로 회부.

### 3.3 시각 판정 (작업지시자)

**A. PR 본질 (페이지 8/27/48 PDF 정합)** — ✅ 통과
- 페이지 8 paragraph 175 (디렉토리 트리 설명) — image 침범 부재 + 3mm gap
- 페이지 27 paragraph 779 (Figure 4-4 caption) — caption-style 자유 영역 보존
- 페이지 48 paragraph 1394 (접근 제어) — image 침범 부재 + 3mm gap

**B. exam_science 회귀 점검** — ⚠️ 회귀 판정
- exam_science p1/p2/p8 영역 의 5번/8번/12번 문항 지문 왼쪽 경계 글자 클립핑
- 회귀/기존 문제 판단 어려움 영역 → 회귀로 판정
- exam_science.hwp 영역 wrap=Square 부재 (wrap=InFrontOfText/BehindText/TopAndBottom 만 존재) → 의도 영역 외 영향 의심

### 3.4 WASM 빌드
- `pkg/rhwp_bg.wasm` 4.60 MB
- `pkg/rhwp.js` 234 KB
- 빌드 시간: 1m 53s

## 4. 처리 결정 — PR #732 통합 후속

### 4.1 결정 영역
- PR #723 머지 유지 (rollback 미진행)
- exam_science 회귀 정정 영역 영역 PR #732 (Task #724, 동일 컨트리뷰터 @jangster77) 영역 영역 후속 처리 영역 으로 통합

### 4.2 결정 영역 의 근거
- PR #732 영역 영역 동일 컨트리뷰터 영역 의 hwp3-sample5-hwp5.hwp paragraph 441 wrap zone + HWP3 파서 IR 정합 영역 — 동일 본질 영역 (wrap zone 영역) 의 후속 정정
- exam_science 회귀 영역 영역 PR #723 영역 의 typeset.rs anchor host self-register 영역 영역 의 외부 영향 의심 영역 → PR #732 영역 영역 동일 영역 추가 가드 가능
- PR #723 영역 본질 정정 (페이지 8/27/48) 영역 ✅ 통과 → 본질 보존 영역 합리

### 4.3 후속 처리 항목 (PR #732 처리 시)
- exam_science p1/p2/p8 영역 의 5번/8번/12번 문항 지문 왼쪽 경계 클립핑 회귀 정정
- typeset.rs anchor host self-register 영역 의 case 가드 영역 추가 정합 (exam_science 영역 영향 차단)
- 광범위 sweep 168 same 영역 → 170 same 영역 회복

## 5. 영향 범위

### 5.1 변경 영역
- HWP3 변환본 영역 의 wrap=Square 그림 paragraph 영역 의 시각 정합 (페이지 8/27/48)
- inter-image-text gap (3mm = 852 HU) 영역 적용

### 5.2 무변경 영역
- treat_as_char=true 영역 무관
- 비-Square wrap (TopAndBottom / BehindText / InFrontOfText) 영역 무관
- LINE_SEG 1 + caption-style 영역 (자유 영역 보존)

### 5.3 위험 영역 (검증된 회귀)
- exam_science.hwp 영역 영역 wrap=Square 부재 영역 영역 영역 paragraph 영역 의 text 좌표 시프트 영역 → PR #732 영역 영역 후속 정정 영역 으로 통합

## 6. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @jangster77 16번째 사이클 (HWP 3.0 파서 영역 핵심 컨트리뷰터) |
| `feedback_hancom_compat_specific_over_general` | LINE_SEG + caption_room 영역 case 가드 영역 좁힘 — caption-style 보존 + Stage 8~9 회귀 영역 rollback 정합 |
| `feedback_visual_judgment_authority` | 작업지시자 시각 판정 게이트 영역 ⓐ 본질 통과 ⓑ 회귀 발견 영역 영역 후속 PR 통합 |
| `feedback_image_renderer_paths_separate` | typeset.rs / paragraph_layout.rs / pagination.rs 세 영역 동기 정정 |
| `feedback_pr_supersede_chain` | **신규 패턴 (c)**: 머지 + 회귀 정정 영역 후속 PR 통합 — PR #723 (본질) → PR #732 (회귀 정정 + 본질) |
| `feedback_process_must_follow` | TDD Stage 1~9 절차 정합 + 후속 분리 (Issue #732 → Task #724) |

## 7. 잔존 후속

- **PR #732 (Task #724) 처리 시 exam_science 회귀 정정 통합** (작업지시자 결정 영역)
- Issue #722 영역 close (closes #722 자동 정합) — 본질 영역 통과 영역 기반
- Issue #732 (Task #724) 영역 OPEN 유지 → PR #732 처리 영역 영역 close

---

작성: 2026-05-10
