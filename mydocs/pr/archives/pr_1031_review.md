# PR #1031 검토 — HWP3 외곽선 paper-edge 정합 회귀 정정 (Task #1029)

## 1. PR 정보

| 항목 | 값 |
|------|-----|
| 번호 | #1031 |
| 제목 | Task #1029: HWP3 외곽선 paper-edge 정합 회귀 정정 (closes #1029) |
| 작성자 | jangster77 (Taesup Jang) — 기존 컨트리뷰터 (PR #989/#995/#1003 등 sample16/HWP3 계열 연속) |
| base ← head | `devel` ← `jangster77:local/task1029` |
| 연결 이슈 | `closes #1029` (OPEN, jangster77 본인 작성, **assignee 없음** — §3.5) |
| mergeable | MERGEABLE |
| CI | Build & Test ✅ / Analyze rust·js·py ✅ / Canvas visual diff ✅ / CodeQL ✅ / WASM skip |
| 변경 | 7 파일 +846 / -12 — 소스 1 (`layout.rs` +31/-12), 문서 6 |
| 생성 | 2026-05-20 07:35 |

## 2. 배경 — 회귀 사고의 사슬

### 2.1 회귀 도입 경위 (PR 본문 + git 검증)

| 단계 | merge | PR | 영향 |
|------|-------|-----|------|
| 회귀 기준선 | `850cfb54` | #1011 (closes #1006) | `PageBorderBasis::PaperBased` 통합 contract 도입 — HWP3/HWP5/HWPX 모두 paper-edge |
| 회귀 도입 | `c2024ec9` | #1003 (closes #990) | Task #990 cherry-pick 시 PR #1011 영역과 충돌 → `--theirs` 자동 해소로 PR #987 시절 `attr & 0x01` 비트 로직으로 **무심코 revert** |
| 회귀 정정 | 본 PR | #1031 | `layout.rs` 4 hunk 복원 |

bisect 표 (PR 본문 §Bisect) 가 `850cfb54` → `c2024ec9` 사이 `border_top y` 가
`17.88px` → `55.64px` 로 튀는 시점을 정확히 특정. **본 환경에서 git
검증**: PR #1031 적용 후 `layout.rs` build_page_borders 영역이 PR #1011
baseline (`850cfb54`) 과 정확히 일치함 — 복원이 사실 그대로.

### 2.2 포맷별 attr 단언 (PR 본문 표)

| 포맷 | attr | basis | `attr & 0x01` (회귀 로직) | 결과 |
|------|------|-------|---------------------------|------|
| HWP3 native | 0x00000000 | PaperBased | **false** (revert) | **body-edge (회귀)** |
| HWP5 변환본 | 0x00000001 | PaperBased | true | paper-edge (우연히 OK) |
| HWPX 변환본 | 0x00000041 | PaperBased | true | paper-edge (우연히 OK) |

HWP5/HWPX 는 변환 과정에서 attr bit0 가 우연히 1 이라 회귀가 가려졌고
HWP3 만 attr=0 (CLAUDE.md HWP3 격리 규칙: 파서가 attr=0 주입) 이라 회귀
노출. Parser 측 PR #1011 의 `basis=PaperBased` 주입은 PR #1003
cherry-pick 시 보존됨 — fix 는 renderer (`layout.rs`) 한정 일관.

## 3. 검토 항목

### 3.1 변경 내용 (4 hunk in `src/renderer/layout.rs`)

| Hunk | 위치 | 변경 |
|------|------|------|
| A | `page_number_baseline_y()` L947 | `(pbf.attr & 0x01) != 0` → `matches!(pbf.basis, PageBorderBasis::PaperBased)` |
| B | `build_page_borders()` L964 | A 와 동일 + 주석 history 정정 + `footer_inside = (attr & 0x04) != 0` 추가 + 디버그 로그 비트 분해 |
| C | L1001 | `(bx, by, bw, bh)` → `(bx, mut by, bw, mut bh)` |
| D | L1015 | `footer_inside` clip 블록 복원 — 페이지 번호 외곽선 바깥 위치 정합 (PR #1011) |

전체 변경: 31+/12-. parser/model 무수정. renderer 단일 파일.

### 3.2 설계 적합성 — 메모리 룰 정합 ✅

- **`feedback_hancom_compat_specific_over_general`**: `attr & 0x01` 같은
  측정 의존 분기 (포맷별 attr 의미가 서로 다름) 를 `matches!(basis,
  PaperBased)` 구조 가드로 복원. 포맷별 비일관성을 parser 가
  normalize (basis 필드) → renderer 는 구조 분기 — 룰 정확 정합.
- **회귀 가시성 분리**: HWP3/HWP5/HWPX 가 한 코드 경로를 공유하므로
  HWP3 만 회귀하던 사고가 다른 두 포맷에서 가려졌음. basis 필드 도입
  자체가 이런 우연 일치를 차단하는 적절한 설계.

### 3.3 코드 품질 — 지적 (수정 요청 불요)

다음 두 항목은 **본 PR 자체의 새 문제가 아니라 PR #1011 baseline
(`850cfb54`) 의 스타일을 그대로 복원한 결과**. git 으로 baseline 과
복원 결과 비교 시 build_page_borders 영역이 byte-identical. 본 PR 책임
아님 → 별도 cleanup task 후보:

**(a) 함수 내부 `use` 문**: `use crate::model::page::PageBorderBasis;`
가 hunk A (L948) + hunk B (L984) 두 곳에 중복. 파일 상단 use 통합 또는
함수 내 단일 use 로 정리 권장 — 별도 task.

**(b) `let _ = &mut by;` unused_mut 보정**: hunk C 에서 `mut by` 로
선언하나 실제 변경은 `bh` 만 일어남. `let _ = &mut by;` 가 unused_mut
경고 회피용 보정. 직관적으로는 `mut` 선언만 떼면 됨 — 별도 cleanup.

→ 두 항목 모두 본 PR 의 변경 책임 외, merge blocker 아님. PR
#1011 시점에 들어온 스타일이며 본 PR 은 baseline 충실 복원.

### 3.4 검증 충실성 — 작업지시자 시각 판정 명시 ⚠️ 확인 필요

PR body / 보고서가 제시한 검증:
- cargo build --release: warning 0 ✅
- cargo clippy --release --lib -- -D warnings ✅
- cargo fmt --check ✅
- cargo test --release --lib: 1307 passed / 0 failed ✅
- cargo test --release --tests: 68 integration / 0 failed ✅
- 12 fixture 페이지 수 sweep 정상 (Task #990 효과 보존)
- 3 포맷 paper_based=true + border_top y=17.88 단언 ✅
- **"작업지시자 한컴 viewer 시각 검증 통과 (HWP3 cover paper-edge 정합 복원)"** ← ⚠️

메모리 룰 `feedback_visual_judgment_authority` (시각 판정 권위는
작업지시자) / `feedback_pdf_not_authoritative` 정합 — PR body 가
선언적으로 "작업지시자 시각 검증 통과" 라 기록하나, **실제 작업지시자가
이미 검증을 수행한 것인지 본 검토 시점에 확인 필요**. 컨트리뷰터
자가검증이면 메모리 룰 `feedback_v076_regression_origin` (v0.7.6 회귀의
origin = 컨트리뷰터 자기 환경 PDF 정답지) 패턴 재발 위험.

추가 결정적 게이트로 본 환경에서 `samples/hwp3-sample16.hwp` p0 SVG
생성 → `border_top y=17.88px` 단언 정량 측정 보조 가능.

### 3.5 이슈 #1029 — assignee 누락 ⚠️

메모리 룰 `feedback_assign_issue_before_work` (이슈 착수 시 즉시
assignee 지정 필수, 일차 방어선) 정면 관련:
- #1029 는 jangster77 본인이 작성 + 본인이 PR 작성. 그러나 issue
  assignee 가 비어 있음.
- "본인 이슈 본인 PR" 패턴은 외부 컨트리뷰터에게 "오픈 타스크" 로
  보이지 않는 사이드 효과는 있으나, 룰의 정확한 적용 — 작업
  결정 즉시 assign 의무 — 는 본 케이스에도 적용 가능.
- **본 PR 처리와는 독립**. PR merge 자체에는 영향 없음. 후속으로 PR
  내부 작업 패턴 안내 권고 후보.

### 3.6 재발 방지 권고 (PR 본문 §재발 방지)

PR 본문이 자체적으로 제안한 3개 항목:
1. cherry-pick `--theirs`/`--ours` 자동 해소 후 영향 hunk line-by-line 검토
2. 시각 회귀 sweep 을 비공개 샘플 외 공개 fixture (`hwp3-sample16`) 로 사전 단언 의무화
3. PR merge 직전 `RHWP_DEBUG_PAGE_BORDER=1` 같은 진단 hook 으로 외곽선 좌표 baseline diff 자동 단언

3개 모두 합리적. 메모리 룰 후보 — `feedback_cherry_pick_theirs_caution`
(가칭) 추가 검토. 본 PR scope 외 → 별도 troubleshooting / 메모리 룰
정리 task 분리 권장.

## 4. 처리 절차 (간소화 4단계)

1. ✅ PR 정보 확인 (본 문서 §1~2)
2. → 본 검토 문서 작성 + 승인 요청 (현 단계)
3. (불요 예상) `pr_1031_review_impl.md` — 코드 품질 (a)(b) 는 PR #1011 baseline 정합 → 본 PR 정정 요청 아님
4. 검증 (로컬 빌드/테스트/clippy + 작업지시자 시각 판정 확인) → `pr_1031_report.md`

## 5. 1차 판단 (작업지시자 승인 전 잠정)

| 영역 | 평가 |
|------|------|
| 설계 방향 | ✅ 적합 — `attr` 비트 → `basis` 구조 가드, 메모리 룰 정합 |
| CI / 결정적 검증 | ✅ 통과 (1307+68 / 0 failed, 페이지 수 sweep 정상) |
| baseline 충실성 | ✅ PR #1011 (`850cfb54`) build_page_borders 영역과 byte-identical 복원 확인 |
| 코드 품질 | ⚠️ (a)(b) — PR #1011 baseline 스타일 그대로, 본 PR 책임 외 (별도 cleanup) |
| 시각 검증 | ⚠️ **확인 필요** — PR 본문 "작업지시자 시각 검증 통과" 가 실제 수행 사실인지 (3.4) |
| 이슈 연결 | #1029 assignee 누락 (3.5, merge blocker 아님) |
| 재발 방지 | PR 본문이 합리적 권고 3건 제안 — 별도 정리 task 후보 (3.6) |

**잠정 결론**: 회귀 정정의 정확성/충실성은 git 검증으로 확인됨 (PR
#1011 baseline 정합). 결정적 검증 전부 통과. **머지 전 1개 게이트**:
작업지시자의 HWP3 sample16 cover 시각 판정 — PR 본문이 이미 수행되었다
주장하는데 실제 사실인지 확인 (자가검증이면 본 환경 직접 판정 필요).

> 본 문서는 검토 계획 + 항목 통합. 작업지시자 승인/피드백 후
> 검증 단계 → `pr_1031_report.md` 로 최종 판단 기록.
