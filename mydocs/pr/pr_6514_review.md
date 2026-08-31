# PR #6514 검토 - fit-test letter spacing trim 계약

- 검토일: 2026-09-01
- 작성자: `planet6897`
- base: `devel` (`upstream/devel@891e395bb`)
- 원 PR head: `b643b3822edccaa234133fc4cf2701910b090b8f`
- 통합 commit: `c8708e2d8`
- 상태: 승인 (개별 범위 기준)

## 범위

- line-breaking fit-test의 trailing letter-spacing trim을 typed `FitWidthHwp`로 구속한다.
- source unit tier를 우회하지 않고 integration case에서 trim 부호, 경계, pair adjustment 계약을 고정한다.

## 검토 결과

- 실제 fill path는 candidate 마지막 글자의 trailing spacing만 제외한 `FitWidthHwp::trimmed`를 만들고, kerning pair adjustment도 동일 fit-width 축에만 더한다.
- tuple field는 private로 유지돼 외부 test가 raw HWPUNIT를 직접 주입할 수 없고, `#[doc(hidden)]` re-export는 integration test 계약 전용이다.
- `issue_5678_fit_test_letter_spacing_trim`은 `release-test` 종료 코드 `0`으로 통과했다.

## 공통 검증

- `rust-test-suite-manifest` prepare/check 통과: 48/48 targets, 최소 6559 cases
- Rust format, native/WASM/workspace/all-target Clippy, workspace build 통과
- full nextest 종료 코드 `0`

## 병합 조건

- #6536와 묶은 통합 검토에서는 #6536의 시각 차단 결함 때문에 PR을 만들지 않는다.
- #6514를 분리 병합하려면 원 PR head와 CI green 상태를 다시 확인한다.

