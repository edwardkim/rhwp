---
PR: #813
제목: fix — improve PDF export fidelity (LINE_SEG priority + HWPX adapter + font fallbacks)
컨트리뷰터: @PhilipKim85 — 2번째 PR (PR #798 close 후속, 본질적으로 첫 머지 시도)
base / head: **main** / fix/pdf-bbox-clipping
mergeStateStatus: BLOCKED
mergeable: MERGEABLE (그러나 base=main 정책 위반)
CI: 결과 부재
변경 규모: +192 / -22, 7 files
커밋: 6
검토일: 2026-05-11
---

# PR #813 검토 — ⚠️ 다수 문제

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #813 |
| 제목 | fix: improve PDF export fidelity (LINE_SEG priority + HWPX adapter + font fallbacks) |
| 컨트리뷰터 | @PhilipKim85 — 2번째 PR (PR #798 close 후, 본질적으로 첫 머지 시도) |
| base / head | **main** / fix/pdf-bbox-clipping — ⚠️ base 정책 위반 |
| mergeable | MERGEABLE / BLOCKED |
| CI | 결과 부재 |
| 변경 규모 | +192 / -22, 7 files |
| 커밋 수 | 6 |
| closes | 부재 — Issue 연결 없음 |

## 2. ⚠️ 주요 문제 1 — base=main 정책 위반

본 환경 영역 영역 외부 컨트리뷰터 PR base 정책:
> **컨트리뷰터 워크플로우 (Fork 기반)** — 원본 저장소의 **devel** 로 PR 생성 (CLAUDE.md 명시)

본 PR base=main → release cycle 영역 영역 직접 진입 시도. **devel base 재제출 요청 필수**.

## 3. ⚠️ 주요 문제 2 — 폰트 라이선스 정책 위반

본 환경 영역 영역 폰트 정책 (`mydocs/tech/font_fallback_strategy.md` 명시):
> 한컴/HY/MS 폰트 모두 **재배포 불가** + 오픈소스 (Noto Serif KR / Noto Sans KR / Pretendard) 영역 영역 폴백 권장

### 본 PR 영역 영역 추가된 fallback chain (`pdf.rs`)
```rust
// 한컴 상용 폰트 영역 영역 우선
"한컴바탕" → "한컴바탕, HCR Batang, Haansoft Batang, 바탕, serif"
"함초롬바탕" → "함초롬바탕, HCR Batang, Haansoft Batang, 바탕, serif"
"HY견고딕" → "HY견고딕, HYGothic-Medium, HYgprM, 돋움, sans-serif"
"HY견명조" → "HY견명조, HYmjrE, 바탕, serif"
"HY신명조" → "HY신명조, HYSinMyeongJo-Medium, 바탕, serif"
"HY중고딕" → "HY중고딕, HYGothic-Medium, HYgtrE, 돋움, sans-serif"
"HY헤드라인M" → "HY헤드라인M, HYHeadLine-Medium, HYgtrE, 돋움, sans-serif"
"휴먼명조" → "휴먼명조, 휴먼모음T, 바탕, serif"
"휴먼고딕" → "휴먼고딕, 휴먼모음T, 돋움, sans-serif"
```

### 라이선스 문제
- `HCR Batang/Dotum` — 한컴 오피스 번들 폰트 (한컴 라이선스)
- `Haansoft Batang/Dotum` — 한컴 상용 폰트
- `HYGothic-Medium` / `HYmjrE` / `HYSinMyeongJo-Medium` / `HYHeadLine-Medium` / `HYgprM` / `HYgtrE` — HY (서울시스템 자회사) 상용 폰트
- `휴먼모음T` — 휴먼소프트 상용 폰트

→ fallback chain 영역 영역 폰트 파일 임베드 부재 (시스템 영역 영역 설치돼 있어야 동작) 영역 영역 라이선스 직접 위반 영역 영역 부재. 그러나 본 환경 정책 영역 영역:
- **오픈소스 우선** (Noto Serif KR / Noto Sans KR / Pretendard) 권장
- 상용 폰트명 영역 영역 사용자 환경 영역 영역 없을 가능성 — 오픈소스 폴백 영역 영역 안정성 정합

본 PR fallback chain 영역 영역 한컴/HY 영역 영역 우선 + 오픈소스 미포함 영역 영역 본 환경 정책 영역 영역 어긋남. **fallback chain 영역 영역 Noto Serif KR / Noto Sans KR / Pretendard 추가 필수**.

## 4. ⚠️ 주요 문제 3 — typeset.rs LINE_SEG 가드 회귀 위험

### 본 환경 영역 영역 누적된 LINE_SEG 가드 (5/6~5/7 사이클)
| PR | 본질 |
|----|------|
| PR #621 | typeset.rs 영역 영역 다중 줄 가드 (`line_segs.len() >= 2`) |
| PR #622 | 다단 vpos-reset (`col_count > 1 && li > cursor_line && vertical_pos == 0`) |
| PR #627 | exam_science p2 글상자 ㉠ 사각형 y 회귀 정정 (Task #520 부분 회귀 복원) |
| PR #632 | vpos-reset 인접 line 보존 더블체크 (`LAYOUT_DRIFT_SAFETY_PX = 10.0` 2회 차감 영역 정정) |
| PR #636 | aift p4 목차 `·` 포함 라인 alignment + Issue #635 흡수 |

→ **typeset.rs 영역 영역 매우 정교하게 누적된 회귀 정정 영역** — `feedback_hancom_compat_specific_over_general` 권위 사례 누적.

### 본 PR 영역 영역 변경
```rust
let has_valid_line_segs = !para.line_segs.is_empty()
    && para.line_segs.iter().any(|seg| seg.line_height > 0);

if has_valid_line_segs {
    // HWP LINE_SEG 영역 영역 무조건 우선
    para.line_segs.iter().map(...).unzip()
} else if let Some(comp) = composed {
    // Composer fallback
}
```

→ **HWP LINE_SEG 영역 영역 무조건 우선** 사용 — 누적된 Composer 정정 영역 영역 우회.

### 회귀 위험
- PR #621 (다중 줄 가드) — Composer 영역 영역 정정된 line_height 영역 영역 LINE_SEG 영역 영역 우선되면 회귀 가능
- PR #632 (LAYOUT_DRIFT_SAFETY_PX 2회 차감 정정) — Composer 경로 영역 영역 정정 영역 영역 LINE_SEG 직접 사용 영역 영역 우회
- PR #636 (aift p4 목차) — Composer 영역 영역 정정된 alignment 영역 영역 우회

→ **광범위 sweep 필수** — 본 환경 fixture (170 페이지) + 컨트리뷰터 PR 본문 명시 fixture (HWP 10p `~88%`, HWPX 10p `~64%`) 영역 영역 시각 검증.

### 검증 부족
PR 본문 영역 영역 "Content Match ~88% (HWP) / ~64% (HWPX)" 명시 — **결정적 검증 (sweep / byte 비교) 부재**. PR #621/#622/#632/#636 사이클 영역 영역 광범위 sweep (160+ fixture / 1,000+ 페이지 / 회귀 0) 영역 영역 표준. 본 PR 영역 영역 표준 미달.

## 5. ⚠️ 주요 문제 4 — Issue 연결 부재

본 환경 영역 영역 절차 정합 영역 영역 Issue → 브랜치 → 계획서 → 구현 순서. 본 PR 영역 영역 Issue 연결 부재 — `closes` 키워드 부재. 본 환경 절차 영역 영역 어긋남.

## 6. ⚠️ 주요 문제 5 — wasm_api.rs pub(crate) core

`HwpDocument` 영역 영역 `pub(crate) core` 변경 + `apply_hwpx_adapter` 신규 메서드 추가. `pub(crate)` 변경 영역 영역 API surface 영역 영역 외부 컨트리뷰터 영역 영역 임의 노출 — 본 환경 영역 영역 점검 필요.

## 7. 정정 본질 — 6 commits, 7 files

### 7.1 commits
| commit | 본질 |
|--------|------|
| `27b7604c` | prevent PDF content clipping by expanding Form XObject BBox |
| `ad7994df` | prioritize original LINE_SEG data for page layout fidelity |
| `2212f08e` | handle HWPX zero LINE_SEG + font fallback expansion |
| `5e1b36bd` | apply HWPX adapter before initial pagination in from_bytes |
| `46e1c8e8` | expand HWPX reflow conditions and reorder initialization |
| `d334eb45` | treat single TAC tables in empty paragraphs as inline |

### 7.2 변경 파일
| 파일 | 변경 |
|------|------|
| `src/renderer/typeset.rs` | LINE_SEG 우선 (Composer fallback) |
| `src/renderer/pdf.rs` | 폰트 fallback 확장 + SVG viewport BBox 확장 + scan_svg_max_x + expand_svg_viewport |
| `src/document_core/commands/document.rs` | HWPX adapter 영역 영역 from_bytes 영역 영역 호출 + reflow 조건 확장 |
| `src/wasm_api.rs` | apply_hwpx_adapter pub + pub(crate) core |
| `src/renderer/height_measurer.rs` | single TAC table 영역 영역 inline 처리 (`tac_tables.len() >= 2` → `!tac_tables.is_empty()` + `seg_width == 0` 가드) |
| `src/renderer/layout.rs` | 빈 라인 |
| `src/renderer/layout/table_layout.rs` | 빈 라인 |

## 8. 본 환경 충돌 가능성

base=main + devel 5/11 누적 (12 PR 머지) 영역 영역 다수 충돌 가능 — typeset.rs / document.rs / pdf.rs 모두 5/6~5/7 사이클 영역 영역 정교 정정 누적.

## 9. 처리 권장 — 옵션 C (재제출 요청)

본 PR 영역 영역 다수 문제:
1. **base=main 정책 위반** — devel 재제출 필수
2. **폰트 라이선스 정책 어긋남** — Noto Serif KR / Noto Sans KR / Pretendard 폴백 추가 요청
3. **typeset.rs LINE_SEG 가드 회귀 위험** — 5/6~5/7 사이클 누적 정정 영역 영역 광범위 sweep 필수
4. **Issue 연결 부재** — Issue 영역 영역 본질 분리 + 연결 요청
5. **wasm_api.rs pub(crate) core** — API surface 변경 영역 영역 본 환경 점검 필요
6. **분리 PR 권장** — 본 PR 영역 영역 3가지 본질 (LINE_SEG / HWPX adapter / 폰트 fallback) 영역 영역 별 PR 분리 가이드

### 옵션 C 권장 — 컨트리뷰터에 재제출 요청
1. base 영역 영역 devel 영역 영역 변경
2. 본 PR 영역 영역 본질 영역 영역 3 분리 PR 영역 영역 분리:
   - **분리 PR 1** — pdf.rs SVG viewport BBox 확장 + scan_svg_max_x (가장 안전, PR #798 후속)
   - **분리 PR 2** — 폰트 fallback (Noto 우선 + 한컴/HY 보조)
   - **분리 PR 3** — typeset.rs LINE_SEG 우선 + HWPX adapter (5/6~5/7 사이클 정합 영역 영역 광범위 sweep 필수)
3. 각 분리 PR 영역 영역 Issue 연결
4. 광범위 sweep 결과 명시

### 옵션 B — 본 환경 영역 영역 부분 cherry-pick + 변형
- pdf.rs SVG BBox 확장 (commits `27b7604c`) 영역 영역 영역 본 환경 폴백 정책 정합 영역 영역 변형 후 cherry-pick
- LINE_SEG + HWPX adapter 영역 영역 광범위 sweep 결과 + 회귀 0 입증 후 별 PR 영역 영역 cherry-pick
- 매우 복잡 — 옵션 C 권장

### 옵션 A — 부재
base=main + 라이선스 정책 위반 + 회귀 위험 + Issue 연결 부재 영역 영역 직접 머지 옵션 부재.

## 10. 처리 옵션 권장 — **옵션 C (재제출 요청)**

본 환경 영역 영역 다수 정책 위반 + 회귀 위험 영역 영역 작업지시자 결정 권장:
- **옵션 C-1**: 컨트리뷰터에 본 PR 영역 영역 close + 5 문제점 정리 + 분리 PR 가이드 + 재제출 요청 (정중 톤, `feedback_pr_comment_tone`)
- **옵션 C-2**: 본 PR 영역 영역 일부 + 변형 cherry-pick (메인테이너 부담)

## 11. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @PhilipKim85 — **2번째 PR** (PR #798 close 후) |
| `feedback_pr_comment_tone` | 신규 컨트리뷰터 영역 영역 차분 톤 + 정정 요청 (다수 문제점 정리) |
| `feedback_external_docs_self_censor` | 폰트 라이선스 — 한컴/HY 상용 폰트 영역 영역 본 환경 정책 영역 영역 어긋남 |
| `feedback_hancom_compat_specific_over_general` | typeset.rs 영역 영역 누적된 정정 (PR #621/#622/#627/#632/#636) 영역 영역 LINE_SEG 무조건 우선 영역 영역 회귀 위험 |
| `feedback_process_must_follow` | Issue → 브랜치 → 계획서 → 구현 순서 — Issue 연결 부재 |
| `feedback_release_sync_check` | base=main 영역 영역 release cycle 영역 영역 직접 진입 시도 — devel 재제출 필수 |
| `feedback_visual_judgment_authority` | "Content Match ~88% / ~64%" 영역 영역 결정적 검증 부재 — 광범위 sweep 필수 |

## 12. 작업지시자 결정 권장

본 PR 영역 영역 5 주요 문제 (base / 폰트 / LINE_SEG / Issue / pub(crate)) 영역 영역 머지 권장 부재. **옵션 C (재제출 요청)** 권장.

만약 옵션 B (부분 cherry-pick) 영역 영역 진행 시:
- pdf.rs SVG BBox 확장 + scan_svg_max_x — 안전 (PR #798 후속, viewport 점검만)
- 폰트 fallback 영역 영역 — Noto 우선 + 한컴/HY 보조 변형 (작업지시자 결정)
- LINE_SEG + HWPX adapter — 별 PR 영역 영역 광범위 sweep 검증 후

---

작성: 2026-05-11
