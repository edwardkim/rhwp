# Task M100 #2313 Stage 5 완료 보고 — metrics 결산과 종료 준비

- 이슈: #2313
- 상위 추적: #2022
- 브랜치: `task2313-legacy-web-removal`
- 기준 브랜치: `upstream/devel`
- upstream 기준 commit: `4817f2c1b5c7fde57a2d4d9b2ec115b02c437d20`
- metrics 대상 commit: `060c45aa2067a7a3112a9a1825f9524ec1b0d8d9`
- Stage 4 commit: `060c45aa`
- 완료일: 2026-07-17
- 상태: local Stage 5 완료, push와 GitHub 게시 승인 대기

## 1. Stage 5 판정

최종 clean commit에서 post-removal metrics를 다시 생성해 Stage 1의 예상값과 정확히 일치함을 확인했다.
same-base 함수 diff 149건은 모두 제거된 `web/` 함수이며 non-legacy stable function diff는 0건이다.

따라서 Total CC 감소는 current Studio 코드의 구조 개선이 아니라 사용하지 않는 legacy 모집단 삭제 결과다.
#2124 공식 snapshot 대비 현재 수치 증가는 snapshot 이후 upstream 기능 누적과 legacy 삭제가 함께 반영된
관측치이며 #2313 직접 성과로 귀속하지 않는다.

로컬 구현·검증·결산 문서는 완료됐지만 PR review, CI, merge와 issue close는 남아 있다. 작업지시자 승인 전
push, PR 생성, GitHub 본문·코멘트 편집과 issue close를 수행하지 않는다.

## 2. metrics provenance

| 항목 | 결과 |
|------|------|
| schema | v2 |
| measured commit | `060c45aa2067a7a3112a9a1825f9524ec1b0d8d9` |
| upstream commit | `4817f2c1b5c7fde57a2d4d9b2ec115b02c437d20` |
| Git working tree | clean |
| measured source | clean |
| included files | 214 |
| test/e2e 모집단 | 제외 |
| official snapshot diff | 0 |

| artifact | SHA-256 |
|----------|---------|
| metrics script | `5d100c90f47671240f463b0a48fe61d34eb8aedbf8c22bbe333f31241f11d087` |
| metrics package lock | `a7ae3c1a0f3c94700cfe29dc9c363657cb1f675c988446d5dc81b7eeecace5dd` |
| pre-removal JSON | `f81a720f455b82d57809d1fcfc7de1272aa78c27f68d8cfbbfd46a68095e5aa5` |
| final post-removal JSON | `7f6f0a45a1961ee7a6fa502b4e6b67264cc3c7e617d3700204d17755d000b54a` |
| post-vs-official JSON | `d9d47d4fba44f0bd32e66f65dd5ed76131ba4cb829e061cc4c7d0e3d7570154f` |

metrics JSON과 summary는 ignored `output/frontend-metrics/task2313/`에 보존하고 commit하지 않는다. review 가능한
수치, 기준 commit과 hash는 이 보고서와 최종 보고서에 기록한다.

## 3. same-base 직접 delta

Stage 1 pre-removal snapshot은 계획 문서 commit `4f8dedac`에서 생성했다. 이후 current source에는 `/web` 삭제
외 함수 변경이 없으므로 이 비교가 #2313의 직접 structural delta다.

| 지표 | pre | post | delta |
|------|----:|-----:|------:|
| included files | 224 | 214 | -10 |
| reported CC functions | 2,516 | 2,367 | -149 |
| Total CC | 13,118 | 12,290 | -828 |
| global Top 20 합 | 2,700 | 2,662 | -38 |
| CC>25 개수 / 합 | 73 / 4,479 | 69 / 4,272 | -4 / -207 |
| CC>100 개수 / 합 | 7 / 1,839 | 7 / 1,839 | 0 / 0 |
| Max CC | 453 | 453 | 0 |

- function diff: 149건, 모두 `removed`
- `web/` function diff: 149건
- non-legacy stable function diff: 0건
- Stage 1 expected와 불일치: 0

legacy group 내부 Top 20 합 442와 global Top 20 감소 38은 다른 모집단이다. 삭제된 group Top 20을 global
개선량으로 사용하지 않고 전체 순위를 다시 계산한 38만 기록한다.

## 4. 삭제된 legacy 모집단

| 지표 | 제거 전 legacy `/web` |
|------|------------------------:|
| 측정 파일 | 10 |
| lines / code lines | 6,592 / 5,795 |
| AST functions | 251 |
| reported CC functions | 149 |
| Total CC | 828 |
| group Top 20 합 | 442 |
| CC>25 개수 / 합 | 4 / 207 |
| Max CC | 86 |

tracked 제거는 앱 측정 파일 10개뿐 아니라 HTML/CSS, Python server, cert/key, generated WASM declaration과
font compatibility link를 포함한 18 entries다. metrics 감소는 이 dead ownership 제거를 정량화할 뿐 current
모듈 품질이 개선됐다는 SOLID 점수로 해석하지 않는다.

## 5. #2124 official 대비 현재 누적치

| 지표 | #2124 official | current | delta |
|------|---------------:|--------:|------:|
| reported CC functions | 2,282 | 2,367 | +85 |
| Total CC | 11,805 | 12,290 | +485 |
| global Top 20 합 | 2,581 | 2,662 | +81 |
| CC>25 개수 / 합 | 62 / 3,932 | 69 / 4,272 | +7 / +340 |
| CC>100 개수 / 합 | 6 / 1,732 | 7 / 1,839 | +1 / +107 |
| Max CC | 453 | 453 | 0 |

official baseline commit은 `3077f96d`이며 current 비교의 function diff는 513건이다. 이 비교에는 #2124 이후
Studio renderer, SVG, font, embed와 기타 upstream 변경뿐 아니라 #2313 legacy 삭제가 함께 포함된다. 따라서
same-base 비교 없이 +485 또는 -828만 단독으로 리팩터링 성패 판정에 사용하지 않는다.

## 6. maintainer 산식 교훈 반영

#1904 결산과 #2130 산식 보정에서 확인된 복잡도 통이동 문제를 막기 위해 다음 지표를 함께 사용했다.

- Total CC와 global Top 20 합
- CC>25/100 개수와 합
- Max CC
- stable function id별 added/removed/changed diff
- test/e2e 제외, clean/dirty metadata와 tool hash

Max CC는 453으로 그대로이고 Total CC 감소는 삭제 함수 149건의 합과 정확히 일치한다. 복잡도를 다른 current
함수로 옮긴 delta는 0이다. SOLID guide는 hotspot의 책임·의존 방향을 검토하는 정성 anchor로만 사용하며,
서로 다른 frontend surface를 하나의 `54/100` 같은 총점으로 합산하지 않는다.

## 7. Phase B 재평가 입력

current Total CC 12,290 중 Studio runtime은 10,751로 87.5%다. CC>25 함수도 69개 중 58개이며 global Top
20 합 2,662가 모두 Studio에서 나온다.

| 후보 | lines | Total CC | CC>25 | Max CC | 해석 |
|------|------:|---------:|------:|-------:|------|
| `diff-engine.ts` | 3,105 | 897 | 8 | 88 | 총량·고복잡도 함수 수가 최대, compare semantics 회귀 위험 큼 |
| `input-handler-mouse.ts` | 1,968 | 748 | 4 | 453 | 최대 함수 999 LOC, pointer/selection 계약 위험 큼 |
| `input-handler-keyboard.ts` | 1,942 | 729 | 5 | 444 | 최대 함수 909 LOC, 편집·단축키 계약 위험 큼 |
| `input-handler.ts` | 4,565 | 677 | 1 | 27 | 파일 크기는 최대이나 함수 최대 CC는 낮아 경계 재편 문제 |
| `picture-props-dialog.ts` | 2,825 | 647 | 2 | 348 | dialog로 격리 가능하지만 undo·serialize characterization 필요 |
| `wasm-bridge.ts` | 2,334 | 370 | 0 | 12 | 크기는 크지만 CC 우선순위는 낮고 public bridge 계약이 강함 |

이 수치는 하나의 자동 순위를 만들지 않는다. 다음 실행 이슈는 위험 낮은 순과 복잡도 높은 순을 함께 비교하고,
선택한 surface의 characterization gate와 public/security contract를 먼저 고정해야 한다. #2313 PR review와 merge
전에는 Phase B 구현 이슈를 생성하지 않는다.

## 8. 완료 기준 대조

| #2313 완료 기준 | 결과 |
|-----------------|------|
| root tracked `web/` 제거 | 완료, 18 entries |
| current production/build/package 직접 소비자 | 0 |
| current 한/영 manual의 `/web` 지원 안내 | 0 |
| CI detector, metrics와 font contract 현행화 | 완료 |
| #2124 snapshot과 historical evidence 보존 | 완료, diff 0 |
| Studio와 frontend package/font gate | PASS |
| legacy 제외 Phase B 모집단 | 생성 완료 |

이 표는 로컬 산출물 완료를 뜻한다. PR review·merge와 issue close가 끝났다는 의미는 아니다.

## 9. GitHub 경계와 다음 순서

`mydocs/report/task_m100_2313_pr_draft.md`에 다음 초안을 작성했다.

1. PR 제목과 본문
2. #2313 PR 생성 후 진행 코멘트
3. maintainer review 요청 코멘트
4. merge 후 #2313 완료 근거·close 코멘트
5. merge 후 #2022 body와 진행 코멘트 갱신안

다음 단계는 작업지시자에게 초안을 제시하고 승인을 받는 것이다. 승인 뒤에도 PR merge 전에는 #2313을
close하거나 #2022의 checkbox를 완료로 바꾸지 않는다.
