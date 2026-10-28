---
kind: pr_review
status: approved
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-04
pr: 6713
issue: 6711
author: edwardkim
---

# PR #6713 self-review — mydocs 월별 archive 거버넌스와 orders/plans 정리

## 결론

**승인.** PR #6713은 `mydocs/orders`, `plans`, `pr`, `report`, `working`의 직접 하위
Markdown을 당월 작업 집합으로 유지하는 월별 archive 거버넌스를 정본화하고, 첫 적용으로
`orders/plans`의 cutoff 이전 후보 773개를 처리한다.

code candidate `968a35305b9b1b4ad209e64d97655df411754bf4`를 독립 재검토한 결과 771개는
rename, 2개는 byte-identical archive가 이미 있는 중복 제거이며, 서로 다른 동명 문서 2개는
suffix 경로로 모두 보존됐다. 예상하지 않은 삭제·경로 범위 누출·신규 링크 또는 metadata 오류는
발견하지 않았다.

이 문서의 `승인`은 작성자 self-review 판정이다. 자기 PR이므로 reviewer 지정이나 GitHub approve
review event를 만들지 않는다. 이 review와 오늘할일만 추가한 trailing head의 GitHub Actions,
최신 `devel` 포함 관계와 mergeability를 다시 확인해야 하며, remote push와 merge는 각각 사용자
승인 게이트를 따른다.

## 라우팅과 메타데이터

- 기본 경로: `collaborator_self_merge.md`
- 보조 경로: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`,
  `rework_and_exceptions.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`와 위 자식 문서
- `review_impl`은 추가하지 않는다. 승인된 [수행계획](../../plans/task_m100_6711.md)과
  [Stage 1](../../working/task_m100_6711_stage1.md),
  [Stage 2-A](../../working/task_m100_6711_stage2.md)가 계획·원장·구현 계보를 고정한다.

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#6713](https://github.com/edwardkim/rhwp/pull/6713) / @edwardkim |
| 관련 이슈 | [#6711](https://github.com/edwardkim/rhwp/issues/6711) (`Refs #6711`) |
| base | `devel@3e06867e601b555141bd22ee8b5157f296db9238` |
| code candidate | `968a35305b9b1b4ad209e64d97655df411754bf4` |
| 규모 | 1,057 files, `+1,589/-1,114`, 3 commits |
| 작성 시점 GitHub 상태 | Open, 비 Draft, `MERGEABLE`; code candidate CI 진행 중 |
| reviewer | self PR이므로 지정하지 않음 |

1,000줄과 1,000개 파일을 모두 넘으므로 대형 PR 경로를 적용했다. 변경의 대부분은 동일 역할
폴더 안의 경로 이동과 그 이동으로 바뀐 상대 링크다. GitHub rename 판정에 기대지 않은 old/new
경로 합계도 1,828개로 PR files API 상한 3,000개보다 작다. 후속 `pr/report`와 `working`은 이 PR에
추가하지 않고 #6711의 순차 PR로 분리한다.

## 거버넌스 검토

- canonical 정본은 `mydocs/manual/codex/docs_and_git_workflow.md` 한 곳이다.
  `mydocs/README.md`는 정보구조 재분류와 월별 작업 증적 정리를 구분하고,
  `hyper_waterfall_docs_guide.md`는 목적과 탐색 규칙만 설명한 뒤 정본을 연결한다.
- archive 경로는 완료·폐기 상태가 아니다. 열린 이슈·PR의 이전 달 문서도 archive에서 계속
  갱신하며, 경로 이동을 close 근거로 사용하지 않는다.
- cutoff는 `Asia/Seoul` 당월 1일 00:00이고 생성일은 filesystem mtime이 아니라 경로의 Git 최초
  도입 commit author timestamp다. 근거가 없으면 추정하지 않는다.
- 대상은 다섯 폴더의 직접 하위 `*.md`뿐이다. assets·evidence·주제별 중첩 디렉터리는 자동 이동하지
  않는다.
- 목적지가 같고 SHA-256도 같을 때만 root 중복을 제거한다. 내용이 다르면 덮어쓰지 않고 최초
  도입일과 내용 hash를 넣은 suffix 경로로 양쪽을 보존한다.
- 링크는 단순 문자열 치환이 아니라 이전 source에서 해석한 실제 target을 새 source 기준으로 다시
  상대화한다. root를 다시 채우는 redirect stub은 만들지 않는다.
- CI가 GitHub PR files API 목록으로 영향도를 판정하는 동안 보수적 변경 경로가 3,000개보다 작도록
  순차 PR로 나눈다. batch 때문에 자식 이슈를 추가하지 않는다.

## 이동·손실 방지 재검토

`git diff --name-status --find-renames upstream/devel...HEAD`를 다시 집계한 결과는 다음과 같다.

| 상태 | 수 | 판정 |
| --- | ---: | --- |
| 수정 | 281 | 링크·canonical·거버넌스·증적 갱신 |
| rename | 771 | `orders/plans`에서 같은 역할의 `archives/`로 이동 |
| 삭제 | 2 | 기존 archive와 SHA-256이 같은 root 중복만 제거 |
| 추가 | 3 | #6711 계획서와 Stage 보고서 2개 |
| 예상 밖 범위 | 0 | 허용한 두 외부 README 링크 소비자만 `mydocs` 밖에서 변경 |

삭제된 두 문서는 다음처럼 `upstream/devel`의 root bytes와 현재 archive bytes가 동일하다.

| 문서 | SHA-256 |
| --- | --- |
| `plans/task_m100_1363.md` | `7e6901eb791f0474465334996795040968aefea380426b3cfc26297256a4fe12` |
| `plans/task_m100_1363_v2.md` | `e39d1b812dde734726dd4d3a2444b0dd1ab8700101b6486f9a77caa4224eb659` |

내용이 다른 `task_m100_1880.md`, `task_m100_2214.md`는 기존 archive를 유지하면서 각각
`task_m100_1880_archived_20260705_fb8827e.md`,
`task_m100_2214_archived_20260712_60d8480.md`로 별도 보존했다. 따라서 동명 충돌로 덮어쓴
문서는 없다.

root에는 `orders` 4개와 `plans` 13개가 남았고 모두 9월 도입 문서다. `plans`의 13개는 작업 전
기준선 12개와 #6711 계획서 1개다. cutoff 이전 direct Markdown 잔존은 0개다.

## 링크·metadata·변경 범위 검증

| 검증 | 결과 |
| --- | --- |
| `git diff --check upstream/devel...HEAD` | 통과 |
| `python3 scripts/check_markdown_links.py` | 609개 문서, 오류 0 |
| 변경 문서 + redirect reference 검사 | 1,649개 문서, 변경 Markdown 1,055개, 오류 0 |
| `python3 scripts/check_document_metadata.py` | 기존 4개 문서의 누락 16건 재현, 신규 오류 0 |
| 이전 경로 잔존 link·canonical | 0 |
| historical 링크 오류 | 526건에서 516건으로 감소, 신규 오류 0 |
| Rust·test·Cargo·WASM·workflow 변경 | 0 |

두 외부 변경은 `npm/hwpctrl-ocx/README.md`와 `tools/hwpctrl_compat/README.md`에서 이동한 계획서를
계속 가리키도록 한 링크 정정뿐이다. 이 때문에 PR 전체는 review-only fast-pass 허용 경로에만
한정되지 않으며, GitHub Full CI 대상으로 처리되는 것이 맞다. 문서·링크 변경이므로 Rust lint,
Cargo, Native Skia, WASM과 시각 sweep은 로컬 변경 범위 검증에서 생략했다.

## 잔여 위험과 후속 경계

- 저장소 밖 웹·문서가 옛 경로를 직접 사용하면 GitHub history 이외의 링크는 깨질 수 있다. 이를
  막으려고 수천 개 redirect stub을 root에 남기지는 않으며, 실제 중요 소비자가 확인될 때 해당
  외부 링크 또는 canonical index를 정정한다.
- metadata 선행 오류 16건과 historical 링크 오류 516건은 이번 이동에서 새로 만든 결함이 아니다.
  범위를 섞어 일괄 정정하지 않았으며 다음 batch에서도 신규 오류 0을 기준으로 비교한다.
- 이 PR은 `orders/plans`만 처리한다. #6711은 `pr/report`, `working`, 다섯 root 전수 감사와 최종
  보고가 끝날 때까지 close하지 않는다.
- trailing 문서 commit도 PR 전체 impact가 두 외부 README를 포함하므로 GitHub 정책이 Full CI 또는
  신뢰 가능한 code-head 재사용 중 어느 경로를 선택했는지 결과로 확인한다. fast-pass를 가정하지 않는다.

## 최종 판정과 다음 조건

- 판정: **승인**
- 판정 대상: code candidate `968a35305b9b1b4ad209e64d97655df411754bf4`
- trailing 조건: 이 review와 오늘할일만 추가한 최신 head에서 GitHub Actions 성공,
  `MERGEABLE`·`CLEAN`, 최신 `upstream/devel` 포함 관계 재확인
- merge 조건: 최신 head SHA 고정과 사용자 merge 승인
- GitHub review: self PR이므로 approve event와 reviewer 지정 없음
- merge 뒤: 최신 `devel` 동기화 후 Stage 2-B `pr/report` continuation branch를 새로 만든다.
  #6711은 계속 open으로 유지한다.
