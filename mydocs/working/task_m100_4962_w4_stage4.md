---
kind: working-note
status: completed
issue: 4962
stage: W4-4
last_verified: 2026-08-22
---

# Task M100 #4962 W4 Stage 4 — 공개 ranking·W5 인계

- **이슈**: [#4962](https://github.com/edwardkim/rhwp/issues/4962)
- **계획**: [`task_m100_4962_w4.md`](../plans/task_m100_4962_w4.md)
- **선행 단계**: [`task_m100_4962_w4_stage3.md`](task_m100_4962_w4_stage3.md)
- **최종 보고**: [`task_m100_4962_report.md`](../report/task_m100_4962_report.md)
- **단계 상태**: W4-4 산출물 완료, 2026-08-22 W4 최종 승인

## 1. 결론

W4-3 local evidence ranking을 재계산 검증한 뒤 공개 가능한 351개 face ranking과 W5 1차 action queue를
생성했다. 1차 queue는 임의의 상위 N개가 아니라 base cumulative risk band A+B 전체 17개다. 이 후보들은
위험 문자 1,562,076자와 base risk mass 7,015,182를 설명한다. 각각 전체의 757,652 ppm과 810,374 ppm다.

17개 모두에 #4963이 요구한 exact 설치, exact 제거, 문서 `substFont`만 제공, 직접 근거가 있는 official
successor만 설치, 관련 font 모두 미설치의 다섯 질문을 생성했다. Oracle Profile schema와 controlled
ladder 자체는 W5 범위이므로 시작하지 않았다.

## 2. 공개 projection 경계

`scripts/font_typesetting_risk_publish.mjs`는 다음을 fail-closed로 확인한다.

1. 입력 kind·issue와 W4-3 canonical body SHA-256 `671e7174…c91d06`
2. W4-3의 unsupported promotion·identity guess·cross-band 이동 0
3. base/action rank가 1부터 351까지 완전한 permutation인지
4. queue가 A·B 밖으로 나가지 않았는지
5. 공개 결과에 corpus identity·path·raw row·문자 trace가 없는지

공개 asset에는 face 이름, aggregate count·ppm, category, compressed fixed-context proxy, stored/fresh mass,
민감도 rank·band, exact source 상태, backend 상태, historical supply 상태, evidence anchor와 W5 질문만 있다.
survey의 `document_count`, URL·note와 로컬 font 절대 경로는 넣지 않았다.

## 3. W5 선택 규칙

- **Queue**: base band A+B, 17개, base rank 1–17
- **Reserve**: band C+D, 334개, 삭제하지 않고 전체 ranking에 유지
- **같은 band 내부 순서**: exact-source-verified → government/legal core → backend divergence → base rank
- **promotion 금지**: source available, supply status, unknown relation만으로는 앞세우지 않음
- **cross-band 금지**: action rank가 각 base band의 원래 rank 구간을 벗어나면 실패

Queue의 exact source 상태는 `available` 2개, `unknown` 15개이며 verified bytes 후보는 0개다. 이는 W5가
먼저 exact source를 찾아야 하는 후보가 많다는 뜻이지 비슷한 font를 successor로 추정할 허가가 아니다.

## 4. 반복 결정성

같은 W4-3 r1을 공개 generator에 두 번 입력했다.

| 항목 | 결과 |
| --- | --- |
| 파일 bytes | 892,640 |
| 파일 mode | `0644` |
| 파일 SHA-256 | `6947e9e8a6c67a60a54b04dc6f1abf75e3cc66d9096a978d301ba2c10bb4ee3a` |
| canonical output hash | `95e7a41d1ed92a60cb66e1705b038c3e9086829b3c8aee48af57e8c2da111a68` |
| 1차/2차 bytes | exact |

산출물은
[`font_typesetting_risk_rank.json`](../report/assets/task_m100_4962/font_typesetting_risk_rank.json)이다.

## 5. GitHub read-only 교차 확인

2026-08-22에 `gh issue view`로 #4960·#4962·#4963을 다시 읽었다. 세 이슈는 모두 OPEN이며 다음 인계가
본문 범위와 일치한다.

- #4960: W3+W4 체크 후보를 준비하되 최종 승인·통합 전에는 체크하지 않음
- #4962: coverage 결정성, stored/fresh 분리, 위험 순위, W5 후보·질문을 모두 준비함
- #4963: 17개 queue와 다섯 controlled-ladder 질문을 입력 후보로 준비함

Issue comment·체크박스·상태를 수정하지 않았다.

## 6. 보호 불변식과 다음 승인

- private 10k 재실행 없음
- W3·W4 local artifact와 font bytes 게시 없음
- exact identity·successor·missing-font relation 신규 확정 없음
- metric DB·fallback·paint·font asset·renderer 변경 없음
- W5 Oracle Profile schema·controlled ladder 미착수
- 원격 push·PR·GitHub write 없음

2026-08-22 메인테이너가 최종 보고와 W5 action queue를 승인해 W4 종료 게이트를 통과했다. 다음은
#4962 통합 준비이며, #4960·#4962 GitHub 본문 갱신, #4963 착수, 원격 push와 PR은 각각 해당 절차의
별도 승인 뒤 진행한다.
