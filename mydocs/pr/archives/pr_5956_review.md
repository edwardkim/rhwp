---
kind: pr-review
status: self-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-24
---

# PR #5956 self-review — #4960 W7 이후 폰트 로드맵 방향 수정

## 라우팅

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서
- 작성자 본인 self-review이므로 reviewer를 지정하지 않는다.
- code candidate: `b97f3708ad2926681554c468e86685ad5a940440`

renderer·layout·paint, sample, 기준 PDF와 visual fixture 변경은 없어 `visual_fixture_evidence.md`를 적용하지
않았다. 단일 self PR이고 update branch나 오래된 base도 없어 다수 PR·재작업 경로도 적용하지 않았다.

## 접수 metadata

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR | [#5956](https://github.com/edwardkim/rhwp/pull/5956) |
| 작성자 | `edwardkim` |
| 관련 이슈 | parent [#4960](https://github.com/edwardkim/rhwp/issues/4960), W7.5 [#5955](https://github.com/edwardkim/rhwp/issues/5955) |
| base / head | `devel` / `task_m100_4960_direction_revision` |
| code candidate 규모 | 7 files, +890 / -55, 3 commits |
| 작성 시점 상태 | Open, non-draft, `MERGEABLE`, `mergeStateStatus=CLEAN` |
| reviewer | 작성자 본인 self-review, 별도 reviewer 미지정 |

1,000줄 기준 아래이며 변경 7개가 모두 `mydocs/**`다. PR 본문은 `Refs #4960`을 사용한다. #4960은
W7.5~W10을 계속 추적해야 하므로 이 PR 병합으로 닫지 않는다.

## 목적과 변경 범위

W0~W7에서 얻은 W4 위험 band, W5 Oracle disposition과 W7 read-only schema 1.0을 반영해 이후 로드맵을
단일 순차 경로에서 evidence·capability gate 기반 graph로 정정하는 것이 목적이다.

- W7과 W8 사이에 registry lifecycle·migration·evidence delta를 소유하는 W7.5 #5955를 배치했다.
- W8 #4967을 evidence-reopen, correction-qualification과 product-correction lane으로 분리했다.
- rank 8 `KoPubWorld바탕체 Light`는 위험 순위 재정렬이나 구현 확정이 아니라 process canary 후보다.
- W9 #4968은 장기 W8 tracker 전체 대신 kerning cohort와 겹치는 face의 metric·selection 동결을 요구한다.
- W10 #4969는 W9와 대상 fixture face 집합의 metric·identity 동결 뒤 진행한다.
- 로컬 canonical·report·stage 문서와 GitHub issue body·sub-issue topology를 같은 graph로 대사했다.

제품 source, schema 1.0 registry, generated projection, metric·fallback·paint, fixture, font asset와 private
corpus 자료는 변경하지 않았다. W7.5 구현과 W8~W10 착수는 각각 별도 이슈·계획·승인 대상이다.

## self-review findings

### blocker 없음

- #4939, #4961, #4962, #4963, #4964와 #4966은 닫혀 있고, #5955와 #4967~#4969는 열려 있어
  W0~W7 완료·W7.5~W10 미착수 상태가 로컬 문서와 일치한다.
- #4960 sub-issue 순서는 W2, W3·W4, W5, W6, W7, W7.5, W8, W9, W10이며 W7.5가 W8보다 앞선다.
- #4960, #5955와 #4967~#4969 본문 SHA-256 및 maintainer comment 5개의 SHA-256이 Stage R2·R3
  증적과 일치한다. 게시된 UTF-8 본문에서 BOM, `??` 치환과 literal `\\n`은 검출되지 않았다.
- W5 정본은 `stage=W5-5C`, `actionableRanks=[]`를 유지한다. 17개 disposition과 rank 1·7·8의
  acceptance ladder를 다시 계측하거나 현재 제품 교정 확정으로 오해하지 않았다.
- W7 schema 1.0과 830개 active rule의 의미를 수정하지 않고 다음 판의 생명주기 계약을 별도 W7.5로
  분리했다.

### 잔여 위험과 후속 경계

- #5955가 완료되기 전에 schema 1.0이나 generated projection을 직접 수정하면 이 PR의 보호 불변식을
  위반한다.
- rank 8 qualification은 첫 공정 검증 후보일 뿐이다. 현재 rhwp 오류·목표 decision plane·portable 정책을
  증명하지 못하면 product change 없이 닫아야 한다.
- W9는 W8 전체를 기다리지 않지만 kerning cohort와 겹치는 metric·fallback 변경을 pair positioning과 같은
  PR에 섞어서는 안 된다.
- #4960은 후속 실행 tracker이므로 이 PR 병합 뒤에도 OPEN으로 유지한다.

## 완료한 로컬 검증

| 검증 | 결과 |
| --- | --- |
| 변경 Markdown 링크 | 7개 문서, 상대 링크 이상 없음 |
| `git diff --check upstream/devel...HEAD` | 통과 |
| 변경 범위 | `mydocs/**` 7개, 제품·fixture·workflow 변경 0 |
| integration manifest | 검토 worktree에서 893 sources / 4,165 static test attrs / 32 suites + 9 exceptions 확인 |
| `cargo fmt --all`·`cargo fmt --all -- --check` | code candidate에서 통과 |
| 문서 metadata 전수 검사 | 이번 diff 밖 4개 문서의 기존 누락 16건만 재현 |

`local_validation.md`의 mydocs-only 범위에 따라 Cargo test, clippy, WASM, browser와 시각 검증은 실행 대상이
아니다. format은 저장소 공통 push gate를 충족하기 위해 generated suite를 준비한 임시 review worktree에서
별도로 통과시켰고, 파생 suite·manifest는 PR에 포함하지 않았다.

## GitHub Actions

code candidate `b97f3708a`는 review-only fast-pass B 경로를 탔다.

| workflow | run | 판정 |
| --- | --- | --- |
| CI | [32652141244](https://github.com/edwardkim/rhwp/actions/runs/32652141244) | preflight·Build & Test 성공, heavy job 정책상 skip |
| CodeQL | [32652141107](https://github.com/edwardkim/rhwp/actions/runs/32652141107) | preflight 성공, Analyze 정책상 skip |
| Proptest roundtrip | [32652141124](https://github.com/edwardkim/rhwp/actions/runs/32652141124) | preflight 성공, worker 정책상 skip |
| Adapter inter-diff | [32652141106](https://github.com/edwardkim/rhwp/actions/runs/32652141106) | preflight 성공, worker 정책상 skip |

실패하거나 대기 중인 check는 없었다. 이 self-review와 오늘할일은 code candidate 뒤의 `mydocs/` 한정
single-parent trailing commit으로 추가한다. push 뒤 exact trailing head의 preflight·Build & Test aggregate,
`MERGEABLE/CLEAN`과 최신 base를 다시 확인해야 한다.

## 최종 권고

로드맵 정정은 역사 계측과 read-only schema를 보존하면서 registry lifecycle, face 교정, kerning과 고급
shaping의 변경 이유를 분리한다. 로컬 정본과 GitHub topology의 상태·의존성·본문 hash가 일치하고 제품
변경은 없다. 추가 blocker는 발견하지 않았다.

self-review는 **완료 / 조건부 merge 권고**다. trailing review-only head의 fast-pass, 최신
`MERGEABLE/CLEAN`, #4960 OPEN 유지와 메인테이너의 별도 merge 승인을 확인하기 전에는 merge하지 않는다.
