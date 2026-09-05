---
kind: pr_review
status: approved
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-05
pr: 6748
issue: 6731
author: edwardkim
---

# PR #6748 self-review — CodeQL password open 반환 경계 분리

## 결론

**승인.** PR #6748의 code candidate
`751b4c5fc67f4e63c147a2aa7f3acced928839be`는 암호 문서 open command와 `DocumentInfo` query를
분리한다. raw password가 metadata 반환 경계를 타지 않으며 기존 문서의 원자적 교체, 암호 재입력,
보호 저장과 평문 public API 계약을 유지한다.

로컬 회귀검사와 exact merge-ref CI가 모두 성공했다. JavaScript/TypeScript CodeQL은 workflow·query·path
제외 없이 87개 rule을 실제 실행했고 결과 0건, merge ref open alert 0건으로 종료됐다. alert #186의
메인테이너 판정 `dismissed` / `used in tests`도 변경하지 않았다. 범위 안 구현 blocker는 없다.

이 문서의 `승인`은 작성자 self-review 판정이다. 자기 PR이므로 reviewer 지정이나 GitHub approve event를
만들지 않는다. 이 review와 승인된 최종 보고서·운영 문서를 추가한 review-only trailing head의 Actions,
최신 `devel` 정합과 PR metadata를 확인한 뒤 메인테이너의 별도 merge 승인을 받아야 한다.

## 라우팅과 메타데이터

- 기본 경로: `collaborator_self_merge.md`
- 보조 경로: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`와 위 자식 문서
- 구현·검증 계보: [수행계획](../../plans/task_m100_6731.md),
  [Stage 1](../../working/task_m100_6731_stage1.md),
  [Stage 2](../../working/task_m100_6731_stage2.md),
  [Stage 3](../../working/task_m100_6731_stage3.md),
  [Stage 4](../../working/task_m100_6731_stage4.md),
  [최종 보고서](../../report/task_m100_6731_report.md)

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#6748](https://github.com/edwardkim/rhwp/pull/6748) / @edwardkim |
| 관련 이슈 | [#6731](https://github.com/edwardkim/rhwp/issues/6731) (`Refs #6731`) |
| base | `devel@9bf5bcfd061b491c89be1ea28ef1fff8a892b6d1` |
| code candidate | `751b4c5fc67f4e63c147a2aa7f3acced928839be` |
| 규모 | 14 files, `+669/-24`, 5 commits |
| 작성 시점 GitHub 상태 | Open, 비 Draft, `MERGEABLE`·`CLEAN`, candidate checks 완료 |
| assignee / label / milestone | 미지정 / 미지정 / 미지정 — #6731 metadata 승계 필요 |
| reviewer | self PR이므로 지정하지 않음 |

PR metadata가 생성 시점에 누락됐다. 병합 전 #6731과 동일하게 assignee `edwardkim`, label `bug`·`ci`,
milestone `v1.0.0`으로 트리야지하고 API로 재조회한다. 이는 코드 판정을 바꾸지 않지만 저장소 운영 완료
조건이다.

## 구현 검토

1. `openPasswordProtectedDocument()`는 `Promise<void>` command로 성공·실패만 표현한다.
2. `WasmBridge.loadDocumentWithPassword()`와 공통 `loadDocumentAtomically()`도 `void`이며 내부에서
   `DocumentInfo`를 만들거나 반환하지 않는다.
3. 암호 open 성공 뒤 `loadDocumentForOpen()`이 `getDocumentInfo()` query를 한 번 호출한다.
4. 평문 `loadDocument()`는 공통 command 성공 뒤 같은 query를 호출해 기존 반환 계약을 유지한다.
5. command가 실패하면 준비 중 문서만 해제하고 기존 문서·최근 문서 상태를 유지한다.

PR 생성 전 공통 atomic helper의 metadata 생성이 남아 있던 허점을 발견하고 code candidate에서 제거했다.
이로써 외부 facade만 `void`로 바꾼 위장 분리가 아니라 내부 생성부터 query 시점까지 경계가 닫혔다.

## 보안·회귀 계약

| 계약 | 확인 결과 |
| --- | --- |
| raw password 지역성 | open 시도 지역 범위 밖 저장·반환·로그 없음 |
| `DocumentInfo` 필드 | 승인된 8개 metadata 필드만 허용 |
| font cache projection | `langId`, `fontName`, `altType`만 사용 |
| snapshot digest 입력 | canonical font rows만 SHA-256 처리 |
| CodeQL 탐지 범위 | workflow·query·language·path 변경 없음 |
| 문서 lifecycle | HWP3·HWP5·HWPX 취소·오입력·성공·보호 저장 통과 |
| #4430 보호 Save As | 현재 UI 계약에 맞춘 `암호 없이 저장` 선택 뒤 전체 E2E 통과 |

#4430 E2E 정정은 제품 동작 우회가 아니다. 2026-08-30 통합된 보호 Save As의 기본 암호 계승과 갱신되지
않은 평문 사본 test가 충돌한 기준선 불일치를 현재 명시적 UI 선택에 맞춘 것이다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| `node --test rhwp-studio/tests/hwp-password-open.test.ts` | 7/7 성공 |
| `npm --prefix rhwp-studio test` | 1,376건, 성공 1,375·실패 0·기존 skip 1 |
| `npm --prefix rhwp-studio run build` | 성공 |
| `npm --prefix rhwp-studio run e2e:hwp-password-open` | HWP3·HWP5·HWPX 전체 성공 |
| `npm --prefix rhwp-studio run e2e:issue-4430-content-loss` | 전체 성공 |
| `git diff --check` | 성공 |
| `python3 scripts/check_markdown_links.py --changed-from upstream/devel` | 내부 상대 링크 이상 없음 |
| `python3 scripts/check_document_metadata.py` | 기존 4개 문서의 16건만 재현, 신규 오류 0 |

Rust source·Rust test·WASM binding과 renderer·layout·fixture는 바꾸지 않았다. 따라서 Rust lint·release-test,
WASM build와 시각 검증은 로컬 변경 범위상 비대상이다.

## exact merge-ref와 GitHub Actions

- merge ref: `a547f5ae51e5c101d98c4618f610f73ef5d9eb36`
- parent 1: base `9bf5bcfd061b491c89be1ea28ef1fff8a892b6d1`
- parent 2: code candidate `751b4c5fc67f4e63c147a2aa7f3acced928839be`
- CI: [run 33933481838](https://github.com/edwardkim/rhwp/actions/runs/33933481838), 성공
- CodeQL: [run 33933481795](https://github.com/edwardkim/rhwp/actions/runs/33933481795), 성공
- JavaScript/TypeScript analysis: `1727960739`, 87 rules, results 0, error 없음
- `refs/pull/6748/merge` open code-scanning alert: 0건
- Render Diff·Proptest roundtrip·Adapter inter-diff·CI Impact Policy: 성공

Rust·Python job은 success로 종료됐지만 내부 analysis step은 impact policy에 따라 skip됐다.
JavaScript/TypeScript analysis는 2분 43초 실제 실행됐다. 따라서 trusted reuse나 비대상 언어의 no-op
성공을 alert 제거 근거로 오인하지 않았다.

## 운영 문서와 증적 검토

- [조사 README](../../tech/investigations/issue-6731/README.md)는 최초·현재 분석과 원인 계보를 설명한다.
- 최초·현재 정규화 JSON은 네 flow의 모든 location과 topology를 보존하며 raw password·token·절대 경로와
  private corpus 식별자를 포함하지 않는다.
- [GitHub 저장소 운영 매뉴얼](../../manual/github_operations.md#94-codeql-alert-귀속과-used-in-tests-근거-보존)은
  source/sink 도입, flow 성립, 최초 분석 등록과 현재 PR 귀속을 분리한다.
- [최종 보고서](../../report/task_m100_6731_report.md)는 원인·구현·검증·잔여 위험과 post-merge 종료 조건을
  연결하며 메인테이너 결과 승인을 반영해 `final`이다.

## 잔여 위험과 후속 경계

- CodeQL 분석기 version이 바뀌면 data-flow가 다시 확장될 수 있다. 재발 시 기존 분류만 재사용하지 않고
  보존한 topology와 새 SARIF를 비교한다.
- PR 분석은 merge tree 결과다. 실제 `devel` merge SHA의 full CodeQL이 같은 JavaScript/TypeScript
  분석을 성공하기 전에는 #6731을 닫지 않는다.
- PR #6725·#6637이 동일 파일을 변경하고 있어 먼저 병합되면 최신 `devel`과 mergeability를 다시 확인한다.
- review-only trailing commit에는 제품 source·test·workflow를 추가하지 않는다. fast-pass가 원래 녹색
  code candidate를 정확히 재사용하는지 확인한다.

## Merge 후 계획

정상 merge commit이 `devel`에 반영된 뒤 다음 순서로 처리한다.

1. merge SHA의 `devel` full CodeQL에서 JavaScript/TypeScript analysis와 alert #186 비재발을 확인한다.
2. workflow·query·language·path가 유지됐는지 재확인한다.
3. PR #6748에 candidate·trailing head·merge SHA와 post-merge 결과를 기록한다.
4. #6731에 원인 계보, command/query 경계와 branch 분석 결과를 기록하고 완료조건 충족 시 close한다.
5. 로컬 `devel`을 fast-forward하고 이번 task의 local·remote branch만 정리한다.

게시 뒤 API로 한글·선두 BOM·`??` 치환과 SHA·run URL을 검증한다. 같은 사실의 maintainer comment가 이미
있으면 중복 게시하지 않는다.

## 최종 판정과 다음 조건

- 판정: **승인**
- 판정 대상: code candidate `751b4c5fc67f4e63c147a2aa7f3acced928839be`
- trailing 조건: report·manual·review·작업 기록만 추가하고 Actions fast-pass, 최신 `upstream/devel`,
  `MERGEABLE`·`CLEAN` 재확인
- metadata 조건: assignee `edwardkim`, label `bug`·`ci`, milestone `v1.0.0` 적용·재조회
- merge 조건: 최신 head SHA 고정과 메인테이너의 별도 merge 승인
- GitHub review: self PR이므로 approve event와 reviewer 지정 없음
- merge 방식: branch protection을 우회하지 않는 정상 merge commit
- merge 뒤: `devel` full CodeQL 성공 후 #6731 결과 comment·close와 task branch 정리
