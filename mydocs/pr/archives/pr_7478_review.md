---
kind: investigation
status: active
---

# PR #7478 리뷰 — HWPX fallback layoutCompatibility 자기닫힘

## 최종 판정

**승인 — 검토한 원 contributor head의 serializer 구조 변경 범위에 한정합니다.**
검토 head는 `a53850cb6ccd7e0037a115f1fdea422a98b05a26`입니다.
GitHub Approve review는 본문을 작업지시자에게 제시하고 확인받은 뒤 게시합니다.
Merge 전 조건은 최신 대상 head·CI·mergeability 재확인과 별도 작업지시자 승인입니다.
현재 기록은 push 전 검토용 문서 commit의 내용입니다. source branch push, GitHub review 게시,
merge는 각각 별도 작업지시자 승인을 받은 뒤에만 수행합니다.

## 접수 정보

| 항목 | 확인 결과 |
| --- | --- |
| PR / 작성자 / base | [#7478](https://github.com/edwardkim/rhwp/pull/7478) / moongioh / devel |
| reviewer | 연결 계정 postmelee이며 requested_reviewers에 이미 지정돼 있습니다. |
| 원 head / 검토 후보 | 둘 다 a53850cb6ccd7e0037a115f1fdea422a98b05a26; 보정 없음 |
| 비교 devel | 0e8fd49fb868da0d47ac1294dcbbda81f0211233 |
| 작성 시점 상태 | OPEN, non-draft, mergeable=true, mergeable_state=clean, maintainer_can_modify=true |
| 규모 / 관련 이슈 | 1 commit, 1 file, +12/-9; 별도 관련 issue 없음 |
| 작성자 맥락 | 저장소의 author:moongioh PR 검색 결과는 #7478 한 건으로, 첫 기여자 경로를 적용했습니다. |

## 적용 절차와 검토 범위

기본 경로는 [collaborator 외부 PR](../../manual/pr_review/collaborator_external_pr.md)입니다.
[접수와 기록](../../manual/pr_review/intake_and_review.md),
[리뷰 템플릿](../../manual/pr_review/review_template.md),
[로컬 검증](../../manual/pr_review/local_validation.md),
[첫 기여자](../../manual/pr_review/first_time_contributor.md)를 읽고 적용했습니다.
[공통 workflow](../../manual/pr_review_workflow.md)의 판정과 원격 조치 분리를 따릅니다.
이전 작업지시의 격리 검토 지시에 따라 원 작업공간을 보존하고 별도 review worktree를 사용했습니다.
이번 범위는 원 contributor head 검토, 리뷰·오늘할일 후속 문서 commit과 Approve 본문 준비입니다.
코드 통합·보정은 없습니다. 원 contributor commit을 그대로 parent로 유지하는 direct source의
review-only A 경로를 사용하며, 원 contributor history를 rewrite하지 않습니다.
검토 기록과 [오늘할일](../../orders/20260930.md)은 같은 후속 commit에 포함합니다.

전체 diff와 write_header 호출 분기를 대조했습니다.
[header.rs](../../../src/serializer/hwpx/header.rs)의 hwpx_head_tail=Some 경로는 원본 tail을 그대로 쓰고,
None 경로에서만 write_compatible_document를 호출합니다. 이 fallback에서 다섯 자식 요소를 제거하고
layoutCompatibility를 자기닫힘으로 출력합니다. targetProgram과 나머지 설정 tail은 변경하지 않습니다.

## 공통 조판 원칙과 시각 검증

판정: **비해당**. serializer 헤더 구조 보존 변경이며 renderer/layout/typeset/paint, 줄 구성,
측정·배치, 분할·이어받기, baseline/golden/래칫 변경이 없습니다.
샘플·PDF 변경도 없습니다. SVG/fresh WASM/Visual Sweep을 승인 필수 항목으로 확대하지 않았습니다.
한컴 2014 종료·정상 열기 대조는 작성자가 제공한 증거이며 reviewer가 직접 재현하지 않았습니다.
이번 승인은 그 버전의 실행 검증 완료를 뜻하지 않습니다.

## 검증 입력 커밋 확인

판정: **충족** — 구조 대조에 사용한 아래 한 파일에 한정합니다.

| 입력 | 역할 / 직접 확인 | SHA-256 / commit |
| --- | --- | --- |
| samples/hwpx/ref/ref_empty.hwpx | 커밋된 reference header ZIP/XML을 직접 열어 layoutCompatibility가 하나이고 자식이 0개임을 확인했습니다. 로컬 bytes와 git show의 blob을 대조했습니다. | c58144645069f7d1258e91404730618ad568bc4d47680ad5f891d3050aa308c7 / a53850cb6ccd7e0037a115f1fdea422a98b05a26 |

Header 단위 테스트가 코드에서 생성하는 Document/XML 입력에는 별도 fixture 파일이 필요하지 않습니다.
외부 한컴 실험 파일을 reviewer의 실행 입력으로 사용하거나 커밋 확인 완료로 기록하지 않았습니다.

## 검증 결과와 재사용

- git fetch 후 최신 devel 및 PR head를 확인했습니다. git diff --check origin/devel...origin/pr7478은 통과했습니다.
- git merge-tree --write-tree origin/devel origin/pr7478은 충돌 없이 통과했습니다.
  결과 tree: 99c2244593a262a47bfe6e9c632751b5cab38aa1.
- [정확한 head CI](https://github.com/edwardkim/rhwp/actions/runs/36531162748)의 A–D archive build/test,
  lint, Build & Test 성공을 확인했습니다. [Archive A 실제 로그](https://github.com/edwardkim/rhwp/actions/runs/36531162748/job/109289989898)에서
  serializer::hwpx::header 48개, parser::hwpx::header 54개 PASS를 확인했습니다.
  fallback 기대값 검사와 원본 tail verbatim-splice 검사도 실제 실행되어 통과했습니다.
- CodeQL 36531162729, Adapter inter-diff 36531162780, Proptest roundtrip 36531162755와
  CI Impact Policy 36533506926의 성공을 확인했습니다.
- WASM Build, Native Skia, frontend gate는 이 run에서 skipped입니다. 실행 통과로 세지 않았습니다.
- 정확한 code head의 CI가 녹색이고 source/test/fixture/workflow 보정이 없으며 현재 base merge simulation이
  통과했으므로 공통 workflow 3.2.2에 따라 로컬 전체 nextest·Native Skia·lint를 중복 실행하지 않았습니다.
  이번 회차의 로컬 Cargo test는 미실행이며, 원격 실제 검사 결과를 reviewer 로컬 실행으로 쓰지 않습니다.
- 수정 전 fallback 테스트 실패는 작성자의 음성 대조 주장으로 확인했고 reviewer가 다시 실행하지 않았습니다.
  기존 source의 다섯 자식과 수정된 기대 문자열의 차이는 diff에서 직접 확인했습니다.

## 발견 사항과 잔여 경계

구체적 도입 결함 또는 차단 사항을 찾지 못했습니다. 원본 tail 보존의 대조군이 유지됩니다.
Header 구조 일치와 실제 테스트 성공을 근거로 원 contributor head를 승인 후보로 판단했습니다.
한컴 2014 직접 실행, 모든 외부 소비자의 호환성, 시각 fidelity 전반은 검증 범위 밖입니다.
기존 review와 inline thread는 재조회 시 비어 있었습니다.

## 처리 계획

1. 원 code head의 required CI 성공을 확인한 뒤 review와 오늘할일만 single-parent trailing commit으로 준비합니다.
2. 최신 devel·원 contributor branch SHA를 고정하고 merge simulation, merge tree의 공백·변경 문서 링크·
   오늘할일 기록 보존, LFS 사전 판독과 remote push dry-run을 확인합니다. 정확한 trailing head와 결과는
   push 승인 요청에 함께 제시합니다. 미검증 또는 실패 항목이 있으면 push하지 않습니다.
3. 작업지시자의 push 승인을 받은 뒤 원격 SHA를 다시 확인하고 문서 commit만 contributor source branch에
   push합니다. push 후 exact trailing head의 preflight·필수 aggregate·mergeability를 확인합니다.
   fast-pass 허용 여부는 실제 preflight 결과로 판단하며, 허용되지 않으면 Full CI 완료를 기다립니다.
4. 한국어 존댓말 Approve 본문을 작업지시자에게 제시합니다. 별도 게시 승인을 받은 뒤 최신 head와
   기존 review를 다시 확인하고 최신 검토 commit_id로 APPROVE를 제출합니다. 본문에는 원 code
   candidate SHA와 review-only trailing head의 역할을 구분합니다.
5. 게시 결과의 author/state/commit_id/body를 API로 재조회하여 UTF-8·실제 줄바꿈과 본문 일치를 확인합니다.
6. push 또는 Approve 게시 승인을 merge, close 또는 merge 후 comment의 승인으로 확대하지 않습니다.

## Merge 후 contributor PR comment 계획

시각 검증은 비해당이며 Visual Sweep 수치·asset URL을 만들지 않았습니다.
Merge와 후속 comment가 별도 승인된 경우에만 확정 merge SHA·CI와 최초 기여 감사 문안을 준비합니다.
이번 Approve 단계에서는 merge 후속 게시를 실행하지 않습니다.
