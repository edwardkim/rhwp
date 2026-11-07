# PR #8750 self-review — 암호 문서 인증·진단 경계

## 최종 판정

**승인 — code head `a373208ba5538646cee0027b101393f6bd845f3f`의 필수 로컬 검증과 GitHub CI가 모두 통과했다.**
작업지시자가 PR 제출과 CI 성공 후 self-review·머지를 승인했다. 검토 기록 작성 시점은
2026-11-08 00:37 KST이며, 문서-only trailing head의 실제 CI 재사용·required check·최신 base
병합 가능 여부를 확인한 뒤 일반 머지로 진행한다. 이 문서는 아직 실행하지 않은 머지를 완료로 표시하지 않는다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#8750](https://github.com/edwardkim/rhwp/pull/8750), edwardkim, devel |
| 관련 이슈 | [#8745](https://github.com/edwardkim/rhwp/issues/8745), `Refs`로 연결하며 protected branch full scan 전에는 OPEN 유지 |
| branch·경로 | `task/8745-password-document-security`, collaborator self-merge 경로 |
| 로컬 제품 검증 기준 | `e4ffabea38062e716f6c333cc950fa31b42b4dd7` |
| 구현·최종 code head | 제품 구현 `f922a759aa`, 기존 테스트 호출 보정·최종 증거 `a373208ba5538646cee0027b101393f6bd845f3f` |
| 실제 CI base | `1e16a19aede5ccb2657d35974940eb596286cdf1` |
| GitHub 병합본 | `7fda4c8e8fb03eb47e3713695592006a113a1dc5`, tree `7ccb386565a67458e7a2af394e26feda9377c3d3` |
| reviewer·상태 | reviewer 미지정, Open·non-draft. code head 전체 check 성공·MERGEABLE/CLEAN 확인 |

`collaborator_self_merge`, `intake_and_review`, `local_validation`, `review_only_fast_pass`,
`post_merge`와 보안 경고 귀속 절차를 적용했다. 생성 전 본인 계정의 다른 Open PR은 없었다.
시각 증적을 요구하는 조판·렌더링 변경은 아래 근거로 비해당이다.

## 변경과 검토

- `DocumentCore::load_bytes`는 기존 `from_bytes_inner`가 성공한 뒤에만 `*self`를 교체한다.
  인증·파싱 실패 시 문서와 편집 상태를 보존하고, 성공한 문서는 기존 조회 API로 사용한다.
  원래 공개 constructor와 JavaScript `openWithPassword`의 시그니처·오류 변환은 유지했다.
- Native·서비스·CLI·MCP·WASM 호출부의 성공·실패 경로를 대조했다. 빈 Core를 래퍼로 옮기는
  `From<DocumentCore>`를 사용해 불필요한 빈 A4 문서 재조판을 추가하지 않았다.
- `OpenOptions`의 Debug는 일반·pretty·중첩 출력에서 암호 값을 가린다. `None`과 공개 자원 한도는
  진단할 수 있다. HWP5 이미지 지연 복호화 resolver의 암호 보관 수명은 유지되며 전역 zeroization을 주장하지 않는다.
- CLI 저장 바이트 수는 실제로 쓴 같은 파일 handle의 metadata에서 얻는다. 경로 재조회 경쟁을
  피하며 metadata 실패를 0바이트 성공으로 숨기지 않는다.
- `load_bytes`를 WholeDocument 교체로 분류한 패스스루 가드와 실패 원자성 회귀를 확인했다.
  가드를 완화하거나 CodeQL query·path·severity·workflow·vendor 코드를 바꾸지 않았다.
- 기존 테스트 8개 파일의 호출 경계와 두 실패 메시지를 보정했지만 입력·assertion·기대값은 유지했다.
  기존 constructor 호환·feature-off 오류 검증 세 경로도 유지했다. 합성 운영 credential과 직접 암호
  로그의 양성 대조는 같은 분석 설정에서 계속 검출됐다.

제품 diff는 인증·조회·진단 경계다. 파서의 문서 생성 결과, IR 조판 속성, 줄 계획·페이지 배치·paint·
backend 수식은 바꾸지 않았다. 따라서 조판 원칙의 새 좌표·분할 계약, 편람 383쪽 gate와 독립
Print 기반 Visual Sweep은 비해당이다. 같은 후보에서 기존 constructor와 새 command의 113쪽 SVG·
저장 바이트를 대조한 결과를 한컴 피델리티나 fresh WASM 픽셀 일치율로 대신 표시하지 않는다.

## 입력과 로컬 검증

[최종 구현 보고서](../../report/task_m100_8745_report.md)와
[원 실행·소스 해시](../../tech/investigations/issue-8745/rust-validation.json),
[최종 테스트 보정 증거](../../tech/investigations/issue-8745/test-boundary-completion.json)를 대조했다.
공개 HWP3·HWP5·HWPX와 PDF R2–R6 인증 입력 11개의 경로·크기·SHA-256·Git blob은
[입력 원장](../assets/pr8750/input-manifest.json)에 있다. 모두 code head에 포함돼 있으며
외부 비공개 corpus나 새 글꼴 파일을 제출하지 않았다.

| 항목 | 실행 결과 |
| --- | --- |
| 전체 회귀 | 최종 테스트 수정본 12,831 passed, 기존 45 skipped |
| Native 공식 검사 | 4,077 passed: 라이브러리·workspace 4,040 + placeholder 2 + direct PDF 35 |
| 집중·변경 파일 | 집중 11건, 최종 변경 파일 49건 통과 |
| Rust lint·build | fmt, Native/WASM32/workspace all-target Clippy, workspace build, suite manifest 통과 |
| 출력 보존 | HWP3·HWP5·HWPX·PDF 113쪽의 constructor/command SVG 및 HWP/HWPX 저장 바이트 동일 |
| fresh WASM | 최적화 28,328,828바이트, SHA-256 `c01a7cc9afadcd9fec29e65cbd3dcbbd5520e3e6230a49f0f88c714fda7a737c` |
| 실제 브라우저 | HWP 암호 열기, PDF Worker 인증, 평문·암호화 저장 재열기 E2E 3개 스크립트 성공 |

[브라우저 실행 원장](../../tech/investigations/issue-8745/wasm-browser-validation.json)은 초기화 13회의
실제 WASM 응답 해시를 검증했다. 별도 해시를 측정하지 않은 Worker 응답까지 측정했다고 표시하지 않는다.
제품·신규 집중 테스트가 바뀌지 않은 이전 실행은 소스 해시로 재사용했고, 최종 수정된 기존 테스트와
필수 Rust lint는 재실행했다. 이번 원격 검토에서는 성공한 로컬 전체 검사를 중복 실행하지 않았다.

## 최신 devel과 원격 CI

새 base의 [#8738](https://github.com/edwardkim/rhwp/pull/8738)은 표·셀 단색 배경 기본값과 저장 보존을
추가했다. 해당 Fill 생성·UI 편집 경로는 이번 인증·진단 diff와 겹치지 않는다. source를 rebase하거나
재작성하지 않았고, 최신 base suite 정책 검사와 실제 병합 tree의 CI로 검증했다.
[로컬 merge-tree와 GitHub 병합본](../assets/pr8750/merge-identity.json)의 tree·두 parent가 일치한다.

| 검사 | 해당 code head 결과 |
| --- | --- |
| [CI](https://github.com/edwardkim/rhwp/actions/runs/42383939527) | Lint·Frontend·Native·Archive A–E·Build & Test 성공 |
| [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/42383939572) | Rust·Python·JavaScript/TypeScript·Actions 전부 성공 |
| [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/42383939570) | 성공 |
| [Adapter](https://github.com/edwardkim/rhwp/actions/runs/42383939641) | 성공 |
| [Proptest](https://github.com/edwardkim/rhwp/actions/runs/42383939492) | 성공 |
| [최종 정책 감사](https://github.com/edwardkim/rhwp/actions/runs/42387007746) | CI Impact Policy success |

[실제 job 로그 요약](../assets/pr8750/ci-test-results.json)의 Archive 합계는 **12,640 passed·45 skipped**,
선택 Native는 **234 passed**다. Studio 단위 검사는 **2,754 passed·2 skipped**, 후속 브라우저 검사는
**2,666 passed·0 failed**다. 로컬 전체·Native 공식 검사와 원격 선택 범위의 숫자를 구분한다.
B의 checkout이 길었으나 같은 실행에서 성공했고, 수동 취소·재실행은 없었다.
[exact-head checks](../assets/pr8750/code-head-checks.json)와 [run 원장](../assets/pr8750/code-head-runs.json)을 보존한다.

## 보안 분석 결과와 한계

GitHub Rust analysis `2118500108`은 위 PR 병합본, CodeQL 2.27.4, 23개 규칙이며
API `results_count=0`·error/warning 없음이다. 실제 job 로그에서 `rust/cleartext-logging` 실행과
정상 추출 1,604개·문제 보고 181개를 확인했다. [분석 metadata](../assets/pr8750/rust-analysis-metadata.json),
[API SARIF](../assets/pr8750/rust-analysis.sarif), [범위·출처·한계](../assets/pr8750/security-ci-summary.json)를 보존했다.
GitHub SARIF API는 [공식 문서](https://docs.github.com/en/rest/code-scanning/code-scanning#get-a-code-scanning-analysis-for-a-repository)가
설명하는 업로드 자료의 부분집합이므로 runner 원본 전체 결과라고 주장하지 않는다.

변경 효과는 같은 로컬 분석기의 [PR 범위 logging 107→0](../../tech/investigations/issue-8745/analysis-coverage.json),
[전체 RHWP logging 48→7→0](../../tech/investigations/issue-8745/full-codeql-validation.json),
양성 대조 검출 유지와 함께 판정했다. 전체 소스 집합·query 설정·추출 경고 경로/내용/중복 수를
대조했다. vendor Python 36건, qcms·legacy 형식·공개 테스트 벡터 등
[별도 검토 항목](../../tech/investigations/issue-8745/full-result-review.md)의 해소를 주장하지 않는다.
추출 한계가 남으므로 저장소 전체에 보안 결함이 없다는 판정도 아니다. 경고 분류·탐지 설정 변경은 0건이다.

## 남은 병합·후속 조건

1. 이 review·오늘할일·증거만 single-parent trailing commit으로 추가한다. 최신 base와의 실제 merge
   tree에서 공백·문서 링크·오늘할일 보존을 확인한 뒤 push한다.
2. trailing head의 실제 CI fast-pass와 정확한 재사용 code SHA, required aggregate·최종 정책·
   MERGEABLE/CLEAN을 다시 확인하고 `--match-head-commit`으로 일반 머지한다. 관리자 우회는 하지 않는다.
3. 병합 SHA·duration 갱신 결과·devel 동기화를 확인한다. 이슈 #8745는
   [보안 운영 규칙 9.4](../../manual/github_operations.md#94-codeql-alert-귀속과-used-in-tests-근거-보존)에
   따라 protected branch full scan 전까지 OPEN을 유지한다. 별도 승인이 없는 scan dispatch는 하지 않는다.
4. 사용자·다른 작업을 보존하고 전용 branch와 사용을 마친 루트 output·공유 debug를 정리한다.
   자체 작업 PR이므로 외부 contributor 감사 댓글은 비해당이다.
