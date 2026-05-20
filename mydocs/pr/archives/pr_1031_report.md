# PR #1031 최종 보고서 — HWP3 외곽선 paper-edge 정합 회귀 정정

- PR: [#1031](https://github.com/edwardkim/rhwp/pull/1031)
- 제목: Task #1029: HWP3 외곽선 paper-edge 정합 회귀 정정 (closes #1029)
- 작성자: jangster77 (Taesup Jang) — 누적 기여자 (PR #989/#995/#1003 등 sample16/HWP3 계열 연속)
- base ← head: `devel` ← `jangster77:local/task1029`
- 결정: **merge (수용)** — 코드 품질 cleanup 동반 없이 그대로
- 일자: 2026-05-20

## 1. 결정

**merge 수용.** PR #1031 의 `layout.rs` 4 hunk 복원은 회귀 도입
commit (PR #1003 cherry-pick `--theirs` 사고) 이전 상태 (PR #1011
baseline `850cfb54`) 와 byte-identical 정합. 모든 검증 게이트 통과
+ 작업지시자 시각 판정 사실 확인.

코드 품질 지적 2건 (검토 §3.3 a/b) 은 **PR #1011 baseline 자체의 스타일**
이며 본 PR 의 책임 외 — cleanup 동반 안 함. 별도 task 후보로 남김.

이슈 #1029 는 OPEN, jangster77 본인 작성, assignee 비어있는 상태.
PR merge 시 `closes #1029` 가 자동 close 처리.

## 2. 검증 결과

| 게이트 | 결과 | 비고 |
|--------|------|------|
| CI: Build & Test | ✅ pass | |
| CI: Analyze rust/js/py | ✅ pass | |
| CI: Canvas visual diff | ✅ pass | |
| CI: CodeQL | ✅ pass | |
| cargo build (네이티브, 본 환경) | ✅ | dev profile |
| cargo fmt --check | ✅ exit 0 | |
| cargo test --release --lib | ✅ 1307/0 (PR 보고) | |
| cargo test --release --tests | ✅ 68/0 (PR 보고) | |
| 페이지 수 sweep 12 fixture | ✅ 정상 (Task #990 효과 보존) | hwp3-sample10/11/13/14/16/16-hwp5/exam_kor/eng/math/aift/biz_plan |
| WASM Docker 빌드 (release + wasm-opt) | ✅ Done in 1m 31s | `pkg/rhwp_bg.wasm` 4.6M |
| **PR #1011 baseline 충실성 (git 검증)** | ✅ `build_page_borders` 영역 byte-identical | 본 환경 git show 850cfb54 ↔ pr-1031 직접 비교 |
| **작업지시자 시각 판정 (HWP3 sample16 cover)** | ✅ **사실 확인 + 정합** | "이미 수행, 정합 확인" — PR 본문 주장 사실 |

메모리 룰 정합:
- `feedback_visual_judgment_authority`: 작업지시자 직접 시각 판정 확인 ✓
- `feedback_pdf_not_authoritative` / `feedback_v076_regression_origin`: 자가검증 우려 해소 ✓

## 3. 회귀 사고 본질 (정리)

```
850cfb54 PR #1011 ─ PageBorderBasis::PaperBased 통합 contract 도입
                   border_top y = 17.88px (paper-edge, 기준선)
   ↓
c2024ec9 PR #1003 ─ Task #990 cherry-pick `--theirs` 자동 해소 사고
                   PR #1011 영역의 basis 로직을 attr & 0x01 비트로 무심코 revert
                   → HWP3 native(attr=0)만 회귀, border_top y = 55.64px
                     HWP5/HWPX는 attr bit0=1로 우연히 가려짐
   ↓
PR #1031 ─ layout.rs 4 hunk 복원 → PR #1011 baseline 정확 일치
            border_top y = 17.88px (복원)
```

### 포맷별 attr 단언 표

| 포맷 | attr | basis | 회귀 로직 결과 | 본 fix 결과 |
|------|------|-------|----------------|-------------|
| HWP3 native | 0x00000000 | PaperBased | body-edge (회귀) | paper-edge (복원) |
| HWP5 변환본 | 0x00000001 | PaperBased | paper-edge (우연 OK) | paper-edge (동일) |
| HWPX 변환본 | 0x00000041 | PaperBased | paper-edge (우연 OK) | paper-edge (동일) |

## 4. 설계 평가

- **메모리 룰 `feedback_hancom_compat_specific_over_general` 정합**:
  포맷별 비일관성 (attr 의미 서로 다름) 을 parser 가 normalize (basis
  필드) → renderer 는 구조 분기 (`matches!(basis, PaperBased)`).
  측정 의존 분기보다 구조 가드가 안전한 사례.
- **회귀 가시성 분리**: PR #1003 사고가 HWP3 만 회귀하고 HWP5/HWPX 에
  서 가려진 점이 `attr & 0x01` 측정 의존의 위험성을 드러냄. basis 필드
  도입 자체가 이 우연 일치를 차단하는 적절한 설계.
- **scope 정확**: renderer (`layout.rs`) 단일 파일 4 hunk. parser/model
  무수정. 회귀 정정 범위가 최소 충실.

## 5. cherry-pick 처리

PR 고유 commit (devel merge `9e64da6c` 제외):
- `944ddff2` Task #1029 Stage 1: layout.rs 4 hunk 복원 (소스)
- `9bd851f4` Task #1029 Stage 2: 회귀 sweep 검증 통과 보고서
- `a307e207` Task #1029 Stage 3: 최종 결과 보고서 + orders 갱신

처리: 3 commit author (jangster77) 보존 cherry-pick. 본 환경 정합
clean-up 별도 commit 없음 (코드 품질 지적은 baseline 정합 → 본 PR scope 외).

## 6. 잔존 / 후속

### 본 PR scope 외 — 별도 task 후보

- **코드 품질 cleanup** (검토 §3.3 a/b):
  - 함수 내부 중복 `use crate::model::page::PageBorderBasis;` (hunk A·B 양쪽)
  - `let _ = &mut by;` unused_mut 보정 — `mut` 선언 제거로 단순화 가능
  - 둘 다 PR #1011 baseline 그대로 → 본 PR 책임 외, 별도 cleanup task 분리 권고
- **이슈 #1029 assignee 누락**: 본인 작성 + 본인 PR 패턴. 메모리 룰
  `feedback_assign_issue_before_work` 적용 안내 후보. merge blocker 아님.
- **재발 방지 권고 3건** (PR 본문 §재발 방지): cherry-pick `--theirs`
  주의 / 공개 fixture 사전 단언 / `RHWP_DEBUG_PAGE_BORDER` 진단 hook.
  troubleshooting 문서 또는 메모리 룰 (`feedback_cherry_pick_theirs_caution`
  가칭) 추가 정리 task 분리 권고.

### 분리 보존 — 본 PR scope 외, 영역 별개

- 다른 OPEN PR (#1032 planet6897 Task #1027, #1026 HaimLee-4869, #1019
  postmelee, #950 dragonnite1221-lgtm) — 본 PR 처리와 독립

## 7. 산출물

- `mydocs/pr/pr_1031_review.md` (검토 문서)
- 본 보고서
- 소스: PR `layout.rs` 4 hunk 복원 (PR #1011 baseline 정합)

## 8. 메모리 룰 갱신 검토

- `project_external_contributors`: jangster77 = 등재된 누적 기여자.
  갱신 불요.
- 신규 룰 후보: `feedback_cherry_pick_theirs_caution` (가칭) — cherry-pick
  자동 충돌 해소 후 영향 hunk 검토 의무. 본 PR 처리에서 직접 추가하지
  않고 별도 정리 task 로 분리 (PR 본문 재발 방지 §3 권고와 합쳐 처리).
- 본 PR 처리 자체로 추가할 룰 없음 — 기존 룰
  (`feedback_hancom_compat_specific_over_general`,
  `feedback_visual_judgment_authority`,
  `feedback_pdf_not_authoritative`) 의 권위 사례.
