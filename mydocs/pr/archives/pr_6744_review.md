---
kind: pr_review
status: approved
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-05
pr: 6744
issue: 6711
author: edwardkim
---

# PR #6744 self-review — 월별 archive 전수 감사와 최종 보고

## 결론

**승인.** PR #6744의 code candidate
`a3f2ab3e26939672126bc86d7705a14f74584868`은 #6711 Stage 4 전수 감사 결과와 메인테이너가
승인한 최종 보고서를 기록한다. 기준선의 cutoff 이전 후보 3,844개는 Git rename 3,832개와
SHA-256 동일본 제거 12개로 모두 귀결됐고, 기준선의 9월 문서 72개는 유지됐다. cutoff 이전 root
잔여와 예상 archive 목적지 누락은 각각 0개이며, 상이 충돌 4개는 기존본과 suffix 보존본이 모두
남아 있다.

candidate 변경은 수행계획서 2줄과 신규 보고서 한 개뿐이다. Rust source·test, Cargo, WASM,
workflow, renderer, sample과 PDF bytes는 바뀌지 않았다. 로컬 문서 검사와 exact-head 선택적 CI가
성공했고 최신 `upstream/devel`과의 merge tree도 깨끗하므로 범위 안 blocker는 없다.

이 문서의 `승인`은 작성자 self-review 판정이다. 자기 PR이므로 reviewer 지정이나 GitHub approve
event를 만들지 않는다. 이 review·오늘할일과 보고서의 당월 root 수 설명만 추가한 trailing head의
GitHub Actions, `MERGEABLE`·`CLEAN`, 최신 `devel` 정합을 다시 확인하고 메인테이너의 별도 merge
승인을 받아야 한다.

## 라우팅과 메타데이터

- 기본 경로: `collaborator_self_merge.md`
- 보조 경로: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`와 위 자식 문서
- `review_impl`은 추가하지 않는다. 승인된 [수행계획](../../plans/task_m100_6711.md),
  [Stage 1](../../working/task_m100_6711_stage1.md),
  [Stage 2-A](../../working/task_m100_6711_stage2.md),
  [Stage 2-B](../../working/task_m100_6711_stage2b.md),
  [Stage 3](../../working/task_m100_6711_stage3.md),
  [최종 보고서](../../report/task_m100_6711_report.md)가 실행·검증 계보를 고정한다.

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#6744](https://github.com/edwardkim/rhwp/pull/6744) / @edwardkim |
| 관련 이슈 | [#6711](https://github.com/edwardkim/rhwp/issues/6711) (`Refs #6711`) |
| base | `devel@693c4b6b3edd1317934d6648449edcf47b0689e3` |
| code candidate | `a3f2ab3e26939672126bc86d7705a14f74584868` |
| 규모 | 2 files, `+172/-0`, 1 commit |
| 작성 시점 GitHub 상태 | Open, 비 Draft, `MERGEABLE`·`CLEAN`, candidate checks 완료 |
| assignee / label / milestone | `edwardkim` / `documentation` / `v1.0.0` |
| reviewer | self PR이므로 지정하지 않음 |

GitHub REST 전수 조회 결과는 `mydocs/plans/task_m100_6711.md` 수정 1개와
`mydocs/report/task_m100_6711_report.md` 추가 1개로 로컬 diff와 일치한다.

## 전수 감사 재검토

| 항목 | 결과 |
| --- | ---: |
| 기준선 direct-root Markdown | 3,916 |
| cutoff 이전 후보 | 3,844 |
| 기준선 9월 문서 보존 | 72/72 |
| Git rename | 3,832 |
| 그중 byte-identical `R100` | 3,289 |
| 링크·canonical 정정 포함 rename | 543 |
| SHA-256 동일본 root 제거 | 12/12 |
| 상이 충돌 양쪽 보존 | 4/4 |
| 예상 archive 목적지 누락 | 0 |
| cutoff 이전 direct-root 잔여 | 0 |

후보 합계는 `3,289 + 543 + 12 = 3,844`로 닫힌다. 이동 commit의 rename·삭제 수는
Stage 2-A `771/2`, Stage 2-B `830/2`, Stage 3-A `1,111/8`, Stage 3-B `1,120/0`으로
단계 보고서와 일치한다. 동일본 12개는 기준선 root와 기존 archive의 SHA-256이 모두 같고, 상이
충돌 4개는 hash가 다른 기존 archive를 덮어쓰지 않았다.

Stage 3-B merge tree의 root 79개는 기준선 생존 72개와 #6711 문서 5개, 동시 작업 #6717 문서
2개다. 최신 base의 root 80개도 Git 최초 도입일을 전수 판정해 모두 cutoff 이후이고 판정 불가
경로는 없다. candidate 보고서와 trailing 오늘할일은 9월 생성 문서이므로 root에 남는 것이
거버넌스에 부합한다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| `git diff --check upstream/devel...HEAD` | 통과 |
| `python3 scripts/check_markdown_links.py --changed-from upstream/devel` | 611개 문서, 오류 0 |
| candidate 보고서 단독 링크 검사 | 오류 0 |
| 추적 Markdown 전수 | base 13,195개 / candidate 13,196개 |
| 내부 링크 | base 9,250개 / candidate 9,252개 |
| 유효 / historical broken | candidate 8,699 / 553, base 대비 broken 증가 0 |
| `python3 scripts/check_document_metadata.py` | 변경하지 않은 기존 4개 문서의 16건만 재현, 신규 0 |
| current-base merge tree | `2f44a4a48a19bfe4c6cbeeedf803e7e7461906cc` |
| 범위 감사 | Rust·Cargo·WASM·workflow 변경 0 |

렌더링·레이아웃과 binary 자산을 바꾸지 않아 시각 검증은 비대상이다. 문서 전용 변경에는 Rust
lint·build와 WASM 검증을 확장하지 않는 프로젝트 정책을 적용했다.

## GitHub Actions와 최신 base

candidate SHA에 대해 다음 workflow와 exact-head status가 성공했다.

- CI [run 33929368594](https://github.com/edwardkim/rhwp/actions/runs/33929368594)
- CodeQL [run 33929368585](https://github.com/edwardkim/rhwp/actions/runs/33929368585)
- Adapter inter-diff [run 33929368578](https://github.com/edwardkim/rhwp/actions/runs/33929368578)
- Proptest roundtrip [run 33929368737](https://github.com/edwardkim/rhwp/actions/runs/33929368737)
- CI Impact Policy Controller
  [run 33929368433](https://github.com/edwardkim/rhwp/actions/runs/33929368433)
- exact-head `CI Impact Policy`
  [run 33929400024](https://github.com/edwardkim/rhwp/actions/runs/33929400024)

문서 전용 selective 경로에서 preflight·trusted reuse·`Build & Test`가 성공하고 Rust·WASM·frontend
등 비대상 job은 정책상 skip됐다. failure·cancelled·timed out·pending check는 0개다.
`upstream/devel@693c4b6b3e`은 candidate의 직접 parent이며 작성 시점에 전진하지 않았다.

## 잔여 위험과 후속 경계

- historical broken link 553건과 metadata 오류 16건은 #6711이 새로 만든 오류가 아니다. 이번
  작업에서 범위를 섞어 정정하거나 성공 수치에서 숨기지 않았다.
- 저장소 밖 소비자가 옛 root 경로를 사용할 가능성은 남는다. 대량 redirect stub은 만들지 않고
  실제 중요 소비자가 확인될 때 canonical index나 해당 외부 링크를 정정한다.
- CodeQL alert #186은 메인테이너가 수동 입력한 `used in tests` 분류를 유지한다. 근거화·재발 방지는
  [#6731](https://github.com/edwardkim/rhwp/issues/6731)에서 별도로 처리하며 이 PR은 alert 상태를
  변경하지 않는다.
- #6711은 merge SHA의 post-merge 검증이 성공하기 전까지 OPEN으로 유지한다.

## Merge 후 계획

정상 merge commit이 `devel`에 반영되고 merge SHA의 필수 Actions가 성공한 뒤 다음 순서로 처리한다.

1. PR #6744에 candidate·trailing head·merge SHA와 post-merge 검증 결과를 남긴다.
2. #6711에 3,844개 후보 귀결, 충돌·링크·metadata 감사와 merge SHA를 기록한다.
3. #6711의 완료조건을 다시 확인한 뒤 close한다.
4. 최신 `devel`을 로컬에 fast-forward하고 이번 task의 local·remote branch만 정리한다.

게시 뒤 API로 한글·선두 BOM·`??` 치환과 SHA·run URL을 검증한다. 같은 사실의 maintainer comment가
이미 있으면 중복 게시하지 않는다. #6731은 OPEN으로 유지한다.

## 최종 판정과 다음 조건

- 판정: **승인**
- 판정 대상: code candidate `a3f2ab3e26939672126bc86d7705a14f74584868`
- trailing 조건: 이 review·오늘할일과 보고서 수치 설명만 추가한 최신 head의 GitHub Actions 성공,
  `MERGEABLE`·`CLEAN`, 최신 `upstream/devel` 정합 재확인
- merge 조건: 최신 head SHA 고정과 메인테이너의 별도 merge 승인
- GitHub review: self PR이므로 approve event와 reviewer 지정 없음
- merge 방식: branch protection을 우회하지 않는 정상 merge commit
- merge 뒤: post-merge 검증 성공 후 #6711 완료 comment·close와 task branch 정리
