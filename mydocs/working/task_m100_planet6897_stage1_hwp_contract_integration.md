# planet6897 HWP3/HWP5 계약 수정 누적 검토

- 기준: `upstream/devel` (`e0851908bbe568e850c4610986247494203b75d5`)
- 검토 브랜치: `review/planet6897-hwp-contracts-20260818`
- 원본 PR: #5176, #5183, #5189, #5191
- 상태: 로컬 누적 체리픽 및 전체 Rust 회귀 완료, draft 통합 PR #5197 생성

## 적용 순서

1. #5176의 HWP5 캡션 공통속성 `attr bit29` 보정 3개 기능 커밋을 적용했다.
   overflow-cell baseline 커밋 `eb17b9c1e329`은 기준 브랜치에 동일 항목이 이미 있어
   빈 체리픽으로 제외했다.
2. #5183의 HWP3 표 미커버 격자 채움 보정을 적용했다.
3. #5189는 #5183의 동일 격자 보정 선행 커밋을 중복 적용하지 않고, 표 셀 겹침 해소
   커밋만 적용했다.
4. #5191의 HWP3 곡선 점 IR 보존 보정을 적용했다.

## 검증

- `node scripts/rust-test-suite-manifest.mjs --prepare`
  - 로컬 파생 suite를 생성했으며, generated 산출물은 Git에 추가하지 않았다.
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=target/pr-review-planet6897-20260818 node scripts/run-rust-test.mjs issue_5136_caption_attr_bit29 -- --cargo-profile release-test --test-threads 8 --no-fail-fast`
  - 4 passed
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=target/pr-review-planet6897-20260818 node scripts/run-rust-test.mjs issue_4367_hwp3_convert_fourth_contract -- --cargo-profile release-test --test-threads 8 --no-fail-fast`
  - 10 passed
- `CARGO_INCREMENTAL=0 cargo nextest run --cargo-profile release-test --target-dir target/pr-review-planet6897-20260818 --tests --test-threads 8 --no-fail-fast`
  - 6,735 passed, 38 skipped, 3 slow, 4분 50초

## 시각 증적 판단

변경은 렌더러의 외관 규칙이 아니라 HWP3 파서와 HWP5 레코드 구조의 개방 안전성
계약이다. `samples/hwp3-table-caption.hwp`, `samples/hwp3-table-grid-gap.hwp`,
`samples/hwp3-table-cell-overlap.hwp`, `samples/hwp3-curve.hwp`를 계약 테스트로
실행했으며, 이번 로컬 검토에서는 별도 PDF를 생성하지 않았다. 한글 2020/2022 실제
개방성은 CI 외 별도 Windows 또는 MCP 증적이 필요할 때 보강한다.

## 통합 PR

- https://github.com/edwardkim/rhwp/pull/5197
- draft 상태에서 CI 결과와 실제 한글 앱 개방성 증적 필요 여부를 추적한다.
