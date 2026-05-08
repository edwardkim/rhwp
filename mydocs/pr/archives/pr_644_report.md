---
PR: #644
제목: Task #643: 페이지 분할 드리프트 정정 (5축 정합) — closes #643
컨트리뷰터: @planet6897 (12번째 사이클)
처리: MERGE (옵션 B — 5 commits 단계별 보존)
처리일: 2026-05-08
---

# PR #644 최종 보고서

## 1. 결정

**옵션 B (5 commits 단계별 보존 merge)** + WASM 빌드 + 작업지시자 시각 판정 통과.

merge commit: `42bb7946`

## 2. 본 환경 검증 결과

### 2.1 cherry-pick simulation
- `local/pr644-sim` 브랜치, 5 commits cherry-pick
- **충돌 0건**

### 2.2 결정적 검증 (1221 테스트)
- `cargo test --release` → ALL PASS, failed 0건
- `tests/issue_643.rs` (신규) ✅ — pi=80 page 6 line 1 정착
- `tests/issue_554.rs` 12/12 ✅ (광범위 sweep)
- `tests/svg_snapshot.rs` 7/7 ✅ (issue_147_aift_page3 골든 갱신 적용)
- `tests/issue_598_footnote_marker_nav.rs` 4/4 ✅

### 2.3 페이지 수 정합 회복
| 환경 | `2022년 국립국어원 업무계획.hwp` 페이지 수 |
|------|-----|
| devel (PR 머지 전) | 40 |
| local/devel (PR #644 머지 후) | **35** |

→ HWP/PDF 원본 정합 회복 (5 페이지 감소).

### 2.4 시각 검증
- WASM 빌드 완료 (`pkg/rhwp_bg.wasm` 14:22 갱신, docker compose run --rm wasm)
- 작업지시자 시각 판정: 페이지 분할 정정 본질 통과
- 검증 시 발견된 별건 결함 → 별도 이슈 #716 등록

## 3. 별도 이슈 등록 (#716)

`samples/20250130-hongbo.hwp` page 1 마지막 줄 LAYOUT_OVERFLOW_DRAW 발견.

### 본질 분리 근거
| 항목 | devel | PR #644 머지 후 |
|------|-------|----------------|
| page 1 SVG md5 | `ab0de6c2e9ecc015402e7acfa091fa02` | `ab0de6c2e9ecc015402e7acfa091fa02` |
| LAYOUT_OVERFLOW_DRAW | line=2 y=1048.2 overflow=20.1px | line=2 y=1048.2 overflow=20.1px |

→ **PR #644 도입 회귀 아님**, devel 에 이미 존재하던 결함.

### 결함 위치
`src/renderer/layout/paragraph_layout.rs:875` (Task #332 Stage 4b 잔존 영역, "drift 의 본질적 해결은 Stage 5" 명시 — Stage 5 미수행).

### 후속 처리
Issue #716 으로 별도 task 진행.

## 4. 5 축 정정 효과

| 축 | 위치 | 정정 |
|----|------|------|
| 1 | `pagination/engine.rs:846-852` fit 루프 | 마지막 줄 lh-only (트레일링 ls 제외) |
| 2 | `typeset.rs:907-914` LAYOUT_DRIFT_SAFETY_PX | 10 → 4px |
| 3 | `layout.rs:1521` VPOS_CORR backward | 1.0 → 8.0px |
| 4 | `layout.rs:1504` VPOS_CORR end_y | sb_N 사전 차감 |
| 5 | `typeset.rs:566-606` Task #404 vpos_end | `line_segs.last()` 직접 사용 |

### 핵심 통찰
- HWP: `vpos_(N+1) - vpos_N = lh_total + ls_total + sa_N + sb_(N+1)`
- Layout: y_advance per pi = `sb_N + lh_total + ls_total`
- → `sb_N ≠ sb_(N+1)` 시 누적 드리프트 → LAYOUT_OVERFLOW

## 5. 메모리 룰 적용 결과

### `feedback_v076_regression_origin`
> 외부 PR 컨트리뷰터들이 자기 환경 PDF 를 정답지로 사용 → 작업지시자 환경에서 회귀

→ **본 PR 에서 게이트 작동 확인**: 작업지시자 시각 검증으로 hongbo 결함 발견 (별건이지만 게이트의 본질 작동). 향후 PR 에서도 동일 절차 유지.

### `feedback_visual_regression_grows`
> 페이지 총 수 byte 비교만으로는 시각 결함 검출 불가. 작업지시자 시각 판정이 핵심 게이트

→ task554 sweep 12/12 통과 + 페이지 수 변화 (40→35) 정합에도 hongbo 결함은 시각 판정에서만 검출.

### `feedback_close_issue_verify_merged`
→ #643 close 는 PR 머지 + devel push 후 자동 처리 (closes #643).

## 6. 산출물

| 산출물 | 경로 |
|--------|------|
| 검토 보고서 | `mydocs/pr/pr_644_review.md` |
| 최종 보고서 | `mydocs/pr/pr_644_report.md` (본 문서) |
| 디버그 오버레이 SVG | `output/debug/pr644-page1-debug.svg` |
| merge commit | `42bb7946` |
| 별도 이슈 | #716 (hongbo page 1 LAYOUT_OVERFLOW_DRAW) |

## 7. 컨트리뷰터 응대

@planet6897 (12번째 사이클) 안내:
- 5 축 분해 본질 정확
- 결정적 검증 통과 + 페이지 수 정합 회복 확인
- merge 결정
- hongbo page 1 결함은 PR 도입 회귀 아닌 별건 (Task #332 잔존), Issue #716 별도 처리

작성: 2026-05-08
