# planet6897 열린 PR 통합 검토 - Stage 5

## 목적

최신 `upstream/devel` 기준으로 체리픽 통합 브랜치를 rebase하고, collaborator-mediated 통합 PR의
최초 remote push 전에 오늘할일을 갱신한다.

## 기준

- rebase 기준: `upstream/devel` `1658d0bb2`
- 통합 대상: #2662, #2663, #2664, #2665, #2666, #2669, #2671, #2706
- 제외 대상: #2714

## 수행

1. 16개 로컬 커밋을 `1658d0bb2` 위로 충돌 없이 rebase했다.
2. `mydocs/orders/20260721.md`에 통합 PR 준비 사실, 검증 범위, merge 전 조건을 기록한다.
3. 최초 remote push 직전에 working tree와 오늘할일 충돌 여부를 다시 확인한 뒤, 원본 저장소의
   `task_m100_planet6897_batch_20260721` head branch로 push한다.

## 완료 조건

- 오늘할일이 최초 PR diff에 포함된다.
- 통합 PR은 원 PR 8건의 review 문서·자산·체리픽 코드와 분리된 최종 준비 커밋을 가진다.
- #2714는 체리픽과 통합 PR 범위에 포함하지 않는다.
