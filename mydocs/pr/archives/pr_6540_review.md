# PR #6540 검토 기록 - CLI 도움말 인덱스와 명령별 안내

- PR: [#6540](https://github.com/edwardkim/rhwp/pull/6540)
- 관련 이슈: [#6539](https://github.com/edwardkim/rhwp/issues/6539)
- base: `devel`
- head: `fix/6539-cli-help-index-20260831`
- 검토일: 2026-09-01

## 판정: 수용 가능

## 범위

- `rhwp --help`는 공개 최상위 명령만 이름순으로 표시하는 짧은 index로 정리했다.
- `rhwp <명령> --help`와 `rhwp edit|inspect --help`는 해당 범위의 상세 안내와 이름순 하위 index를 제공한다.
- 내부 진단 명령 `core-pages`, `dump-extents`, `measure-width`는 capability 자기서술과 직접 상세 help를 유지하되 root index에는 노출하지 않는다.
- 기존 dispatcher와 did-you-mean이 의존하는 capability 등록 순서는 변경하지 않았다.

## 회귀 계약

- root help는 200줄 미만의 공개 명령 index여야 하며 이름순이어야 한다.
- 각 공개 최상위 명령, `edit`, `inspect`의 모든 선언 하위 명령은 해당 scoped help에서 이름순으로 발견돼야 한다.
- 기존 root help 의존 계약은 대상 명령의 scoped help를 검사하도록 옮겼다.
- 세 진단 명령은 직접 호출한 `--help`에서 입력 형식과 주요 옵션을 안내해야 한다.

## 검증

```text
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --target-dir target/pr-review -- -D warnings
node scripts/rust-test-suite-manifest.mjs --check
cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 --no-fail-fast
git diff --check
```

결과: 성공. nextest는 8,906 passed, 46 skipped였다. 1개 장시간 테스트는 성공으로 완료됐다.

## 시각 증적

해당 없음. CLI help, 회귀 계약, 매뉴얼만 변경하며 renderer, fixture, golden, sample 또는 PDF를 변경하지 않는다.

## 병합 전 확인

- PR 최신 head의 required CI가 성공 또는 정책상 expected skip인지 확인한다.
- `mergeable=MERGEABLE` 및 `mergeStateStatus=CLEAN`을 확인한다.
- 병합 뒤 실제 `devel` CI 결과를 기록하고 Issue #6539 자동 종료 여부를 확인한다.
