# clippy 와 관련 cargo test

fmt 게이트를 닫은 뒤에 린트와 시험을 닫는다. 순서는 같은 checkout 에서
동시에 돌리지 않는다 (`local_validation.md`).

## clippy

```bash
cargo clippy -- -D warnings
```

경고 한 건도 실패다. `-A` 로 숨기거나 이 스킬 범위 밖 파일을 대량
재포맷하지 않는다.

워크스페이스가 크면 변경 crate 에 한정할 수 있다. 한정했다면 PR 본문에
명령과 사유를 적는다. 기본은 위의 한 줄이다.

## 관련 cargo test

```bash
# 이 스킬 계약
cargo test --test agent_contributor_skill_contract -- --nocapture
python -m unittest scripts.tests.test_agent_contributor
```

`tests/cases/` 에 새 원본을 넣었으면

```bash
node scripts/rust-test-suite-manifest.mjs --generate
node scripts/rust-test-suite-manifest.mjs --check
```

`tests/generated/` · `tests/suites/manifest.json` · `Cargo.toml` generated
블록은 수기 수정하지 않는다.

`src/` 의 `#[cfg(test)]` 줄 번호가 바뀌면
`node scripts/rust-unit-test-tiers.mjs --generate` 도 실행한다.
이 스킬 파동은 `src/` 를 건드리지 않으므로 보통 불필요하다.

## 범위별 기본 (`local_validation.md` §4.3)

| 변경 | 최소 |
|------|------|
| mydocs 만 | `git diff --check`, 링크. Cargo 생략 가능 (사유 기록) |
| Rust | focused test + fmt + clippy. 가능하면 release-test 전체 |
| renderer/layout | 위 + 시각 근거 |
| 스킬·계약만 | 이 절의 두 시험 + fmt + clippy |

전체 `cargo nextest run --tests` 는 CONTRIBUTING 권장이다. 시간이 없어
관련 시험만 돌렸으면 PR 본문에 **실행한 명령**을 적는다.

예제: [11_clippy_deny_warnings.md](../examples/11_clippy_deny_warnings.md),
[12_related_cargo_test.md](../examples/12_related_cargo_test.md).
