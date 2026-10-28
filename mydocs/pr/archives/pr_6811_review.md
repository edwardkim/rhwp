# PR #6811 검토

## 판정: 승인

메인터너 보정이 완료된 코드 후보와 동일한 파일 트리의 최신 devel 병합 결과를 승인한다.
원 기여자 head 자체를 승인한 것이 아니다. 문서 trailing head의 최신 CI와 병합 직전 상태는
별도 확인하며, 이 기록은 병합 완료나 CI 재사용 이슈의 해결 선언이 아니다.

## 검토 대상

| 항목 | 내용 |
| --- | --- |
| PR | [#6811: 공백 없이 붙은 글꼴 명령 분리](https://github.com/edwardkim/rhwp/pull/6811) |
| 기여자 | zer0bi9, GitHub FIRST_TIME_CONTRIBUTOR |
| 대상 | devel |
| 원 기여자 head | `3a02b30046a7b6413d17ef50286befb8033b4a9a` |
| 메인터너 보정 및 녹색 코드 후보 | `7be27c47cb7ffe4d52bdb0e4900fa8f2a963077f` |
| 최신 devel 기준 | `13c92feb67d2bf0ae62349f41c5f5cd83845a4a5` |
| 현재 base 병합 bridge | `f26673870bd57ffc920abf9595fed15115fa92d0` |
| GitHub 검사 merge | `b7d114081667711fb1a81f25ef2141d1d1aae0f0` |
| 두 merge의 동일 tree | `d8dc8e250d1e68cc649f471f8e708d4093ca6c98` |
| PR 고유 변경 | EqEdit 렉서와 integration 테스트 2개 파일, 120줄 추가·2줄 삭제 |
| 상태 | 코드 후보 CI 완료 시 MERGEABLE / CLEAN, 문서 head에서 재확인 필요 |

기본 경로는 collaborator 매개 외부 PR이다. 접수·로컬 검증·첫 기여자·review-only fast-pass·
merge 후속 처리 절차를 적용한다. 작성자 원 commit은 재작성하지 않았고 보정은 별도 commit이다.

## 문제와 메인터너 보정

원 구현은 `rmC`, `itx`, `boldv` 등 공백 없는 글꼴 명령을 인식하는 개선이었다.
원 head에는 다음 문제가 있었다.

1. 기존 source-side 단위 테스트에 두 개를 추가해 CI의 PR base 대비 테스트 수 제한에 걸렸다.
   전체 `4207 > 4205`, EqEdit 모듈 `49 > 47`이었다. 두 테스트를 삭제하지 않고
   `tests/cases/pr_6811_eqedit_font_prefix.rs`로 옮겨 공개 API 회귀로 보존했다.
2. 접두사를 제거한 나머지를 단순 식별자로 분류해 중첩 명령과 숫자가 공백 형태와 다른 경로를 탔다.
   분리를 렉서로 옮겨 나머지 입력이 기존 숫자·구조 명령·글꼴 파서를 거치도록 했다.
3. `rmboldv`, `bolditx`, `rmsqrt{x}`, `rmbar{x}`, `rm12.5`, `bold12x`, 위첨자와
   대소문자 변형을 공백 형태의 LaTeX 및 degraded 결과와 비교했다.
   기존 명령·한글·Unicode 단어 보존 검사도 추가했다.

허용 접두사는 `bold`, `rm`, `it`로 제한했다. 일반 명령 전체로 접두사 집합을 넓히지 않았으며,
기여자의 일반 단어 비분리 검사와 단위·화학식 회귀를 유지했다. 테스트 수 기준선과 CI 정책은 완화하지 않았다.

## 실제 검증 결과

로컬 환경은 macOS, 고정 Rust 1.93.1이며 전용 target은
`target/pr6811-eqedit-maintainer-20260909`였다. Cargo 작업은 순차 실행했다.

| 검증 | 결과 |
| --- | --- |
| source-side 테스트 정책 | 4205 tests, 298 modules로 통과 |
| 파생 suite 준비·manifest 검사 | 통과, generated 파일은 커밋하지 않음 |
| cargo fmt 검사 | 통과 |
| native Clippy | 통과 |
| WASM32 lib Clippy | 통과 |
| workspace build | 통과 |
| workspace all-targets Clippy | 통과 |
| EqEdit 집중 nextest | 77개 통과, 8 threads |
| 이동·보강한 integration 회귀 | 4개 통과, 8 threads |
| 최신 devel 병합 | 자동 병합 성공, GitHub 검사 merge와 전체 tree 동일 |

포맷 후 generated harness drift는 파생 suite 재생성으로 해소했다. 제품 코드나 검사 기준선을
변경하지 않았다. nextest의 `junit.report-skipped` 미지원 경고는 있었지만 두 집중 실행 모두 종료 코드 0이었다.

코드 후보 `7be27c47cb7ffe4d52bdb0e4900fa8f2a963077f`의 원격 결과는 다음과 같다.

- [CI 34341919474](https://github.com/edwardkim/rhwp/actions/runs/34341919474): lint, Archive A/B/C/D 실행 worker 및 Build & Test 성공.
- [CodeQL 34341919580](https://github.com/edwardkim/rhwp/actions/runs/34341919580): Rust 분석을 포함한 실행 check 성공.
- [Adapter 34341919438](https://github.com/edwardkim/rhwp/actions/runs/34341919438): 실행 성공.
- [Proptest 34341919551](https://github.com/edwardkim/rhwp/actions/runs/34341919551): 실행 성공.
- CI Impact Policy 성공. Native Skia·frontend·독립 WASM Build는 영향 분류상 skip이며 실행 성공으로 세지 않는다.

전체 Rust 회귀는 이번 로컬 실행에서 반복하지 않았다. 위 코드 후보의 GitHub Archive A/B/C/D
성공을 근거로 사용한다. WASM Clippy를 실제 WASM 패키지 빌드나 브라우저 검증으로 표현하지 않는다.

## 시각 증적 필요 여부

이번 변경은 DocLang EqEdit → LaTeX 변환 경로이며 renderer/layout/Studio 출력과 sample·PDF를
바꾸지 않았다. LaTeX·degraded 결과를 검사하는 결정적 회귀를 직접 증거로 사용했다.
별도 PDF·PNG·visual sweep은 생성하거나 검증하지 않았다. 기여자가 본문에 기재한 비첨부 코퍼스
1,982건의 측정은 기여자 보고이며 메인터너가 재실행한 결과로 계산하지 않는다.

## 오늘할일과 문서 trailing 처리

원 source에는 `mydocs/orders/20260909.md`가 없고 최신 devel에는 존재했다.
단순 신규 추가의 add/add 충돌을 확인한 뒤 사용자 승인에 따라 최신 devel을 한 번 병합했다.
bridge는 자동 병합이며 수동 source·test·workflow 충돌 해소는 없었다.
현재 CI가 검사한 merge와 bridge tree가 같음을 확인한 뒤 기존 오늘할일에 이번 PR 항목만 추가했다.
review·오늘할일 trailing commit에는 코드·test·workflow·sample·파생 산출물을 넣지 않는다.

## #6901·#6815 검증 범위

- [#6901](https://github.com/edwardkim/rhwp/issues/6901): fork PR의 정확한 녹색 코드 후보,
  current-base bridge와 review tail을 후속 실증에 사용한다. 그러나 현재 archive workflow에는
  B/C/D duration artifact 게시의 same-repository 조건이 남아 있다. PR CI 성공만으로 fork
  post-merge CI 재사용 완료라고 판단하지 않는다. 병합 후 verifier 사유·worker·duration refresh를 확인한다.
- [#6815](https://github.com/edwardkim/rhwp/issues/6815): 이번 계보에는 한 번의 current-base
  자동 병합 bridge와 문서 tail이 생긴다. 자동 bridge를 지나 코드 후보를 찾는 경로는 검증 후보지만,
  수동 mydocs 충돌 해소 및 squash 재현 자체는 아니다. 병합 후 CI/CodeQL의 후보 선택과 heavy skip을
  확인해야 하며, 이 사례 하나를 전체 완료 기준 충족으로 확대하지 않는다.
- 두 이슈는 이 PR의 closing 대상이 아니다. 실제 증거와 완료 기준을 확인하기 전에는 close하지 않는다.

## 병합 전·후 처리 계획

1. trailing 문서의 상대 링크·diff와 최신 devel merge simulation을 확인한다.
2. 같은 fork branch로 일반 push하고 최신 head의 CI·required checks·MERGEABLE·CLEAN을 확인한다.
   fast-pass가 거부되면 Full CI를 기다린다.
3. 승인된 일반 merge commit으로 병합하고 devel을 fast-forward한다.
4. merge SHA의 CI·CodeQL·Adapter·Proptest·Close Issues 및 duration refresh 결과를 확인한다.
   재사용 성공과 Full 실행 성공을 분리해 기록한다.
5. 실패·취소·증거 누락이면 완료로 보고하거나 관련 이슈를 닫지 않는다.

## Merge 후 contributor PR comment 계획

원 PR에 첫 기여 감사와 환영을 명시하고, 기여자의 공백 없는 글꼴 명령 지원과 메인터너의
렉서·테스트 위치 보정을 구분한다. 실제 merge SHA, 코드 후보·trailing head·devel의 Actions
링크, 로컬 집중 81개와 lint 결과를 적는다. 이 리뷰는 merge SHA 고정 blob URL로 연결한다.
시각 검증은 미실시·비대상으로 명시하고 존재하지 않는 이미지나 전체 코퍼스 검증을 주장하지 않는다.
기존 동일 증적 comment가 있으면 수정하고 중복 게시하지 않는다. UTF-8 body file로 전송하고 API로 확인한다.
contributor fork branch는 보존하며, 이번 로컬 검토 branch와 전용 target만 종료 조건을 확인한 뒤 정리한다.
