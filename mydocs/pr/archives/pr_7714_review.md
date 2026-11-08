# PR #7714 검토 — 누름틀 이벤트와 문서 교체

## 최종 판정

**메인터너 보정 후 승인** — [#7709](https://github.com/edwardkim/rhwp/issues/7709) 해결 범위의
필수 로컬 검증과 통합 code head CI를 완료했다. 원 head와 보정한 통합 후보를 구분한다.
남은 원격 조건은 문서 trailing head의 CI·mergeability 확인 및 승인된 정상 merge·후속 처리다.

## 접수와 코멘트

- 원 [PR #7714](https://github.com/edwardkim/rhwp/pull/7714), 기존 기여자 `wook8170`, base devel.
- 원 head `688c896d065d68249b2ec370fb89f86cf30e9bd5`, 체리픽 `15067601e415beb27268612001ad95b2a189db9d`. 원 author와 `-x` 원 SHA를 보존했다.
- reviewer `jangster77`을 먼저 지정하고 기존 `edwardkim` 요청을 유지한 것을 API로 확인했다.
- [기존 코멘트](https://github.com/edwardkim/rhwp/pull/7714#issuecomment-6104122006)의
  지적을 실제 실행으로 재현하고 보정 전후로 확인했다. 최신 조회에서 새 코멘트·source head 변경은 없었다.

## 구현 주장·실행 증거와 조판 원칙 검토

실제 소비 경로는 `field-info-changed` → 연결별 forwarder → capability를 협상한
`fieldChanged` MessageChannel → SDK listener다. 같은 fieldId의 중복은 제거하고
다른 필드 진입·이탈은 전달한다. 기존 `documentChanged` 전송과 capability 필터도 유지한다.

기존 코멘트의 문서 reload 반례에서는 이전 `lastFieldId`가 남아 false 이탈과 같은 ID의
새 진입을 누락했다. `subscribeFieldChanges`가 `document-swapped`를 같은 forwarder의
null 입력으로 전달하고, off/destroy는 필드·문서 교체 구독을 함께 해제한다.

실제 SDK E2E에서 보정 전 이탈·같은 ID 재진입 assertion FAIL / 보정 후 PASS를 확인했다.
다른 ID 진입, 같은 필드 내 중복 제거, 밖→밖 무알림, 해지 뒤 무알림을 대조했다.
단위 4검사는 기존 `documentChanged`와 동시 배달 및 두 구독 cleanup도 확인한다.
원 envelope·schemaVersion 1·엄격한 SDK 값 검증은 유지한다.

정식 검사는 `rhwp-studio/tests/embed-field-events.test.ts`와
`rhwp-studio/e2e/embed-field-events.test.mjs`에 있다. 이벤트 전달·수명주기 계약이므로
이 PR 자체는 좌표·조판 결과를 변경하지 않는다. 통합 인쇄 보정의 시각 증거는
[PR #7715 검토](pr_7715_review.md)에 연결한다.

## 공통 검증과 실제 CI

통합 [PR #8003](https://github.com/edwardkim/rhwp/pull/8003)의 정확한 code head는 `9c1c681ee44c65eb924a66ac18b0b86876ad767b`다.
[Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39051970167) · [CI](https://github.com/edwardkim/rhwp/actions/runs/39051970186) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/39051970001) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39051970099) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39051970173) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39051970257)가 실제 완료·성공했다. 언어별 CodeQL Analyze와 GHAS check도 확인했고,
정책상 skipped·neutral은 실제 실행 성공과 구분했다. 해당 merge ref의 열린 CodeQL alert는 0건이다.

| 로컬 검증 | 실제 결과 |
| --- | --- |
| 전체 nextest release-test | 10,823 PASS / 0 FAIL / 기존 45 SKIP, 실행 1310.134초 |
| fmt·Native/WASM Clippy·workspace build·all-target Clippy | 5개 모두 exit 0 |
| Native Skia lib | 4,043 PASS / 기존 12 IGNORE |
| Skia missing-picture PNG / direct-PDF | 2 PASS / 14 PASS |
| Studio 전체 단위·타입·production build | 2,327 PASS / 기존 2 SKIP, 타입·build exit 0 |
| editor / frontend bindings / hwpctrl | 36 / 3 / 26 PASS, 선언·pack exit 0 |
| 실제 Chrome E2E | 7종 exit 0, SDK profile 9개 assertion PASS |
| manifest·unit-tier·E2E manifest | 통과, E2E 161파일 / 161행 |
| 문서 링크·metadata | 링크 통과, base/head 기존 16오류 동일·신규 0 |

전체 Rust 명령은 `CARGO_BUILD_JOBS=8 cargo nextest run --locked --cargo-profile release-test
--target-dir /home/tsjang/rhwp/target/pr-review --tests --test-threads 12 --no-fail-fast`다.
공용 target을 순차 재사용했다. 제품 보정 source `cfd06cd2b5cc47a268134d0c09f657a7a9fd24d0`
이후 제품 입력 바이트가 동일한 것을 지문으로 확인했으며, 후속 변경은 E2E·증적뿐이다.
문서 trailing 때문에 같은 제품의 전체 회귀·WASM 빌드를 반복하지 않는다.

WASM은 Docker 없는 승인된 호스트 진단 경로에서 wrapper `--target web --out-dir pkg --no-opt`,
jobs 8·release codegen-units 16으로 fresh 빌드했다. 최적화 배포 검증으로 확대하지 않는다.
실제 Studio E2E는 Chrome 155.0.8059.79, 표준 Sweep의 지정 브라우저는 Chrome 154.0.8037.57이다.
Windows 모듈 URL은 기존 정규화 패턴에 맞췄으나 실제 Windows 실행으로 보고하지 않는다.

원격 후속 조건은 이 문서 trailing head의 실제 required check·fast-pass·mergeability 확인과 정상 merge다.
아직 실행 전인 merge·이슈 종료·댓글·정리를 완료로 쓰지 않는다. 사용자는 전체 후속 처리를 승인했다.


## 증적과 한계

공통 인쇄 변경의 원본·PDF·전체 TSV·직접 판독·기존 차이는 [PR #7715 검토](pr_7715_review.md)와
[증적](../assets/issue_7710/README.md)에 기록했다. 이 PR의 API 계약 검증을 물리 배치 검증으로 대체하지 않는다.

실행 로그·입력/producer 지문·CI API 영수증은 검토 중 ignored output에 보관하고,
후속 정리 시 `/home/tsjang/rhwp-pr-evidence/pr-8003/`에 해시 확인 후 보존한다.
공용 target·기존 입력/PDF·contributor fork는 보존한다.

## Merge 후 contributor PR comment 계획

정상 merge·devel 포함·duration 결과·관련 이슈 상태를 확인한 뒤 원 PR에 한국어 존댓말로
원 기여 감사, author/SHA 보존, 메인터너 보정 이유와 수정 전 FAIL/후 PASS, 실제 CI 링크,
개별 검토·범위와 남은 차이를 게시한다. 통합 완료에 따른 원 PR close임을 명시하고
직접 merge된 것으로 표시하지 않는다. contributor fork는 수정·삭제하지 않는다.
실제 줄바꿈의 UTF-8 body-file로 게시하고 API로 내용·한글·선두 BOM을 재확인한다.
같은 merge·증거의 댓글은 중복 게시하지 않는다. 사용자 승인 범위에서 이슈와 원 PR을 처리하고
자체 임시 branch·root output 전체를 정리한다.
