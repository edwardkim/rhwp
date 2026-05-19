---
name: "rhwp 첫 PR" 컨트리뷰터 환영 + 일관 처리 의식
description: 외부 첫 기여자 보고서에 "rhwp 첫 PR" 표현 명시, fork base 동기화 권장
type: feedback
originSessionId: 4861649d-834a-43c6-a262-9f08333360e8
---
외부 컨트리뷰터의 **첫 PR**은 보고서/처리 패턴에서 환영 표현을 명시하고, fork base 동기화 위험을 사전 안내한다.

**Why**: rhwp는 v0.5.0까지 단독 개발 → v1.0.0+ 부터 커뮤니티 개방 단계. 첫 기여자가 안전하고 환영받는다고 느끼는 것이 v1.0.0 비전의 일부. 외부 컨트리뷰터의 fork가 본 devel보다 뒤처졌을 때 단순 머지 시 "사실상 revert" 위험 큼.

**How to apply**:
- PR 처리 보고서 (`mydocs/pr/pr_{N}_report.md`)에 "rhwp 첫 PR" 표현 포함 (해당되는 경우)
- PR base가 본 devel보다 N commit 뒤처졌으면:
  - **즉시 close 권장 (옵션 C)**
  - 분리 PR + base 동기화 권장 명시
- 같은 이슈에 두 PR 동시 제출 시: 양 PR 동시 댓글로 협업 PR 권유 시도

**관련 사례**:
- PR #571 (@xogh3198): base skew 옵션 C 처리
- PR #578 vs #579: 양 컨트리뷰터 협업 권유 첫 시도
