---
PR: #813
제목: fix — improve PDF export fidelity (LINE_SEG priority + HWPX adapter + font fallbacks)
컨트리뷰터: @PhilipKim85 — 2번째 PR (PR #798 close 후, 본질적으로 첫 머지 시도)
처리: 옵션 C — 5 정책 위반 + 회귀 위험 영역 영역 close + 재제출 요청 (분리 PR 가이드)
처리일: 2026-05-11
관련: PR #798 close (5/11) 후속
---

# PR #813 처리 보고서

## 1. 처리 결과

✅ **close 완료** — 옵션 C (5 정책 위반 점검 + 재제출 요청 안내)

| 항목 | 값 |
|------|-----|
| 처리 | close 후 분리 PR 재제출 요청 |
| 컨트리뷰터 안내 | 정중 톤 + 5 문제점 정리 + 분리 PR 3 가이드 |
| 후속 | 컨트리뷰터 분리 PR 영역 영역 재제출 대기 |

## 2. 본 PR 본질

@PhilipKim85 영역 영역 PDF 출력 품질 개선 — 3 본질:
1. typeset.rs LINE_SEG 우선 (Composer 영역 영역 fallback)
2. document.rs HWPX adapter 영역 영역 from_bytes 영역 영역 호출
3. pdf.rs 폰트 fallback 확장 (한컴/HY/휴먼 시리즈 12개)

## 3. ⚠️ 5 정책 위반 + 회귀 위험

### 3.1 base = main 정책 위반
본 환경 영역 영역 외부 컨트리뷰터 워크플로우 (CLAUDE.md 명시): **devel base 영역 영역 PR 생성**. main 영역 영역 release cycle 영역 영역 직접 진입.

### 3.2 폰트 라이선스 정책 어긋남 (작업지시자 강조)
본 환경 정책 (`mydocs/tech/font_fallback_strategy.md`):
- 한컴/HY/MS 폰트 모두 재배포 불가
- 오픈소스 (Noto Serif KR / Noto Sans KR / Pretendard) 폴백 권장

본 PR fallback chain:
- `HCR Batang/Dotum` — 한컴 라이선스
- `Haansoft Batang/Dotum` — 한컴 상용
- `HYGothic-Medium` / `HYmjrE` / `HYSinMyeongJo-Medium` / `HYHeadLine-Medium` / `HYgprM` / `HYgtrE` — HY 상용
- `휴먼모음T` — 휴먼소프트 상용

폰트 임베드 부재 영역 영역 라이선스 직접 위반 부재 — 그러나 본 환경 정책 영역 영역 오픈소스 우선.

### 3.3 typeset.rs LINE_SEG 가드 회귀 위험 (가장 심각)

본 환경 영역 영역 5/6~5/7 사이클 누적 정정:
| PR | 본질 |
|----|------|
| #621 | 다중 줄 가드 |
| #622 | 다단 vpos-reset 가드 |
| #627 | 글상자 ㉠ y 회귀 정정 |
| #632 | LAYOUT_DRIFT_SAFETY_PX 2회 차감 정정 |
| #636 | aift p4 목차 alignment + Issue #635 흡수 |

본 PR `has_valid_line_segs` 가드 영역 영역 HWP LINE_SEG 무조건 우선 → 누적 Composer 정정 우회.

본 PR "Content Match ~88% (HWP) / ~64% (HWPX)" 영역 영역 결정적 검증 부재. 본 환경 표준 영역 영역 광범위 sweep (160+ fixture / 1,000+ 페이지 / 회귀 0) 영역 영역 미달.

### 3.4 Issue 연결 부재
`closes` 키워드 부재 — Issue → 브랜치 → 계획서 → 구현 절차 어긋남.

### 3.5 wasm_api.rs pub(crate) core
API surface 변경 영역 영역 외부 컨트리뷰터 영역 영역 임의 노출.

## 4. 컨트리뷰터 안내 본질 (정중 톤)

본 환경 영역 영역 댓글 [#813#issuecomment-4421118440](https://github.com/edwardkim/rhwp/pull/813#issuecomment-4421118440):
- PR #798 이어 다시 기여 환영 표시
- 5 정책 위반 점검 (base / 폰트 / typeset / Issue / pub(crate))
- 분리 PR 3 가이드 영역 영역 명시:
  - **분리 PR 1** — pdf.rs SVG viewport BBox 확장 (안전, PR #798 close 후속, 우선 권장)
  - **분리 PR 2** — 폰트 fallback (Noto Serif KR / Noto Sans KR / Pretendard 우선 + 한컴/HY 보조)
  - **분리 PR 3** — typeset.rs LINE_SEG + HWPX adapter (광범위 sweep + 회귀 0 입증 필수)

## 5. 5/6~5/7 사이클 영역 영역 권위 사례 (typeset.rs 정정 누적)

본 PR 영역 영역 회귀 위험 핵심 영역 — typeset.rs 영역 영역 누적된 정정:

| PR | 컨트리뷰터 | 본질 | 메모리 룰 |
|----|----------|------|----------|
| PR #621 | @planet6897 | typeset.rs 다중 줄 가드 (`line_segs.len() >= 2`) | `feedback_hancom_compat_specific_over_general` |
| PR #622 | @planet6897 | 다단 vpos-reset 가드 (col_count + cursor_line + vertical_pos) | 권위 사례 강화 |
| PR #627 | @planet6897 | exam_science 글상자 ㉠ y 회귀 (Task #520 부분 회귀 복원) | `feedback_close_issue_verify_merged` |
| PR #632 | @planet6897 | LAYOUT_DRIFT_SAFETY_PX 2회 차감 정정 (Task #332 stage4b 종결) | 권위 사례 누적 |
| PR #636 | @planet6897 | aift p4 목차 alignment + Issue #635 흡수 (TDD 5 단계) | 권위 사례 강화 누적 |

→ 본 환경 영역 영역 typeset.rs 영역 영역 매우 정교한 정정 보호.

## 6. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @PhilipKim85 — **2번째 PR** (PR #798 close 후) |
| `feedback_pr_comment_tone` | 신규 컨트리뷰터 영역 영역 차분 톤 + 정중한 정정 요청 |
| `feedback_external_docs_self_censor` | 폰트 라이선스 — 한컴/HY 상용 폰트 영역 영역 본 환경 정책 영역 영역 어긋남 점검 |
| `feedback_hancom_compat_specific_over_general` 권위 사례 강화 | typeset.rs 누적 정정 (PR #621/#622/#627/#632/#636) 영역 영역 LINE_SEG 무조건 우선 영역 영역 회귀 위험 점검 |
| `feedback_process_must_follow` | Issue 연결 부재 + 분리 PR 가이드 |
| `feedback_release_sync_check` | base=main 정책 위반 점검 |
| `feedback_visual_judgment_authority` | "Content Match ~88% / ~64%" 정성 측정 영역 영역 광범위 sweep 결정적 검증 미달 |
| `feedback_pr_supersede_chain` | PR #798 (close, viewport 축소 본질 결함) → PR #813 (close, 5 정책 위반) → 분리 PR 1/2/3 (재제출 대기) |

## 7. 잔존 후속

- 컨트리뷰터 영역 영역 분리 PR 1 (SVG BBox) 재제출 영역 영역 우선 머지 가능 영역 영역
- 분리 PR 2 (폰트 fallback) 영역 영역 Noto/Pretendard 변형 후 머지 검토
- 분리 PR 3 (LINE_SEG + HWPX adapter) 영역 영역 광범위 sweep + 회귀 0 입증 후 머지 검토

---

작성: 2026-05-11
