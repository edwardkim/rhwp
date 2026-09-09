---
kind: snapshot
status: active
canonical: mydocs/plans/task_m100_6711.md
issue: 6711
last_verified: 2026-09-04
---

# #6711 Stage 1 — 월별 아카이브 거버넌스와 이동 기준선

## 1. 결과

메인테이너의 기억에 의존하던 월별 정리를 세 문서의 정본 관계로 고정했다.

- canonical 절차: `mydocs/manual/codex/docs_and_git_workflow.md`
- 문서 구조·생명주기 경계: `mydocs/README.md`
- Hyper-Waterfall에서의 목적 설명: `mydocs/manual/hyper_waterfall_docs_guide.md`

역할별 merge·review 절차가 같은 달 문서를 먼저 archive로 옮기는 기존 방식은 유지한다. 새 월별
규칙은 그 절차에서 남은 문서가 다음 달에도 root에 누적되지 않게 하는 상한이다. `archives`는 완료
상태가 아니라 보관 위치이며, GitHub 이슈·PR 상태와 분리한다.

## 2. 판정 기준

- 기준 commit: `upstream/devel@3e06867e601b555141bd22ee8b5157f296db9238`
- cutoff: `2026-09-01T00:00:00+09:00`
- 대상: `mydocs/{orders,plans,pr,report,working}`의 직접 하위 `*.md`
- 제외: 각 root 아래의 디렉터리, assets, evidence, 다른 확장자
- 생성 시각: 해당 경로가 Git에 처음 추가된 commit의 author timestamp
- fallback: 추가 commit이 드러나지 않는 merge 유입 경로는 전체 이력에서 최초 존재 commit 사용
- fail-closed: Git 근거가 없거나 목적지 내용이 다르면 자동 추정·덮어쓰기를 하지 않음

filesystem mtime은 clone과 checkout으로 바뀌므로 판정 자료에서 제외했다. 후보별 old/new path는
Stage 2·3의 Git rename diff가 전수 원장이 되고, 최초 commit/time과 source SHA-256은 같은 입력
commit에서 다음 절차로 재계산할 수 있다.

## 3. 기준선 인벤토리

| 폴더 | root 문서 | cutoff 이전 | 9월 생성 |
| --- | ---: | ---: | ---: |
| `orders` | 67 | 63 | 4 |
| `plans` | 722 | 710 | 12 |
| `pr` | 120 | 119 | 1 |
| `report` | 722 | 713 | 9 |
| `working` | 2,285 | 2,239 | 46 |
| 합계 | 3,916 | 3,844 | 72 |

이 수치는 #6711 문서를 만들기 전 기준선이다. 이번 작업에서 생성되는 계획서·단계 보고서·최종
보고서는 모두 9월 생성 문서이므로 root에 남는 것이 정상이다. 완료 조건은 최종 root 총수를 72로
고정하는 것이 아니라 다음 두 조건이다.

1. 기준선의 9월 문서 72개가 보존된다.
2. cutoff 이전 생성 direct Markdown이 root에 0개 남는다.

## 4. 링크 기준선

- 이동 후보: 3,844개
- 이동 후보로 들어오는 Markdown 링크: 999개, source 문서 491개
- 이동 후보 문서 안의 로컬 상대 링크: 2,404개
- 현재 존재하는 target: 2,294개
- 현재 이미 존재하지 않는 target: 110개
- 다섯 대상 폴더를 archives까지 전수 검사한 historical 오류: 526개
- canonical 기본 검사: 609개 문서, 깨진 내부 상대 링크 0개

과거 오류 526개를 이번 대량 이동과 섞어 고치지 않는다. 이동 전에 유효했던 링크가 새로 깨지는
경우와 기준선에 없던 오류만 회귀로 판정한다.

## 5. 목적지 충돌 원장

### 5.1 byte-identical — root 중복만 제거

| root 경로 | SHA-256 |
| --- | --- |
| `plans/task_m100_1363.md` | `7e6901eb791f0474465334996795040968aefea380426b3cfc26297256a4fe12` |
| `plans/task_m100_1363_v2.md` | `e39d1b812dde734726dd4d3a2444b0dd1ab8700101b6486f9a77caa4224eb659` |
| `pr/pr_2331_maintainer_review.md` | `085ea428cd97dc3646367e7df33e12ea3e5a21b3c7c6e673aab05337effcb8a2` |
| `report/task_m100_1363_report.md` | `d2ed6bfa7e3ef29628ef0acb4599d352be578a3bf6870abfb40ea73a7d3fc2c9` |
| `working/task_m100_1363_stage1.md` | `ac63005caa48ad44a780496a4ec852e3c76028088b2418934550c88d51e4d1ea` |
| `working/task_m100_1363_stage2.md` | `0802debd5e85c0305446b053b6f9ccbbd9e3b39423bdd4104402ddb165194f73` |
| `working/task_m100_1363_stage3.md` | `b1e0a5c3212845538cf990444d3f2f61c4d17c7f22a425c2f8854f84308ce612` |
| `working/task_m100_1363_stage4.md` | `fc0d214de9a098b7fa69dbe7a6fc7bd631712b87660291f6d37f3d1010f39181` |
| `working/task_m100_1363_stage5.md` | `5b23a03e2abd9f44b8fe0571d8417ea2ee68e945fab4a7dfb2fb45c0a83cf346` |
| `working/task_m100_1363_v2_stage1.md` | `267ebb54d2387237e03b707503048c0e16931f8dedf7322fc1b6120ca56fc01a` |
| `working/task_m100_1363_v2_stage2.md` | `71fd93952903b9b5c49c08d80d220fd7e3badf140ac4291750e1348c70d91958` |
| `working/task_m100_1363_v2_stage3.md` | `923f630100bf051978aad40471cc22a100f40ddecb4f7fdffb4ec806327f7947` |

각 archive 목적지의 SHA-256이 표의 root와 같은 것을 다시 확인했다. Stage 2·3에서는 목적지를
유지하고 root duplicate만 제거한다.

### 5.2 divergent — 양쪽 보존

| root 경로 | root SHA-256 | 기존 archive SHA-256 | 새 목적지 |
| --- | --- | --- | --- |
| `plans/task_m100_1880.md` | `fb8827e0…` | `3a524fdc…` | `plans/archives/task_m100_1880_archived_20260705_fb8827e.md` |
| `plans/task_m100_2214.md` | `60d84809…` | `7c01e1f9…` | `plans/archives/task_m100_2214_archived_20260712_60d8480.md` |
| `pr/pr_1844_review.md` | `a72eb2d8…` | `0cf21421…` | `pr/archives/pr_1844_review_archived_20260703_a72eb2d.md` |
| `pr/pr_2370_review.md` | `100f4495…` | `da7c3202…` | `pr/archives/pr_2370_review_archived_20260725_100f449.md` |

suffix의 날짜는 root 문서의 최초 Git 도입일, 7자는 root blob 내용 SHA-256의 앞 7자리다. 기존
archive 문서의 이름과 bytes는 바꾸지 않는다.

## 6. 열린 이슈와의 교차 확인

이전 달 문서를 가진 다음 이슈는 2026-09-04 실시간 조회에서 여전히 열려 있다.

- #536 — `working`
- #3790 — `plans`, `working`
- #5447 — `plans`, `report`, `working`
- #5959 — `working`
- #6243 — `plans`

이 문서들도 생성 월 기준으로 archive로 이동한다. 이는 이슈 close 조건 충족이나 작업 완료를 뜻하지
않으며, 후속 작업은 이동된 경로에서 이어간다.

## 7. 문서 변경과 검증

Stage 1에서 변경한 거버넌스 문서는 다음과 같다.

- `mydocs/manual/codex/docs_and_git_workflow.md`
- `mydocs/README.md`
- `mydocs/manual/hyper_waterfall_docs_guide.md`

검증 기준:

- `git diff --check`: 통과
- `python3 scripts/check_markdown_links.py`: 609개 문서, 오류 0개
- `python3 scripts/check_document_metadata.py`: 기존 오류 16개 재현, Stage 1 신규 오류 0개
- Rust source, Cargo, WASM, workflow 변경: 없음

## 8. 다음 단계

Stage 2는 governance와 `orders/plans/pr/report` 후보 1,605개를 첫 PR 단위로 처리한다. 이동·링크
갱신을 적용한 뒤 GitHub의 rename 판정에 기대지 않는 보수적 파일 수가 3,000개에 근접하면 push
전에 batch를 더 작게 나눈다. 원격 push와 PR 생성은 별도 승인을 받는다.
