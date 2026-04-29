---
name: 이슈 착수 시 즉시 assignee 지정 필수
description: GitHub 이슈를 내부에서 작업하기로 결정하면 브랜치 생성 전에 반드시 assignee 를 지정해야 함. 누락 시 외부 기여자가 같은 이슈를 독립적으로 집어갈 수 있음
type: feedback
originSessionId: 67d1cb8f-86d4-4672-b831-a8d028a1cfcf
---
이슈 착수 결정 → 브랜치 생성 전 **반드시** 해당 이슈에 assignee 를 지정. GitHub 이슈는 기본적으로 "누구든 가져갈 수 있는 오픈 타스크" 로 해석되기 때문.

**Why:** 2026-04-23 이슈 #259 (HY 폰트 매핑) 을 내부 작업하기로 하고 `local/task259` 브랜치에서 진행했으나 **이슈에 assignee 를 지정하지 않음**. 외부 기여자 @planet6897 이 같은 이슈를 보고 합리적으로 "assignee 없음 = 오픈 타스크" 로 판단하여 PR #264 로 5-Stage 완주 해결. 우리가 먼저 머지 → 기여자가 자신의 작업이 무의미해진 것을 확인하고 자진 close. 기여자의 몇 시간 노력이 증발. "열린 PR 확인 누락" 은 표면적 방어책이고, **진짜 원인은 이슈에 내부 작업 중임을 표시하지 않은 것**.

**How to apply:**
1. 이슈 착수 결정 즉시 (브랜치 생성 전):
   ```
   gh issue edit <num> --repo edwardkim/rhwp --add-assignee edwardkim
   ```
   또는 GitHub UI 에서 Assignees 설정. 이슈에 assignee 가 있으면 외부 기여자는 보통 "진행 중" 으로 인식해 중복 작업을 피함.

2. 내부 작업이 취소/연기되면 즉시 assignee 제거:
   ```
   gh issue edit <num> --remove-assignee edwardkim
   ```

3. 타스크 프로세스 순서 갱신:
   - 구: 이슈 확인 → 브랜치 → 할일 → 계획서 → 구현
   - 신: 이슈 확인 → **assign** → 브랜치 → 할일 → 계획서 → 구현

4. 기존 메모리 `feedback_check_open_prs_first.md` 는 "열린 PR 확인" 자체는 여전히 유효하지만, **assign 이 일차 방어선**. PR 확인은 이차 방어선.

**관련 사건**: `mydocs/orders/20260423.md` §13 (사건 재구성).
