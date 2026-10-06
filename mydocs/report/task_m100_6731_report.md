---
kind: report
status: final
canonical: mydocs/report/task_m100_6731_report.md
issue: 6731
last_verified: 2026-09-05
---

# #6731 CodeQL alert #186 근거화·재발 방지 최종 보고서

## 1. 판정

PR [#6748](https://github.com/edwardkim/rhwp/pull/6748)의 code candidate
`751b4c5fc67f4e63c147a2aa7f3acced928839be`는 password open command와 metadata query의 반환 경계를
분리했고, 기존 암호 문서 lifecycle과 raw password 무유출 계약을 유지했다. 정확한 PR merge tree를 분석한
JavaScript/TypeScript CodeQL 결과는 87개 rule에서 0건이므로 alert #186의 기존 네 flow는 제거됐다.

메인테이너는 2026-09-05 이 결과를 승인했다. Issue #6731은 PR merge 뒤 `devel` full CodeQL에서 같은
flow가 재발하지 않고 실제 password source 탐지 범위를 제외하지 않았음을 확인한 다음 닫는다.

## 2. 원인 계보와 기준선 증적

alert #186(`js/insufficient-password-hash`)은 제품의 password hashing이 아니라 테스트·계측용 폰트 규칙
snapshot JSON의 SHA-256 무결성 digest를 sink로 지적했다.

| 구간 | 확인 결과 |
| --- | --- |
| 암호 open 도입 | `00c1f361ab5`, 2026-07-27 |
| 범용 `sha256Text()` 도입 | `795e7b5fac2`, 2026-08-16 |
| 폰트 runtime snapshot rows·hash 도입 | `b299e1650e9`, 2026-08-23 16:54 KST |
| alert 최초 등록 | analysis `1659063897`, `devel@5057a7fcaf0`, 2026-08-24 00:04:25 KST, CodeQL 2.26.3 |
| 변경 전 현재 재현 | analysis `1725324233`, `devel@d1831146587`, 2026-09-04, CodeQL 2.26.4 |

최초·현재 분석은 모두 네 flow와 각 60~64개 location을 가졌고, 줄 이동을 제외한 파일·message topology
SHA-256은 `e05bfecb52277f618bb35577d62e2a0997d0cb8f5639b368e56c81c93eb410f5`로 같았다.

정규화 증적은 다음 경로에 보존했다.

- [조사 README](../tech/investigations/issue-6731/README.md)
- [최초 data-flow](../tech/investigations/issue-6731/alert_186_first_dataflow.json)
- [변경 전 현재 data-flow](../tech/investigations/issue-6731/alert_186_current_dataflow.json)

증적에는 raw password·token·secret 값, 사용자 절대 경로와 private corpus 식별자가 없다. alert의 최종
분류는 메인테이너가 결정한 `dismissed` / `used in tests`를 유지했다.

## 3. 실제 원인

CodeQL의 민감 함수 이름 휴리스틱은 `loadDocumentWithPassword()`의 반환값을 password source로
분류했다. 실제 반환된 `DocumentInfo`에는 raw password가 없지만 `fontsUsed`가 Studio의 폰트 상태 분석과
module-global 해소 cache를 지나간다. 정적 분석은 브라우저 Studio와 별도 Node 계측 프로세스를 하나의
실행 세계로 연결해 snapshot의 `resolvedFace`와 SHA-256까지 flow를 구성했다.

따라서 SHA-256을 KDF로 바꾸거나 CodeQL path를 제외하는 것은 원인을 해결하지 않는다. 반환값으로 상태
변경과 metadata 전달을 동시에 표현한 API 경계가 오귀속의 시작점이었다.

## 4. 구현한 command/query 경계

1. `openPasswordProtectedDocument()`는 성공·실패만 표현하는 `Promise<void>` command다.
2. `WasmBridge.loadDocumentWithPassword()`와 공통 `loadDocumentAtomically()`도 metadata를 만들거나
   반환하지 않는 `void` command다.
3. 암호 문서는 command 성공 뒤 `loadDocumentForOpen()`이 `getDocumentInfo()` query를 한 번 호출한다.
4. 기존 평문 `loadDocument()`도 공통 command 성공 뒤 같은 query를 한 번 호출해 public 반환 계약을
   유지한다.
5. CodeQL workflow·query·path, font cache와 snapshot SHA-256 구현은 바꾸지 않았다.

PR 생성 직전 공통 helper가 여전히 `DocumentInfo`를 만들어 버리던 첫 구현의 허점을 발견해 code
candidate 전에 제거했다. 최종 구조는 metadata query를 중복 실행하지 않는다.

## 5. 보호 불변식 결과

| 보호 불변식 | 판정 근거 |
| --- | --- |
| raw password는 open 시도 지역 범위를 벗어나지 않는다 | source contract와 세 형식 browser storage·metadata 검사 통과 |
| command 반환값이 metadata 운반자가 아니다 | 외부 helper·WasmBridge·공통 atomic helper 모두 `void` |
| `DocumentInfo`에 credential 필드가 없다 | 승인된 8개 필드 exact 계약 통과 |
| font cache key에 credential이 없다 | `langId`, `fontName`, `altType` projection 계약 통과 |
| snapshot digest는 canonical font rows만 hash한다 | Node import·입력 projection 계약 통과 |
| 실제 CodeQL 탐지를 query 제외로 약화하지 않는다 | workflow·query·path diff 0, PR JavaScript 분석 87 rules 유지 |
| 암호 문서 lifecycle을 유지한다 | HWP3·HWP5·HWPX 취소·오입력·성공·보호 저장 E2E 통과 |

## 6. 검증 결과

### 로컬

| 명령 | 결과 |
| --- | --- |
| `node --test rhwp-studio/tests/hwp-password-open.test.ts` | 7/7 성공 |
| `npm --prefix rhwp-studio test` | 1,376건, 성공 1,375·실패 0·기존 skip 1 |
| `npm --prefix rhwp-studio run build` | TypeScript·Vite production build 성공 |
| `npm --prefix rhwp-studio run e2e:hwp-password-open` | HWP3·HWP5·HWPX 전체 성공 |
| `npm --prefix rhwp-studio run e2e:issue-4430-content-loss` | 전체 성공 |
| `git diff --check`·변경 문서 링크 검사 | 성공 |

#4430 E2E에서 발견한 timeout은 2026-08-30 통합 `2ca9aa90f7`이 보호 Save As 기본 동작을 암호
계승으로 바꾸면서 평문 사본 test를 현행화하지 않은 기준선 결함이었다. 해당 시나리오가 현재 UI 계약의
`암호 없이 저장`을 선택하도록 test 한 곳만 정정했고 전체 여정이 통과했다.

Rust source·Rust test·WASM binding과 renderer·layout·fixture는 바꾸지 않아 Rust lint·release-test·WASM
build와 visual sweep은 로컬 변경 범위상 비대상이다.

### PR exact head와 CodeQL

- code candidate: `751b4c5fc67f4e63c147a2aa7f3acced928839be`
- base: `9bf5bcfd061b491c89be1ea28ef1fff8a892b6d1`
- merge ref commit: `a547f5ae51e5c101d98c4618f610f73ef5d9eb36`
- merge ref parents: 위 base와 code candidate 두 SHA
- CodeQL run: [33933481795](https://github.com/edwardkim/rhwp/actions/runs/33933481795)
- JavaScript/TypeScript analysis: `1727960739`, `results_count=0`, `rules_count=87`, error 없음
- `refs/pull/6748/merge` open code-scanning alert: 0건
- CI run: [33933481838](https://github.com/edwardkim/rhwp/actions/runs/33933481838), 성공
- Render Diff·Proptest roundtrip·Adapter inter-diff: 모두 성공

Rust·Python Analyze job은 성공 상태로 종료됐지만 내부 분석 step은 영향 언어가 아니어서 skip됐고,
JavaScript/TypeScript Analyze는 실제로 2분 43초 실행됐다. 따라서 workflow가 no-op으로 성공한 결과를
flow 소멸 근거로 사용하지 않았다.

## 7. 운영 절차 반영

[GitHub 저장소 운영 매뉴얼](../manual/github_operations.md#94-codeql-alert-귀속과-used-in-tests-근거-보존)에
다음 재발 방지 절차를 추가했다.

- source/sink 도입, 최초 flow 성립, 최초 분석 등록, 현재 PR 귀속 시점을 분리한다.
- 최초·현재 SARIF의 모든 location과 도구·SHA·ref를 정규화 증적으로 보존한다.
- `used in tests`는 sink 역할, 민감값 projection, runtime 경계와 탐지 비약화가 모두 증명될 때만
  메인테이너가 선택한다.
- PR exact merge-ref SARIF와 merge 뒤 protected branch full scan을 모두 확인한 뒤 이슈를 닫는다.

## 8. 잔여 위험과 종료 절차

- CodeQL data-flow는 분석기 version에 따라 다시 확장될 수 있다. 재발하면 기존 분류만 재사용하지 않고 새
  analysis와 보존한 topology를 비교한다.
- PR 분석은 merge tree의 결과다. 실제 `devel` merge SHA의 branch analysis가 같은 language·query로
  성공하기 전에는 branch 상태를 확정하지 않는다.
- 동일 파일을 변경하는 PR #6725·#6637이 먼저 병합되면 merge 직전 최신 `devel` 정합과 exact head CI를
  다시 확인한다.
- report·self-review·오늘할일만 추가하는 trailing commit은 녹색 code candidate의 review-only
  fast-pass 조건과 최신 aggregate를 별도로 확인한다.

종료 순서는 review-only trailing head 검증, 정상 merge commit, `devel` full CodeQL, alert #186 비재발과
탐지 설정 유지 확인, Issue #6731 결과 comment·close, local·remote task branch 정리다.
