# PR #5351 검토 - feat(agent): 전략가(근거 대장) 스킬 고도화

- PR: https://github.com/edwardkim/rhwp/pull/5351
- 작성자: `kevin9327`
- base: `devel`
- 원 head: `c41cfb736361cf0553fdda42db3262ebe9f13e0c`
- 원본 적용 SHA: `c41cfb7`
- 누적 검토 branch: `review/kevin9327-macos-20260818`
- 검토 후보 head: `cd9f557ac`

## 결론

누적 통합 PR에 **조건부 수용**한다. 원 PR의 기능 범위는 보존했고, 체리픽 충돌과 현재 정책 불일치는 메인터너 보정으로 분리했다. Docker WASM 재검증과 원격 CI 통과 전에는 병합하지 않는다.

## 검토 범위

- Strategist의 근거 대장·정지 규칙·좌표 계약을 통합했다.
- 체리픽은 최신 `upstream/devel@0bc05ef81` 위에서 원 작성자 커밋 계보를 보존하는 `-x` 방식으로 누적했다.

## 메인터너 보정

- 수동 `[[test]]` 등록은 제거하고 파생 suite 정책으로 이관했다. 정지 코드와 기존 `capabilities --search` 관측 진입점도 문서화했다 (`f5b1b5771`, `cd9f557ac`).

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
