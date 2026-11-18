# v0.9.1 릴리즈 준비 증거

[준비 보고서](../../../report/release_0_9_1_preparation_report.md)와
[수행 계획](../../../plans/release_0_9_1_20261118.md)의 검증 범위를 따른다.

- `contributor-ledger.json`·`contributor-overrides.json`: v0.9.0 이후 1,225개 commit,
  사람 19명·Dependabot과 출처 판정. 원시 이메일은 포함하지 않는다.
- `published-crate-source-audit.json`·`candidate-crate-versions.json`: 공개 archive를
  실제 다운로드·checksum 확인한 소스 대조와 새 버전 8개.
- `completed-rust-gates.json`·`rust-validation-source-cohort.json`: 실제 Rust 실행 SHA,
  결과·시간과 중간 commit의 동일 Rust/Cargo 입력 증거.
- `fresh-wasm-build.json`: 표준 release·혼합 crate·실제 wasm-opt·source/도구/JS/WASM hash.
- `local-gate-results.json`: Studio·타입·브라우저·포장·Cargo 소비의 실제 명령과 결과.
- `npm-package-consumer-result.json`: 실제 tgz를 푼 별도 브라우저 소비와 버전·재열기.
- `local-crate-archives.json`·`package-registry-isolation.json`: 로컬 .crate 원장,
  새 버전 8개·공개 archive 재사용 6개 및 정규화 의존성의 격리 확인.
- `publish-preflight.json`: 공개 소유자·exact version·checksum 검증. 새 버전은
  `pending-approval`이며 업로드는 수행하지 않았다.
- `pdf-warning-baseline-comparison.json`: 보고용 PDF 경고의 기존 CI 대조.
  독립 한컴 Print·전쪽 Visual Sweep 증거로 사용하지 않는다.
- `extension-manifest-comparison.json`·`web-network-source-review.json`: 권한·CSP,
  추가 네트워크 코드와 실행 범위를 구분한 확장 확인.

로컬 산출물은 실제 게시물이 아니다. 최종 승인된 main/tag에서 새로 포장·게시하고
그 source SHA와 공개 채널 상태를 별도 검증한다. AMO source 재빌드와 최종 archive
원장은 실행 완료 뒤 이 경로에 추가한다.
