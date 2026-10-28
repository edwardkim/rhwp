---
kind: pr_review
status: approved
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-03
pr: 6696
issue: 6695
author: edwardkim
---

# PR #6696 self-review — PDF 오라클 단일 경로·Git LFS 폐지

## 결론

**승인.** PR #6696은 한컴 PDF 오라클을 `pdf/**` 한 경로의 일반 Git blob으로 통합하고,
`pdf-2020/**`·`pdf-large/**`와 Git LFS를 활성 운영 경로에서 제거한다. 이동 전 원장의 실제 PDF
bytes와 이동 뒤·독립 clone의 SHA-256을 전건 대조했으며, workflow·Oracle Public·LLM verifier와
운영 문서도 같은 단일 경로 계약으로 맞췄다.

self-review에서 수동 Oracle Public Advisory의 embedded Python gate가 `os.environ`을 사용하면서
`import os`를 빠뜨린 blocker를 발견했다. code candidate
`872afc8836e7976ce5e12e0ad71613c565d65e00`에서 import를 복원하고 workflow 본문을 tiny 저장소에서
그대로 실행하는 회귀 테스트를 CI 계약 묶음에 연결했다. 이후 로컬 검증과 GitHub Full CI는 모두
성공했다. 새 blocker는 발견되지 않았다.

이 문서의 `승인`은 작성자 self-review 판정이다. 자기 PR이므로 reviewer 지정이나 GitHub approve
review event를 만들지 않는다. 이 review·오늘할일만 추가하는 후행 head의 review-only checks와
mergeability를 다시 확인한 뒤 정상 merge할 수 있으며, remote push와 merge는 각각 별도 사용자
승인 게이트다.

## 라우팅과 메타데이터

- 기본 경로: `collaborator_self_merge.md`
- 보조 경로: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`,
  `review_only_fast_pass.md`, `rework_and_exceptions.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`와 위 자식 문서
- `review_impl`은 추가하지 않는다. 승인된 계획서·Stage 보고서·최종 보고서가 구현과 검증 계보를
  이미 고정한다.

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#6696](https://github.com/edwardkim/rhwp/pull/6696) / @edwardkim |
| 관련 이슈 | [#6695](https://github.com/edwardkim/rhwp/issues/6695) (`Closes #6695`) |
| base | `devel@b6b9384ed9e679a8d143c6dcbaacef502614eb9e` |
| code candidate | `872afc8836e7976ce5e12e0ad71613c565d65e00` |
| 규모 | 177 files, `+113,773/-50,897`, 7 commits |
| 작성 시점 GitHub 상태 | Open, 비 Draft, `MERGEABLE`, `CLEAN`; 성공 check 31, 정책 context 성공 1, skip 4, 실패·대기 0 |
| reviewer | self PR이므로 지정하지 않음 |

1,000줄을 넘으므로 대형 PR 경로를 적용했다. 증가분의 대부분은 Oracle Public catalog·report·
transcript와 LLM verifier corpus 16개 shard의 정식 재생성 결과다. 이 생성물을 source 계약과
분리하면 경로·pair·coverage·manifest가 서로 어긋나므로 논리적으로 원자적이다. PDF binary는
이동 원장으로 별도 검산했고, 작업 중 최신 `devel`을 보존한 merge commit도 이력에 남겼다.

## 자산 전환과 보호 불변식

- 최종 `pdf/**`에는 실제 PDF 1,178개, 1,079,000,023 bytes가 있다. 최대 파일은
  50,228,784 bytes로 정한 50 MiB 미만 상한을 충족한다.
- `pdf-2020/**` 1개와 `pdf-large/**` 18개의 old/new 경로·크기·SHA-256을 먼저 동결했다. LFS
  pointer 17개의 선언 OID·크기와 local object의 실제 bytes·`%PDF-` magic이 모두 일치했다.
- 기존 목적지와 byte-identical한 2개는 목적지를 보존하고 source만 제거했다. 신규 목적지 17개는
  이동 뒤 원장 SHA-256과 모두 일치하며 hash 충돌이나 복원 불가 object는 0개다.
- 현재 tree의 `pdf-2010/`, `pdf-2020/`, `pdf-large/`, PDF LFS pointer와 PDF LFS attribute는
  0개다. 공개 history rewrite, force-push, 원격 LFS object purge는 수행하지 않았다.
- 신규 정책 검사기는 정본 경로, 50 MiB 상한, PDF magic, LFS pointer·attribute, 폐기 root의
  재등장을 fail-closed로 검사한다.

## 소비자·생성물 계약

- CI·CodeQL·Pages·Render Diff·Adapter Diff·Proptest workflow와 CI impact classifier·policy는
  `pdf/**`만 PDF review-only 입력으로 인정한다. 폐기 root가 다시 나타나면 fast-pass로 접지 않는다.
- Oracle Public은 sample 979개, 매칭 sample 565개, pair 1,030개, unmatched 414개로 재생성됐다.
  PDF 1,178개의 page count 측정에 모두 성공했다.
- 파일명에서 명시적으로 판독한 버전은 1,068개이며 suffix가 없는 110개는 `unknown`으로 보존했다.
  과거 디렉터리 이름을 버전의 대리 신호로 사용하지 않는다.
- LLM verifier corpus 16개 shard, 122,400행을 단일 root로 다시 만들었고 verifier error는 0건이다.
- historical CHANGELOG·완료 계획·과거 PR 기록은 당시 사실을 보존했다. 살아 있는 안내와 실행 경로만
  새 정본으로 바꿔 역사 기록을 현행 규약처럼 오인하지 않게 했다.

## self-review blocker와 정정

최초 PR head의 `.github/workflows/oracle-public-advisory.yml`은 PDF gate에서
`os.environ["GITHUB_OUTPUT"]`을 호출하지만 `import os`가 없었다. 이 workflow는
`workflow_dispatch` 전용 advisory이고 required check가 아니므로 일반 PR CI가 embedded 본문의
실행 오류를 직접 드러내지 못했다.

정정 commit `872afc883`은 다음 최소 범위만 바꿨다.

1. embedded gate에 `import os`를 복원했다.
2. workflow에서 정확한 Python 본문을 추출해 tiny PDF 저장소에서 실행하고 `GITHUB_OUTPUT`의
   `real_pdfs`, `lfs_pointers`, `runner`, `reason`, `should_run`을 검증한다.
3. 단일 `pdf` root와 수동·비필수 advisory 경계를 계약 테스트로 고정했다.
4. 새 테스트를 CI의 `Validate workflow contracts` 단계에 연결했다.

정정 뒤 해당 CI 단계가 원격에서도 통과했다. 단순 문자열 존재 검사에 머물지 않고 실제 embedded
프로그램의 import·파일 접근·출력 계약을 실행하므로 같은 종류의 누락을 PR CI에서 탐지한다.

## 로컬·독립 clone 검증

[최종 보고서](../../report/task_m100_6695_report.md),
[Stage 1 원장](../../working/task_m100_6695_stage1.md),
[수행계획](../../plans/task_m100_6695.md)에 다음 결과를 고정했다.

| 검증 | 결과 |
| --- | --- |
| PDF repository policy | 7 tests와 live tree 통과 |
| Oracle Public Advisory embedded gate | 실제 실행 포함 3/3 통과 |
| workflow Python contract discovery | 179/179 통과 |
| CI impact classifier / policy | 44/44, 37/37 통과 |
| trusted post-merge reuse | 18/18 통과 |
| workflow YAML parse | 21개 통과 |
| Oracle Public / LLM verifier unit | 139/139, 42/42 통과 |
| LLM verifier corpus | 122,400행, error 0 |
| Rust native·WASM32·workspace gate | fmt, clippy, build, manifest 모두 통과 |
| Markdown 링크·`git diff --check` | 655 docs, 통과 |

LFS object store가 없는 `--no-local --single-branch` 독립 clone에서도 PDF 1,178개,
1,079,000,023 bytes, magic 1,178/1,178, LFS pointer 0, 원장 SHA-256 19/19 일치를 확인했다.
현재 clone이 현행 PDF를 받는 데 LFS smudge나 별도 LFS bandwidth를 요구하지 않는다는 직접 증거다.

## GitHub Full CI

code candidate `872afc8836e7976ce5e12e0ad71613c565d65e00`에서 성공 check 31개와 exact-head
CI 정책 context 1개가 성공했고, 조건상 비대상 4개만 skip됐다. 실패·대기는 0개다.

- [CI 33766355053](https://github.com/edwardkim/rhwp/actions/runs/33766355053): workflow 계약,
  Lint, Native Skia, frontend package, 네 archive build/test shard와 Build & Test 성공
- [CodeQL 33766355058](https://github.com/edwardkim/rhwp/actions/runs/33766355058): Rust 15분 11초,
  Python, JavaScript/TypeScript 분석 성공
- [Render Diff 33766354747](https://github.com/edwardkim/rhwp/actions/runs/33766354747): preflight와
  Canvas visual diff 성공
- [Proptest 33766355122](https://github.com/edwardkim/rhwp/actions/runs/33766355122): preflight와
  prop roundtrip 성공
- [Adapter inter-diff 33766355027](https://github.com/edwardkim/rhwp/actions/runs/33766355027): preflight와
  본 검사 성공
- [CI Impact Policy Controller 33767899366](https://github.com/edwardkim/rhwp/actions/runs/33767899366):
  `mode=full`, workflow·Rust·frontend package·render·Skia·세 CodeQL 언어 대상 판정과 최종 정책 성공

## 시각 검증 판정

별도 PDF 시각 sweep은 요구하지 않는다. 이 PR은 PDF를 새로 생성하거나 renderer·layout을 바꾸지 않고
검산된 동일 bytes를 경로 이동했다. 따라서 이 작업의 정답은 pixel 유사도가 아니라 이동 전후와 독립
clone의 byte-identical SHA-256 19/19 보존이다. Render Diff가 Full CI에서 통과했지만 이를 한컴 PDF
시각 동등성의 새 증거로 확대 해석하지 않는다.

## 최신 devel 호환성과 잔여 위험

작성 시점 최신 `upstream/devel`은 `b6b9384ed9e679a8d143c6dcbaacef502614eb9e`이며 code candidate의
조상이다. `git merge-tree --write-tree HEAD upstream/devel` 결과
`7961db738be5244b411eebecbd9eebb1f40d81b6`은 HEAD tree와 같고 GitHub도 `MERGEABLE`, `CLEAN`을
보고한다.

- 일반 clone은 기준선에 없던 PDF blob의 압축 pack을 추가로 받는다. 로컬 증분 pack 측정값은
  32,097,140 bytes이나 GitHub wire 전송량의 불변값으로 주장하지 않는다.
- 현재 commit은 과거 원격 LFS storage와 이미 소비한 quota를 삭제하지 않는다. purge는 비용·복구
  위험을 별도로 검토할 비범위다.
- 수동 Oracle Public Advisory는 계속 비필수·advisory다. 이번 정정은 실행 가능성을 보장하지만 이를
  required release gate로 승격하지 않는다.
- 대형 PR이므로 review-only 후행 head에서도 trusted 판정, 최신 base 관계와 mergeability를 다시
  확인한다. 자동 merge나 admin 우회는 사용하지 않는다.

## 최종 판정과 다음 조건

- 판정: **승인**
- 판정 대상: code candidate `872afc8836e7976ce5e12e0ad71613c565d65e00`
- trailing 조건: 이 review·오늘할일만 추가한 최신 head에서 review-only checks 성공,
  `MERGEABLE`·`CLEAN` 및 최신 base 재확인
- merge 조건: 최신 head SHA 고정, 사용자 merge 승인, `--admin` 우회 없는 정상 2-parent merge commit
- GitHub review: self PR이므로 approve event와 reviewer 지정 없음
- merge 후: merge SHA·최신 `devel` 포함·post-merge CI와 #6695 자동 close를 확인한 뒤 승인된
  branch/worktree 정리를 수행한다.
