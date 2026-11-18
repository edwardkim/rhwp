---
kind: investigation
status: active
canonical: mydocs/manual/publish_guide.md
last_verified: 2026-11-18
---

# v0.9.1 릴리즈 준비 증거

[준비 보고서](../../../report/release_0_9_1_preparation_report.md)와 [수행 계획](../../../plans/release_0_9_1_20261118.md)을 따른다. 기능 마감은 사용자 승인 devel `197ce19e99`이며 이후 새 기능은 추가하지 않는다.

- `contributor-ledger.json`: 1,238개 commit, 사람 19명·Dependabot, 미분류 0건. 원시 이메일은 없다.
- `completed-rust-gates.json`·`rust-validation-source-cohort.json`: Rust C0와 프런트엔드 5167의 실제 실행 SHA·동일 Rust 입력 증거.
- `local-gate-results.json`·`fresh-wasm-build.json`: 실제 명령·결과·시간과 최적화 JS/WASM 해시.
- `local-crate-archives.json`·`package-registry-isolation.json`·`publish-preflight.json`: 새 8개·기존 6개, 별도 소비와 공개 소유자·checksum 확인. 업로드는 없다.
- `package-artifacts.json`·`npm-package-consumer-result.json`: 로컬 preview 패키지와 실제 별도 소비.
- `amo-source-result.json`·`amo-source-file-audit.json`·`amo-rebuild-comparison.json`·`amo-rebuilt-firefox-e2e-result.json`: 추출 소스의 실제 냉 재컴파일·재포장·동작 확인. 이번 JS/WASM·확장 파일의 해시가 같음을 확인했으며 다른 호스트까지의 일반적 비트 재현을 주장하지 않는다.
- `chrome-settings-correction.json`·`embedded-open-correction.json`·`embedded-open-controls.json`: 실제 실패에서 원인 보정과 정상 대조군 재검증까지의 증거.
- `protected-validation.json`·`protected/{native,wasm}/`: 18문서·3,542쪽의 전쪽 TSV·정밀 metrics·입력/출력 해시. 공식 보호값 하락·누락은 0행이다.
- `protected-font-*.json`·`protected-nested-environment.json`: 과거 공급 조건의 해시 재구성. 비공개 글꼴 프로그램은 포함하지 않는다.
- `protected/direct-review/`: 중첩 표·기호·앞뒤 내용의 독립 PDF/Native/WASM 판독 자료.
- `handbook-gate.json`: 정식 편람 Native 383쪽 gate의 실제 결과.
- `pdf-warning-baseline-comparison.json`: 기존 CI와 보고용 경고의 대조이며 독립 Print의 전쪽 증거를 대신하지 않는다.
- `extension-manifest-comparison.json`·`web-network-source-review.json`: 권한·CSP와 실제 production 경로의 정적 네트워크 대조. 실행 범위를 구분한다.

로컬 산출물은 실제 게시물이 아니다. 각 영수증의 실제 실행 소스를 보존하며, 최종 승인된 main/tag에서 다시 포장·게시한 뒤 공개 소비를 별도로 검증한다. 원격 CI·병합·실제 배포 완료를 미리 선언하지 않는다.
