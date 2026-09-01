# PR #6540 검토 기록 - CLI 도움말 인덱스와 명령별 안내

- PR: [#6540](https://github.com/edwardkim/rhwp/pull/6540)
- 관련 이슈: [#6539](https://github.com/edwardkim/rhwp/issues/6539)
- base: `devel`
- head: `fix/6539-cli-help-index-20260831`
- 검토일: 2026-09-01

## 판정: 메인터너 보정 후 수용 가능

## 범위

- `rhwp --help`는 공개 최상위 명령과 내부 개발·회귀 명령을 각각 이름순으로 표시하는 짧은 index로 정리한다.
- `rhwp <명령> --help`와 `rhwp edit|inspect --help`는 구조화된 `명령·그룹·사용법·옵션·예시`를 정본으로 출력하고, 해당 범위의 상세 안내와 이름순 하위 index를 제공한다.
- 내부 진단 명령 `core-pages`, `dump-extents`, `measure-width`는 root index의 내부 개발·회귀 섹션과 직접 상세 help에 모두 표시한다.
- 기존 dispatcher와 did-you-mean이 의존하는 capability 등록 순서는 변경하지 않았다.

## 회귀 계약

- root help는 200줄 미만이며 공개·내부 개발·회귀 명령 section이 각각 이름순이어야 한다.
- 모든 capabilities 최상위 명령과 선언 하위 명령은 해당 scoped help에서 `사용법·옵션·예시`와 이름순 index를 제공해야 한다.
- 기존 root help 의존 계약은 대상 명령의 scoped help를 검사하도록 옮겼다.
- 세 내부 진단 명령은 직접 호출한 `--help`에서 구조화된 계약, 입력 형식과 주요 옵션을 안내해야 한다.

## 검증

```text
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --target-dir target/pr-review -- -D warnings
node scripts/rust-test-suite-manifest.mjs --check
cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 --no-fail-fast
git diff --check
```

결과: 성공. nextest는 8,915 passed, 46 skipped였고, 2개 장시간 테스트도 성공으로 완료됐다.

## 시각 증적

해당 없음. CLI help, 회귀 계약, 매뉴얼만 변경하며 renderer, fixture, golden, sample 또는 PDF를 변경하지 않는다.

## 병합 전 확인

- PR 최신 head의 required CI가 성공 또는 정책상 expected skip인지 확인한다.
- `mergeable=MERGEABLE` 및 `mergeStateStatus=CLEAN`을 확인한다.
- 병합 뒤 실제 `devel` CI 결과를 기록하고 Issue #6539 자동 종료 여부를 확인한다.
