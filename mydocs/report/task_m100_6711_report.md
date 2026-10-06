---
kind: report
status: final
canonical: mydocs/report/task_m100_6711_report.md
issue: 6711
last_verified: 2026-09-05
---

# #6711 mydocs 월별 아카이브 거버넌스·첫 전수 정리 최종 보고서

## 1. 최종 판정

**Stage 1~3의 네 순차 PR과 Stage 4 전수 감사는 계획한 보호 불변식을 충족했다.** 기준
commit의 직접 하위 Markdown 3,916개 중 cutoff 이전 후보 3,844개는 모두 archive 경로로
귀결됐고, 기준선의 9월 문서 72개는 root에 그대로 남았다. 메인테이너는 2026-09-05에 Stage 4
보고서를 승인했다. 제출·병합 절차가 남아 있으므로 Issue #6711은 아직 OPEN으로 유지한다.

- 기준선: `devel@3e06867e601b555141bd22ee8b5157f296db9238`
- 시간 경계: `Asia/Seoul`의 `2026-09-01T00:00:00+09:00`
- Stage 3-B merge: `d1831146587b1ac2346f9ed1216a64c2943a02f9`
- Stage 4 감사 기준: `devel@693c4b6b3edd1317934d6648449edcf47b0689e3`
- 판정: `qualified-monthly-archive-audit`, 메인테이너 승인 완료

이 감사에서 archive 경로는 완료 상태가 아니라 생성 월에 따른 보관 위치라는 거버넌스 결정을
그대로 적용했다. 문서 이동 자체를 근거로 관련 이슈나 PR의 상태를 바꾸지 않았다.

## 2. 후보 전수 귀결

기준선 tree와 Stage 3-B merge tree를 checkout 시각이나 filesystem mtime 없이 Git path로 직접
대조했다.

| 폴더 | 기준선 root | cutoff 이전 후보 | 기준선 9월 문서 보존 | 후보 잔여 |
| --- | ---: | ---: | ---: | ---: |
| `orders` | 67 | 63 | 4 | 0 |
| `plans` | 722 | 710 | 12 | 0 |
| `pr` | 120 | 119 | 1 | 0 |
| `report` | 722 | 713 | 9 | 0 |
| `working` | 2,285 | 2,239 | 46 | 0 |
| 합계 | 3,916 | 3,844 | 72 | 0 |

3,844개 후보의 귀결은 다음 식으로 닫힌다.

| 귀결 | 수 | 검산 근거 |
| --- | ---: | --- |
| byte-identical Git rename (`R100`) | 3,289 | 네 이동 commit의 rename similarity |
| 링크·canonical 정정을 포함한 Git rename | 543 | 같은 commit의 `R055`~`R099` 계보와 단계별 링크 대조 |
| 기존 archive와 동일한 root 중복 제거 | 12 | 기준선 source·archive SHA-256 12/12 일치 |
| 합계 | 3,844 | `3,289 + 543 + 12` |

일반·suffix 목적지를 후보별로 다시 계산한 결과 목적지가 없는 후보는 0개다. Git이 rename으로
추적한 3,832개와 해시로 증명한 동일본 12개를 합치면 계획한 후보 전부가 손실 없이 처리됐다.

## 3. 중복과 상이 충돌 원장

기준선에서 목적지 basename이 이미 존재한 문서는 16개였다. 그중 12개는 source와 기존 archive의
SHA-256이 같아 archive를 유지하고 root 중복만 제거했다.

- `plans`: `task_m100_1363.md`, `task_m100_1363_v2.md`
- `pr`: `pr_2331_maintainer_review.md`
- `report`: `task_m100_1363_report.md`
- `working`: `task_m100_1363_stage1.md`~`stage5.md`,
  `task_m100_1363_v2_stage1.md`~`stage3.md`

내용이 다른 4개는 기존 archive를 덮어쓰지 않고 source 최초 도입일과 content hash를 붙인 별도
경로로 보존했다. Stage 4에서 기존본과 suffix 보존본이 모두 존재함을 4/4 확인했다.

| 원 root | 기존 archive hash | root hash | root의 보존 경로 |
| --- | --- | --- | --- |
| `plans/task_m100_1880.md` | `3a524fdc…` | `fb8827e0…` | `plans/archives/task_m100_1880_archived_20260705_fb8827e.md` |
| `plans/task_m100_2214.md` | `7c01e1f9…` | `60d84809…` | `plans/archives/task_m100_2214_archived_20260712_60d8480.md` |
| `pr/pr_1844_review.md` | `0cf21421…` | `a72eb2d8…` | `pr/archives/pr_1844_review_archived_20260703_a72eb2d.md` |
| `pr/pr_2370_review.md` | `da7c3202…` | `100f4495…` | `pr/archives/pr_2370_review_archived_20260725_100f449.md` |

전체 SHA-256 값은 [Stage 1 원장](../working/task_m100_6711_stage1.md)에 보존돼 있다. 이동 뒤
상대 링크가 달라진 문서는 bytes 자체의 동일성을 주장하지 않고 Git rename 계보와 논리 link target
보존을 함께 증거로 사용했다.

## 4. 순차 PR과 Git 계보

| 단계 | PR | 후보 | rename / dedupe | code candidate 보수적 경로 | 최종 GitHub files | merge commit |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| Stage 2-A `orders/plans` | [#6713](https://github.com/edwardkim/rhwp/pull/6713) | 773 | 771 / 2 | 1,828 | 1,058 | `009e30fe1f` |
| Stage 2-B `pr/report` | [#6715](https://github.com/edwardkim/rhwp/pull/6715) | 832 | 830 / 2 | 1,735 | 906 | `c9cc1f7fc7` |
| Stage 3-A `working` | [#6726](https://github.com/edwardkim/rhwp/pull/6726) | 1,119 | 1,111 / 8 | 2,369 | 1,260 | `9e8e8bc567` |
| Stage 3-B `working` | [#6730](https://github.com/edwardkim/rhwp/pull/6730) | 1,120 | 1,120 / 0 | 2,347 | 1,229 | `d183114658` |

네 PR은 모두 `MERGED`이며 실제 GitHub file 수가 3,000개 제한 안에 있다. 보수적 경로 수는
rename을 전혀 인정하지 않는 code candidate 측정값이다. 최종 GitHub file 수에는 self-review 등
후행 증적 commit이 포함되므로 candidate의 rename-aware 수보다 1~2개 많다.

네 이동 commit `968a35305b`, `15d4a8f25a`, `f73a68590d`, `e44032cb9d`를 `-M`으로 다시
분석한 결과 rename은 각각 771, 830, 1,111, 1,120개이고 direct-root 삭제는 각각 2, 2, 8,
0개다. 삭제 12개는 모두 3절의 동일본 원장에 포함되며 예상 밖 삭제는 없다.

## 5. root 감소와 당월 문서 구분

Stage 3-B merge tree의 root에는 79개가 있었다. 이는 기준선의 9월 문서 72개, #6711이 만든
계획·단계 문서 5개, 동시에 병합된 #6717 문서 2개다. 따라서 물리 root 수는 `3,916 -> 79`,
3,837개·97.9826% 감소했다. 동시 작업 문서를 제외한 archive 대상 제거율은
`3,844 / 3,844 = 100%`이고, 기준선 root에서 archive 대상이 차지한 비율은 98.1614%다.

Stage 4 기준 최신 `devel`에는 #6697의 9월 `working` 문서가 하나 더 들어와 root가 80개다.
Git 추가 이력을 단일 scan으로 다시 판정한 결과 80/80개 모두 cutoff 이후 문서이고 판정 불가
경로는 0개다. 이 최종 보고서를 포함한 code candidate에서는 root가 81개이며, self-review 절차가
2026-09-05 오늘할일을 추가한 trailing tree에서는 82개다. 두 문서 모두 9월 문서이므로 다음 달
유지보수 전까지 archive 대상으로 보지 않는다.

## 6. 링크·metadata·범위 감사

Stage 3-B merge 시점의 전수 Markdown은 13,186개, 내부 링크 9,224개, 유효 8,671개,
historical broken 553개였다. Stage 4 최신 기준에서는 다음과 같다.

| 검사 | 결과 |
| --- | --- |
| canonical `python3 scripts/check_markdown_links.py` | 609개 문서, 오류 0 |
| 추적 Markdown 전수 | 13,195개 |
| 내부 상대 링크 | 9,250개 |
| 유효 / historical broken | 8,697 / 553 |
| metadata | 604개 검사, 기존 4개 문서의 16건만 재현 |
| cutoff 이전 direct Markdown | 0개 |
| Git 최초 도입일 판정 불가 | 0개 |
| 예상 archive 목적지 누락 | 0개 |
| Rust·Cargo·WASM·workflow 변경 | 0개 |

Stage 3-B 뒤 동시 작업으로 Markdown 9개와 유효 링크 26개가 늘었지만 historical broken은
553개로 변하지 않았다. 네 순차 PR의 단계 보고서도 각 base와 결과 사이의 신규 broken-link와
metadata 오류가 0개임을 기록한다. 따라서 기존 오류를 이 작업의 성공으로 숨기거나 새 오류를
추가하지 않았다.

Stage 3-A에서 발견된 Gym 문서의 완성 경로와 분할 `.join(...)` 소비자는 root redirect로
우회하지 않고 실제 archive 경로로 고쳤다. Stage 3-B의 font evidence generator와 hash contract도
같은 원칙으로 갱신했다. 이 실패 계보를 반영한 canonical 거버넌스는 Markdown 링크뿐 아니라
test·tool·generator·fixture의 완성 문자열과 분할 조립 경로를 모두 감사하도록 규정한다.

## 7. 다음 달 재실행 순서

매월 첫 유지보수 구간에 다음 순서로 반복한다.

1. 최신 `upstream/devel`과 clean worktree를 확인한다.
2. `Asia/Seoul` 당월 1일 00:00을 cutoff로 잡고 다섯 root의 직접 하위 `*.md`만 수집한다.
3. Git 최초 도입 commit의 author timestamp로 후보를 판정하고, 근거가 없으면 자동 이동하지 않고
   예외 원장에 남긴다.
4. 목적지 충돌을 SHA-256으로 나눠 동일본만 제거하고 상이본은 suffix 경로로 양쪽을 보존한다.
5. Markdown 상대 링크와 실행 경로 소비자를 함께 감사하고 old/new 논리 target을 보존한다.
6. rename-aware·보수적 경로 수를 측정해 PR file API 3,000개 아래로 순차 batch를 나눈다.
7. 각 batch를 앞선 merge가 포함된 최신 `devel`에서 만들고 canonical 링크, metadata delta,
   historical broken-link delta, cutoff 잔여를 검증한다.
8. 마지막 batch 뒤 전수 감사를 수행하고 이슈·PR 상태는 문서 경로와 별도로 판정한다.

세부 정본은 [문서·Git 워크플로](../manual/codex/docs_and_git_workflow.md#monthly-archive-governance)다.

## 8. 완료 조건과 남은 외부 절차

| 완료 조건 | 판정 |
| --- | --- |
| 월별 archive 정본·안내 문서 3개 | 충족 |
| 후보 3,844개 손실 없이 처리 | 충족 |
| 기준선 9월 문서 72개 보존 | 충족 |
| 상이 충돌 4개 양쪽 보존 | 충족 |
| 신규 내부 링크·metadata 오류 0 | 충족 |
| 각 PR이 file API 3,000개 미만 | 충족 |
| cutoff 이전 direct Markdown 0 | 충족 |
| 최종 보고서 메인테이너 승인 | 충족 |

보고서는 `final`로 전환됐다. 남은 절차는 Stage 4 문서 전용 commit을 만든 뒤 별도 승인에 따라
push·PR·exact-head CI·self-review·정상 merge를 수행하는 것이다. merge SHA의
post-merge 검증이 성공한 뒤에만 Issue #6711을 close하고 이번 task branch를 정리한다.

CodeQL alert #186의 최종 분류는 메인테이너가 입력한 `used in tests`를 유지한다. 해당 alert의
근거화·재발 방지는 [#6731](https://github.com/edwardkim/rhwp/issues/6731)의 별도 범위이며 #6711
종료를 막지 않는다.
