---
kind: investigation
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-14
---

# PR #7115 메인터너 보정·후속 단계

[개별 review](pr_7115_review.md)의 현재 판정은 **메인터너 보정 후 수용 가능**이다. 원 head의 보류 사유는 명시한 통합 보정 코드에서 해소했다.

## 적용 이력

- branch: `review/planet6897-20260914`
- rebase 당시 기준: `upstream/devel` = local `devel` = `037e4906a93e99896daa145a5ee5517824bfeaf4`
- source: `fa94b52a07de08af9d4320ab911caefa19e323dd` → rebase 후 `0fb0d9345963471fe5bbceee0944ef854c5008aa` (`-x`·원 저자 유지)
- 메인터너 보정: `b80db1a21 + 2f59c8937 (초기 5d35b37cc의 전역 공백 보정 철회)`
- 최종 검증 코드: `2f59c89373f497068f9a0bcb2c22730ec7dc7e51`
- 입력/PDF: 기존 Git/LFS 파일 재사용; 보존 commit `ab3184254`

## 해결한 원인과 확인

이전 review의 유효 저장 LineSeg 가설을 정정한다. 실제 원본은 section 0, 표 host pi=936, cell=3, p=9/13의 LineSeg가 없는 NO_LS 문단이다. 들여쓴 다줄 셀의 반각 공백 채움 규칙이 양쪽정렬 분배가 없는 마지막 한 줄에도 적용돼 문장 끝을 밀었다.

동일한 frame 채움기로 현재 행 시작부터 글꼴 공백 후보를 계산하고, 남은 문단이 그 구간에서 끝나는 경우에만 후보 행·폭·높이를 함께 게시한다. 한 행 문단과 다줄 문단의 마지막 행에 공통 적용하며, 중간 행은 기존 반각 채움을 유지한다. 토큰 경계 재생은 이진 탐색으로 찾고, 커닝이 준비된 문단은 기존 폭 소유 경로를 유지한다. 원본 Justify와 NO_LS 들여쓰기 계약을 사용하며 파일명·쪽수·문자열 조건을 생산 코드에 넣지 않았다. 처음 시도한 전역 min(반각, 글꼴 공백) 보정은 49쪽을 과소 측정해 철회했다.

독립 한컴 PDF p108의 9호와 마지막 3호가 각각 완전한 한 줄로 p108에 남는다. p75의 부대시설 조항은 한컴과 같은 3행 및 각 행의 전체 문자열을 검사한다. 그 마지막 다)가 별도 행으로 밀리지 않아 p76~77에 원 PR이 더한 한 줄을 제거한다. p49 대조군은 자산관리회사 조항의 줄끝 운 / 법 / 인과 총 3행을 유지한다. 전체 문서 페이지 수만으로 통과시키지 않고 Visual Sweep에서 해당 문장과 후속 내용을 직접 확인한다.

76~77쪽은 원 PR 이전 base의 흐름으로 복구한 것이며 한컴 전체 일치가 아니다. p77의 이전 조항 2행 잔존, p109 상단 여백, 기존 글꼴·좌표 차이 및 76076 p81의 사고/사고를 차이는 별도 잔여다. 이 차이를 숨기기 위해 문서별 분기나 baseline 갱신을 추가하지 않았다.

## 단계

| 단계 | 상태 |
| --- | --- |
| upstream/devel 동기화·rebase | 완료; 원 18 commit 재적용, 충돌 없음, 이전 branch 백업 보존 |
| 세 보류 항목 보정 | #7104 그림 배제/빈 lane, #7113 재귀·quoted literal, #7115 마지막 가시 행 공백 |
| 로컬 검증·Visual Sweep | 보류 사유 해소 — 원 PR의 부분 개선 범위 수용; 정확한 결과·입력·PNG는 개별 review에 기록 |
| 통합 PR | #7138 생성 완료; upstream `codex/planet6897-integration-20260914` → devel, owner 리뷰 요청 없음 |
| CI·trailing 기록 | 통합 code candidate CI 후 review·오늘할일 trailing 절차 적용. 이번 로컬 검증을 원격 CI로 표기하지 않음 |
| merge·원 PR/issue 후속 | 최종 SHA·CI·MERGEABLE/CLEAN 및 승인 범위 확인 후 수행. 부분 이슈를 완료로 닫지 않음 |
| devel·정리 | merge 후 동기화하고 실행 중 Rust/Cargo가 없는지 확인한 뒤 소유한 review target만 정리 |

원 PR에 게시할 comment에는 반영 source SHA, 메인터너 보정, 통합 merge SHA, Visual Sweep 직접 링크와
고정된 대표 PNG, 남은 이슈 범위를 포함한다. 현재 이 문서는 게시 완료 기록이 아니다.
다른 작업의 변경을 reset/clean으로 버리지 않는다. [후속 처리 정본](../../manual/pr_review/post_merge.md)을 따른다.

## PR #7138 code CI 완료 후 trailing 단계

- 통합 code candidate `a7898ff72a0b22e1e4071b682616a26dc82fd801`의 Full CI·CodeQL·Render Diff·Adapter·Proptest·Policy 성공을 확인했다.
- [개별 review](pr_7115_review.md)의 최신 절이 이전 조사 단계·SHA보다 우선한다. 원래 조사·검증 이력은 보존한다.
- 원 PR별 review와 [오늘할일](../../orders/20260914.md)을 같은 통합 PR의 single-parent 문서 trailing commit으로 반영한다. 코드 변경·base merge/rebase는 하지 않는다.
- push 전 최신 base/head merge-tree, 공백·문서 링크·기존 오늘할일 보존을 검사하고 push 뒤 exact trailing head의 required CI와 재사용 결과를 확인한다.
- 병합·원 PR 댓글/close·이슈 잔여 확인·devel 동기화·duration refresh·소유 산출물 정리는 승인된 merge 이후 단계다.
