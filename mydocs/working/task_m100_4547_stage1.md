# task_m100_4547 stage1 — 운동장 3부: 게이트 채점 (T14)

- 이슈: #4547 / 브랜치: task_m100_4547 (task_m100_4545 적층)
- 산출물: T14(티어 3) + 고정 정책 자산(gym/assets/) + README 3부 + 1호 32/32

## 폐루프 확장

- 채점 = `rhwp gate --policy(고정 자산) --keyring --anchor-log --deep` 한
  호출의 verdict:allow — 재계산 원칙이라 정책 자산은 골든 부패 없음, 실패 시
  violations 가 모자란 축을 명세(과제 자체가 디버깅 가이드).
- 함정 실측: 정책 자산을 gym/tasks/ 에 두면 채점기 글롭이 과제로 오인
  (KeyError) — gym/assets/ 분리 규약 확립.
- 1호 실주행: keygen→서명 캡슐→앵커 등재→gate allow → 전 14과제 32/32.
