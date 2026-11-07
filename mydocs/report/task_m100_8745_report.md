# #8745 — 암호 문서 보안 경계 보강

## 결과

[이슈 #8745](https://github.com/edwardkim/rhwp/issues/8745)의 로컬 구현·제품 검증·전체 Rust CodeQL 분석을 완료했다.
구현 커밋은 `f922a759aa`, 기준 devel은 `e4ffabea38062e716f6c333cc950fa31b42b4dd7`이다.
전체 분석에서 발견한 기존 테스트 8개 파일의 호출·진단도 보정하고 2026-11-07 23:48 KST에
최종 분석을 확인했다. 이후 작업지시자가 PR 제출·CI 성공 후 self-review·머지를 승인했다.
[PR #8750](https://github.com/edwardkim/rhwp/pull/8750)의 code head
`a373208ba5538646cee0027b101393f6bd845f3f`에서 GitHub CI와 최종 정책 감사가 모두 성공했다.
[self-review 기록](../pr/archives/pr_8750_review.md)은 승인이다. 문서-only trailing head의 최종
검사를 확인한 뒤 머지하는 단계이며, 아직 머지 완료로 기록하지 않는다.

`OpenOptions`의 자동 Debug가 암호를 출력할 수 있던 결함을 명시적인 가림 처리로 보완했다.
운영 로그에서 실제 유출이 관측됐다는 주장과는 구분한다. 열기는 인증 command와 문서 조회를
분리하고 성공 뒤에만 Core를 교체한다. 실패하면 이전 문서·편집 상태가 남는다.
Native·서비스·CLI·MCP·WASM의 사용 경로에 적용했고 기존 공개 constructor와 WASM API는 유지했다.
CLI 저장 크기는 실제 쓴 파일의 같은 handle에서 얻는다.

## 검증

| 항목 | 실제 결과 |
| --- | --- |
| 집중 회귀 | 11건 통과. Debug 비노출, 실패 원자성, 호환 API, 실제 CLI 저장 재열기, PDF R2–R6 인증, HWP5 지연 이미지 복호화 |
| 전체 회귀 | 최종 테스트 수정본에서 12,831건 통과·기존 45건 skip. 변경한 8개 파일의 49건도 모두 통과 |
| Native 공식 3종 | 라이브러리·workspace unit 4,040건 + placeholder 2건 + direct PDF 35건 = 4,077건 통과 |
| Rust lint·build | fmt, root/WASM32/workspace all-target Clippy, workspace build, suite manifest 통과 |
| 출력 보존 | 기존 constructor와 새 command의 HWP3·HWP5·HWPX·PDF 113쪽 SVG 및 문서별 HWP/HWPX 저장 바이트 동일 |
| fresh WASM | wasm-pack·wasm-opt 완료. 28,328,828바이트, 실제 브라우저 초기화 13회의 응답 해시 일치 |
| 브라우저 | HWP3/HWP5/HWPX 암호 열기, PDF Worker 인증, HWP/HWPX 평문·암호화 저장 재열기 E2E 3개 스크립트 모두 통과 |
| 같은 로컬 CodeQL의 PR 범위 | logging 107→0. 정상 추출 1,604개·문제 파일 181개·warning 831개의 전후 경로·내용·중복 수 동일 |
| 전체 Rust 범위 CodeQL | RHWP logging 48→7→0. 별도 vendor Python logging 36건 유지. 전후 4,730개 소스 파일 집합·query 설정과 추출 warning 115건 동일 |
| 실제 유출 양성 대조 | 합성 코드의 암호 직접 로그·고정 운영 credential 검출 유지 |

[명령·소스·실행 기록](../tech/investigations/issue-8745/rust-validation.json),
[WASM·브라우저 증거](../tech/investigations/issue-8745/wasm-browser-validation.json),
[로컬 CodeQL 전후](../tech/investigations/issue-8745/analysis-coverage.json),
[전체 Rust 분석](../tech/investigations/issue-8745/full-codeql-validation.json),
[최종 테스트 보정·검증](../tech/investigations/issue-8745/test-boundary-completion.json),
[기존/신규 호출의 출력 대조](../tech/investigations/issue-8745/output-parity-result.json)에 근거를 보존했다.
113쪽 대조는 호출 경계의 출력 보존이며 독립 한컴 Print 피델리티 점수가 아니다.
조판·paint·backend 계산은 변경하지 않았다.

## 남겨 둔 계약과 후속 절차

PR 범위의 공개 테스트 벡터·작업 배열 16개와 형식 관련 2개 CodeQL 결과는
[개별 판정](../tech/investigations/issue-8745/crypto-result-review.md)에 따라 보존했다.
HWP3 DES를 현대적인 암호라고 주장하거나 기존 형식의 키 유도 규칙을 임의로 바꾸지 않는다.
HWP5 지연 이미지 로더는 암호를 메모리에 보관할 수 있다. Debug 가림은 모든 복제본의
zeroization이나 호출자가 암호 필드를 직접 출력하는 경우의 보호를 보장하지 않는다.

[전체 범위 결과](../tech/investigations/issue-8745/full-result-review.md)에 vendor Python 36건,
qcms 포인터 테스트 6건, 공개 벡터·작업 배열·형식 관련 나머지 결과를 구분했다.
전체 분석의 정상 추출 4,610개·문제 보고 85개는 전후 동일하다. 추출·타입 해석 한계가 있으므로
저장소 전체의 모든 보안 결함이 없다는 판정으로 확대하지 않는다.

PR·push·self-review·merge는 후속 사용자 승인에 따라 진행한다. 경고 재분류는 수행하지 않았다.
이슈 종료에 필요한 protected branch full scan은 GitHub 운영 규칙의
명시적인 후속 승인과 실행 결과를 따라 판단하며 자동 실행하거나 완료로 간주하지 않는다.

재부팅 전 완료 결과를 재사용했고, Node sandbox 접근과 생성 suite 배치 문제는 실행 경로에서
보정했다. 두 실패를 제품 결함이나 통과로 바꿔 기록하지 않았다.
[구현·실패·재개 상세](../working/task_m100_8745_implementation.md).
