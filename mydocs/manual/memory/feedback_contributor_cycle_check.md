---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-17
name: 컨트리뷰터 사이클 영역 사전 점검 의무
description: PR review 시작 시 gh pr list --author <login> --state all 영역으로 컨트리뷰터 영역의 PR 누적 영역 직접 영역 점검 의무. "첫 사이클" 영역의 표현 영역 임의 추정 금지
type: feedback
originSessionId: 4ef64500-5782-4a52-a9a8-9330ea6c128b
---
PR review 시 컨트리뷰터 영역의 사이클 영역 표현 (몇 번째 사이클 PR) 영역은 **반드시 직접 영역 점검 영역 후 영역 사용 영역**. 임의 추정 영역 (예: 본 PR 영역의 commit author 영역만 보고 영역 "첫 사이클" 영역으로 영역 가정 영역) 영역 금지 영역.

**Why:** PR #673 review 영역에서 @jangster77 (Taesup Jang) 영역을 "첫 사이클 PR" 영역으로 영역 review/report/orders 영역 모두 영역 기재 영역. 작업지시자 정정 영역: "이 컨트리뷰터도 여러번 PR 하신 분입니다." 직접 영역 점검 결과 영역 — @jangster77 영역은 PR #451 (HWP 3.0 파서 초기 구현) 부터 영역 PR #678 까지 영역 13개 PR 영역의 누적 영역 영역 핵심 컨트리뷰터 영역. 본 환경 영역의 HWP 3.0 파서 영역은 본 컨트리뷰터 영역의 영역 영역으로 영역 도입 영역. "첫 사이클" 영역의 표현 영역은 컨트리뷰터 영역의 누적 기여 영역을 영역 무시 영역하는 결함 영역.

**How to apply:**

### review 영역 시작 시 (필수 순서)
1. PR 정보 영역 점검 영역 후 영역 (gh pr view <N>) **즉시** 영역:
   ```bash
   gh pr list --repo edwardkim/rhwp --author <login> --state all --limit 30 \
     --json number,title,state,mergedAt
   ```
2. 컨트리뷰터 영역의 PR 영역 누적 영역 정확 영역 산출 영역
3. review 보고서 영역 작성 시 영역 정확한 사이클 번호 영역 영역 사용 영역

### 표현 영역의 정확성
- "첫 사이클 PR" 영역 — `gh pr list` 결과 영역 1건 영역인 경우 영역만 영역 사용
- "N번째 사이클 PR" 영역 — 정확한 누적 건수 영역
- 핵심 영역 기여 영역의 컨트리뷰터 영역의 경우 영역 — 핵심 기여 영역도 영역 함께 명시 (예: "HWP 3.0 파서 영역의 핵심 영역 컨트리뷰터")

### 알려진 핵심 컨트리뷰터 영역 (2026-05-08 기준)
- **@planet6897 (Jaeuk Ryu)** — Layout 영역 / 페이지네이션 영역 다회 사이클 (16+)
- **@jangster77 (Taesup Jang)** — HWP 3.0 파서 영역 핵심 영역 (PR #451 부터, 13+)
- **@postmelee (Taegyu Lee)** — rhwp-studio 영역 / Firefox AMO 영역 (PR #339 부터, 다회)
- **@johndoekim (johndoe)** — hit_test 영역 정정 영역 (PR #645 등)
- **@oksure (Hyunwoo Park)** — rhwpDev 디버깅 툴킷 영역 (PR #602/#684 등)

### 컨트리뷰터 응대 영역의 본질
다회 사이클 영역 컨트리뷰터 영역에 "첫 사이클" 영역으로 영역 잘못 영역 기재 영역하면 영역 누적 기여 영역 무시 영역 + `feedback_pr_comment_tone` 영역의 차분한 사실 중심 영역 정합 영역 위반 영역.

**관련 룰**:
- `feedback_pr_comment_tone` — 차분한 사실 중심 응대
- `feedback_pr_supersede_chain` — 동일 컨트리뷰터 영역의 PR 영역 점검 영역
