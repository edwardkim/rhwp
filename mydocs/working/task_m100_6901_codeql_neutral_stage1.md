# #6901 보완 Stage 1: post-merge CodeQL 재사용 판정 일치

## 분석

- 재현 PR: #7038, merge `8661e75af8ec5ef06efcb236ccfaa3aa06c8aa70`.
- [devel CI](https://github.com/edwardkim/rhwp/actions/runs/34601192203)는 `current-base-review-bridge-green-pr-workflow-reused`로 코드 CI `34598844280`을 재사용하고 heavy worker를 skip했다.
- [devel CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34601192197)은 `latest-pr-workflow-candidate-not-successful`로 거부하고 분석 worker를 실행했다.
- 코드 head `d2668706f7f7ac9a1c4f070e311d1ded57998100`의 [CodeQL run](https://github.com/edwardkim/rhwp/actions/runs/34598844306)과 최종 문서 head의 [CodeQL run](https://github.com/edwardkim/rhwp/actions/runs/34600878896)은 모두 success였다.
- 코드 head의 [GHAS CodeQL check](https://github.com/edwardkim/rhwp/runs/103262048486)는 completed/neutral이었다. 출력은 Python 분석 configuration 하나가 없어 PR에서 새로 유입된 alert를 판단할 수 없다는 경고다. 이를 모든 언어의 alert 비교가 정상 완료됐다는 증거로 해석하지 않는다.
- PR `codeql.yml` preflight는 각 언어 Analyze 성공을 별도로 요구하면서 GHAS `success`와 `neutral`을 허용한다. post-merge 후보 수집은 GHAS `success`만 허용해 같은 성공 후보를 제외했다. 이후 과거 취소 head `ae38528acf9711869457a26d67e29cbbaf896247`의 run `34598002635`에 도달해 취소 후보 사유가 최종 진단으로 출력됐다.

## 이슈 관련성과 범위

- [#6901](https://github.com/edwardkim/rhwp/issues/6901)의 post-merge 재사용 신뢰 계약과 같은 기능 영역이다. 기존 fork duration artifact 미발행 문제와는 원인이 다르며, 이번 사례는 same-repository PR이다.
- [추가 재현 댓글](https://github.com/edwardkim/rhwp/issues/6901#issuecomment-5634815793)을 게시하고 API body 재조회를 완료했다. 사용자의 전체 개선 지시에 따라 아래 fork duration 발행·소비 보완도 같은 작업에 포함한다. 실제 후속 fork PR 실증 전에는 #6901 전체 완료를 주장하거나 close하지 않는다.
- 초기 CodeQL 보정 대상은 `.github/workflows/trusted-postmerge-ci-reuse.yml`의 full 후보 수집이다. 전체 개선 범위에는 아래에서 확정한 duration 발행·소비와 최신 rerun 거부를 추가하며, 제품 소스와 기존 SHA/tree/PR 신뢰 조건은 완화하지 않는다.

## 수정 계약

- workflow 자체 completed/success 및 CodeQL preflight와 JavaScript/TypeScript·Python·Rust Analyze 전부 success라는 기존 선행조건을 유지한다.
- GHAS 발행자, CodeQL check 이름, exact candidate SHA, 최신 check 선택, 해당 run 시작 이후의 check 시각과 completed 조건을 유지한다.
- 위 조건을 충족한 GHAS conclusion만 PR preflight와 동일하게 success/neutral을 허용한다. skipped, failure, cancelled, pending, 누락·과거 attempt의 check는 계속 거부한다.
- neutral을 수용할 때 실제 check ID를 로그에 남겨 운영자가 원 경고를 추적할 수 있게 한다. check 상태를 success로 바꾸거나 경고를 숨기지 않는다.
- 이미 시작한 devel run에는 소급 적용되지 않는다. 개선 PR 자체는 CI enforcement 변경이므로 full 검증이 예상된다. devel 반영 뒤 후속 PR에서 reuse=true와 CodeQL Analyze worker skip을 실증해야 한다.

## 결과 및 검증 상태

- CodeQL 판정 보정을 반영했다. 아래 전체 잔여 분석에 따른 duration·rerun 보정도 같은 작업 단계에 반영했다. 테스트 통과나 배포 완료를 의미하지 않는다.
- 이번 요청은 코드 수정이며, 아직 계약 테스트·실제 수정본 run을 실행하지 않았다. 기존 #7038 성공 결과를 수정본 검증으로 재사용하지 않는다.
- 후속 검증은 성공/neutral의 정상 조건, 언어 worker 실패·skip, GHAS 실패·skip·pending·누락, SHA/발행자/시각 불일치, 최신 취소 run 거부를 포함해야 한다.

## 전체 잔여 범위 추가 분석

- `run-nextest-archives.yml` 117행 이후는 여전히 same-repository PR만 duration을 업로드한다. upstream에서 실행된 정상 fork PR도 read-only 권한으로 측정 artifact를 발행하도록 변경한다. fork 자체 저장소의 실행과 metrics branch 쓰기는 허용하지 않는다.
- 기존 duration JSON은 run/ref/sha가 세 파일 사이에서 같은지만 검사한다. API가 선택한 run·attempt·PR·저장소·head·실제 테스트 merge SHA와의 결합, ZIP/JSON 크기·중복·경로·수치 경계 검증을 추가해야 한다.
- artifact 이름에 attempt를 추가하고, fork에는 새 provenance 필드를 필수로 요구한다. 기존 same-repository의 attempt 1 자료는 기존 형식의 엄격한 run/ref/merge SHA 검증을 거쳐 제한적으로 호환한다. 오래된 fork 또는 이전 attempt 자료는 거부한다.
- 비신뢰 원본 ZIP은 actions:read/contents:read verifier에서만 제한된 크기로 읽는다. 경로·추출을 허용하지 않고 JSON 한 파일만 검사한다. 검증된 세 JSON은 현재 run/attempt의 정규화 artifact로 다시 발행한다. contents:write refresh job은 이 정규화 자료만 받는다.
- 새 신뢰 helper도 enforcement surface에 포함한다. 개선 PR이 자신의 새 verifier를 신뢰하는 일을 방지하도록 이전 devel 부모의 helper만 로드하고, 미배포 상태에서는 full lane으로 fallback한다.
- 실패·취소된 최신 후보를 더 오래된 성공으로 우회하지 않는다. duration 누락·변조와 CodeQL 증거 거부의 상세 원인을 후보 run 단위로 로그에 남긴다.
- 구현 완료와 검증 완료를 구분한다. 변경 계약 테스트와 실제 fork full/trailing/merge의 B/C/D 발행·CI skip·refresh·CodeQL/Adapter/Proptest 실증은 별도 완료 gate다.

## 전체 범위 코드 보정 결과

- upstream의 devel 대상 PR은 fork도 B/C/D duration을 발행할 수 있게 했다. 이름과 JSON에 run attempt를 결합하고 PR·head repository·head SHA/ref를 기록한다. worker는 contents:read를 유지한다.
- 신뢰 helper `scripts/trusted-postmerge-duration-evidence.mjs`를 추가했다. API artifact ID·run·저장소·head·생성 시각·만료·크기, 세 archive의 완전성, ZIP 단일 JSON·경로/링크/암호화/크기·중복 JSON key, report schema·provenance·수치/식별자·중복 target/case를 검사한다.
- 선택한 원본은 read-only verifier에서만 다운로드·검사하고 정규화된 JSON을 현재 run/attempt의 새 artifact로 발행한다. 발행이 실패하면 reuse=false로 마감한다. write-capable refresh job은 정규화된 현재 run artifact만 읽고 원본 fork ZIP을 직접 추출하지 않는다.
- 기존 same-repository attempt 1의 legacy report는 검증된 run/ref/tested merge SHA와 세 archive 내용 검사를 거쳐 지원한다. fork의 legacy/이전 attempt 자료는 허용하지 않는다. policy의 measurement_sources에도 새 provenance를 보존한다.
- 후보 run 선택에서 merge 이후 updated_at이라는 이유로 최신 rerun을 목록에서 먼저 버리지 않도록 했다. 최신 실패·취소·pending 및 merge 이후 완료된 성공은 거부하고 과거 성공으로 우회하지 않는다.
- 다음 미완료 항목: 기존/신규 계약 테스트 갱신 및 실행, PR CI, 개선이 배포된 devel에서 실제 fork full/trailing/post-merge 재사용 실증. 이 항목들이 남아 있으므로 #6901은 OPEN 유지다.

## 계약 테스트 검증 결과

앞 절의 테스트 미실행·대기 상태를 아래 결과로 갱신한다.

- JavaScript 계약 테스트: 318개 통과, 실패 0개, 건너뜀 0개.
- Python workflow 계약 테스트: 109개 통과, 실패 0개.
- 실제 reusable workflow의 github-script를 실행하고 Git 객체와 ZIP을 사용해 fork full 실행 → 문서 trailing commit → 일반 merge 경로를 검증했다. GitHub API와 네트워크 호출은 모의 응답이므로 실제 Actions 배포 검증과 구분한다.
- CI, CodeQL, Adapter inter-diff, Proptest의 fork 재사용 허용 경로를 확인했다. 저장소·head·attempt 불일치, 만료·누락·변조 artifact, 안전하지 않은 ZIP, 최신 실패·취소·대기 실행은 거부한다.
- CodeQL의 모든 언어 Analyze 성공을 요구한 상태에서 동일 SHA의 GHAS neutral을 허용한다. neutral의 configuration warning 자체를 해결했다고 주장하지 않는다.
- 테스트 작성 중 fork Git fetch 인자를 same-repository bridge 인자로만 제한한 모의 환경 오류를 수정했다. 운영 검증 조건을 완화하지 않았다.
- Rust 제품 코드 변경이 아니므로 Cargo 전체 회귀 및 WASM 빌드는 수행하지 않았다.

### 남은 실제 검증

변경 배포 후 실제 fork PR의 full → 문서 trailing → merge에서 heavy worker skip, 검증·정규화한 B/C/D duration 전달 및 refresh, CodeQL/Adapter/Proptest 결과를 확인해야 한다. 이 검증 전에는 #6901 전체 해결이나 종료를 선언하지 않는다. 현재 변경은 아직 commit/push하지 않았다.
