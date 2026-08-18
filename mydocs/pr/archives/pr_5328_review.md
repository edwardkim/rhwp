# PR #5328 검토 - feat(agent): 대전(Codex) 항해 스킬 고도화

- PR: https://github.com/edwardkim/rhwp/pull/5328
- 작성자: `kevin9327`
- base: `devel`
- 원 head: `92b7771807b8636002fb912fa10926e79ecccf7a`
- 원본 적용 SHA: `92b7771`
- 누적 검토 branch: `review/kevin9327-macos-20260818`
- 검토 후보 head: `cd9f557ac`

## 결론

누적 통합 PR에 **조건부 수용**한다. 원 PR의 기능 범위는 보존했고, 체리픽 충돌과 현재 정책 불일치는 메인터너 보정으로 분리했다. Docker WASM 재검증과 원격 CI 통과 전에는 병합하지 않는다.

## 검토 범위

- Codex 항해 스킬의 capabilities 검색 fallback·fixture·경계 계약을 통합했다.
- 체리픽은 최신 `upstream/devel@0bc05ef81` 위에서 원 작성자 커밋 계보를 보존하는 `-x` 방식으로 누적했다.

## 메인터너 보정

- 계약 테스트를 `tests/cases/`로 이동했고 한 글자 한국어 검색도 허용하도록 실제 검색 표면에 맞췄다 (`04d1a94b8`, `c9f88fd86`).

## 검증

- suite 정책: `node scripts/rust-test-suite-manifest.mjs --check` 통과 (683 source, 3,112 static test, 32 suite + 9 exception)
- unit tier 정책: `node scripts/rust-unit-test-tiers.mjs --check` 통과 (4,225 test, 298 module)
- formatter: `cargo fmt --all -- --check` 통과
- clippy: `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=target/pr-review-kevin9327-macos-20260818 cargo clippy --all-targets -- -D warnings` 통과
- 전체 회귀: `cargo nextest run --cargo-profile release-test --target-dir target/pr-review-kevin9327-macos-20260818 --tests --test-threads 12 --no-fail-fast` → **7,153/7,153 통과**, 38 skipped
- Native Skia: lib 58/58, `issue_2225_missing_picture_placeholder` 2/2, `render_p37_direct_pdf_export` 4/4 통과
- WASM Docker: `docker compose --env-file .env.docker run --rm wasm`는 로컬 Docker daemon 미기동으로 시작 전 차단됨. 코드 실패가 아니며 daemon 기동 뒤 재실행이 남아 있다.

## 리스크와 후속 조건

- 이 문서는 원 PR 단위의 검토 기록이며, 실제 병합은 누적 통합 PR의 최신 head를 대상으로 한다.
- Docker daemon을 기동한 뒤 WASM 게이트를 재실행하고, 해당 결과와 원격 CI가 통과해야 한다.
- 외부 원 PR은 이 누적 PR 병합 후 체리픽 수용 사실을 코멘트로 남기고 close한다.

## 권고

WASM과 원격 CI가 통과하면 누적 통합 PR에서 수용한다. 개별 원 PR은 직접 병합하지 않는다.

## 2026-08-18 통합 후보 재검증 갱신

- 이 원 PR의 기능·보정은 누적 통합 후보 `0fb81b451`에서 함께 검증했다. 원 PR 자체를 직접 병합하지 않는다.
- `cargo fmt --all -- --check`, `node scripts/rust-test-suite-manifest.mjs --check`, `node scripts/rust-unit-test-tiers.mjs --check`를 통과했다.
- `CARGO_INCREMENTAL=0 cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 10 --no-fail-fast`를 완료해 **7,693 passed, 38 skipped**를 확인했다.
- 기존 Docker WASM 로컬 재실행 제한은 유지한다. 최종 수용은 통합 PR 최신 head의 GitHub Actions(WASM 포함) 통과와 작업지시자 승인 뒤에만 확정한다.

## 최종 권고 갱신

**조건부 수용 유지.** 개별 기능은 누적 통합 PR에서만 수용하며, 원 PR은 통합 PR 병합 뒤 해당 통합 PR을 근거로 close한다.
