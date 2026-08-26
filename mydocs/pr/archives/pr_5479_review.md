# PR #5479 검토 - test: proptest 왕복 변형·예외 고도화 (M04-f)

## 결론

누적 통합 PR #5530에 **조건부 수용**한다. 원 PR의 contributor 변경은 rewrite하지 않았고, 이 통합 PR에서만 병합을 판단한다.

## 메타데이터

- 원 PR: [5479](https://github.com/edwardkim/rhwp/pull/5479)
- 작성자: `kevin9327`
- base: `devel`
- 문서 작성 시점 상태: `OPEN`
- 원 PR head: `8575ac32a95da1cef7be6d5324bbc0e39a4aeff6`

## 체리픽 범위

- 통합 branch 체리픽 commit: `23faf3e0e4fbc4841f815bea104bc22bcdecf559`
- GitHub 원 PR 제목을 범위 정본으로 사용하며, source head의 단일 기능 변경만 이 기록의 검토 대상이다.

## 메인터너 보정

통합 branch의 HWPX 호환 보정은 contributor 원 commit과 분리돼 있다. 이 원 PR의 author·source history는 amend, rebase, force-push하지 않는다.

## 시각 영향

- 원 PR 제목상 독립적인 기준 PDF·픽셀 동등성 주장을 추가하지 않는다. 최종 수용 전 최신 통합 PR CI를 확인한다.

## 검증

- `cargo fmt --all -- --check` 통과
- `node scripts/rust-test-suite-manifest.mjs --check` 통과
- `node scripts/rust-unit-test-tiers.mjs --check` 통과
- `CARGO_INCREMENTAL=0 cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 10 --no-fail-fast` 완료: **7,693 passed, 38 skipped**

## 리스크와 후속 조건

- GitHub Actions, mergeability, required check는 문서 작성 뒤 변할 수 있으므로 병합 직전에 #5530 최신 head 기준으로 다시 확인한다.
- 원 PR은 문서 작성 시점에 열려 있다. #5530 병합 뒤 이 통합 PR을 근거로 원 PR을 comment 후 close한다.

## 권고

최신 #5530 head의 GitHub Actions가 통과하고 작업지시자가 승인하면 이 원 PR 변경을 통합 PR에서 수용한다. 원 PR은 직접 병합하지 않는다.
