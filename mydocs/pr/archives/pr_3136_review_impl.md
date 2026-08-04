# PR #3136 통합 적용 기록 — 브라우저 인쇄/PDF 저장

## 적용

| 항목 | 내용 |
| --- | --- |
| 기준 / 브랜치 | `upstream/devel@1b5950a95` / `integrate/postmelee-20260724` |
| 누적 위치 | #3125, #3130 뒤의 3/3 |
| Draft 포함 근거 | 작업지시자의 명시 지시 |
| 원 PR 상태 | Draft + CONFLICTING/DIRTY + maintainer_can_modify=false |
| 적용 SHA | `c63abac, 1028451, 3536174, aafe29d, bb75241, c87cdf9, 6044550, f058343, c06a2f8, df30ade, fc1f947, 0e76a25, c0dc1f6` |

## 보정 내용

오늘할일 문서 충돌은 다른 작업의 #2308 기록을 유지해 합쳤다. Studio file command 충돌에서는
deferred pagination flush와 print preview 진입을 모두 보존했고, paint builder 충돌에서는 기존
editor regression과 print profile regression을 분리했다. 이러한 해소는 원 contributor branch로
push하지 않고 통합 후보 브랜치의 메인터너 보정으로만 수행했다.

## 후속 순서

1. 문서·asset 추가는 코드 검증 뒤의 운영 기록이므로 full local suite를 재실행하지 않는다.
2. 통합 PR을 만들고 최신 head CI를 모니터링한다. Draft 원 PR의 기존 CI가 없으므로 통합 PR CI가
   실제 수용 게이트다.
3. merge 뒤 #3126 open 상태를 확인한다. native dialog/후속 browser matrix가 남아 있으면 이슈와
   원 PR을 자동 close하지 않는다.
4. 감사 및 supersede 안내 코멘트는 merge 뒤 문안을 제시하고 별도 승인받아 게시한다.
