# PR #4932 검토 - HWPX OLE shape component 원문 보존

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4932](https://github.com/edwardkim/rhwp/pull/4932) |
| 작성자 | `planet6897` |
| 검토자 | `jangster77` |
| 검토 방식 | 외부 기여 PR 누적 체리픽 검토 및 메인터너 보정 |
| base / 원본 head | `devel` / `bdeae910f490454b59ae4e390be15bfcb14c6075` |
| 검토 브랜치 반영 | `709ceec7c`, `c2762524d` |
| 메인터너 보정 | `1afe41afb` |
| 검토일 | 2026-08-16 |
| 작성 시점 병합 상태 | `CLEAN` |

## 검토 범위와 판단

- OLE의 `id`와 `instid`, offset·원본/현재 크기·flip·renderingInfo·lineShape를 분리 보존하는
  방향은 적절하다.
- OWPML 스키마상 `hp:ole`가 상속하는 공통 도형의 `id`는 `xs:nonNegativeInteger`이므로
  명시적 `id="0"`도 유효하다. 원본 구현은 이를 속성 부재와 합쳐 직렬화 시 `instid`로 바꿀 수 있었다.
- 메인터너 보정 `1afe41afb`은 `Option<u32>`으로 속성 부재와 `0`을 구분하고, 별도 회귀 테스트를
  추가했다.

## 검증

- `cargo fmt --all -- --check` 통과
- `CARGO_TARGET_DIR=target/pr-review-planet6897 cargo test --lib issue4669` 통과, 5건
- `CARGO_TARGET_DIR=target/pr-review-planet6897 cargo test --test issue_4668_pic_offset_preserved` 통과
- 누적 검토 브랜치에서 `git diff --check` 통과

## 최종 권고

원본 PR head에는 `id="0"` 보정이 아직 없으므로, 메인터너 보정을 해당 PR 또는 통합 PR에 반영한 뒤
병합을 권고한다.


## 통합 병합 판단 및 local validation

- 통합 PR: [#4941](https://github.com/edwardkim/rhwp/pull/4941)
- 통합 코드 후보: `1afe41afb`를 포함한 `integrate/planet6897-open-prs-20260816`
- `cargo fmt --all -- --check`: 성공
- `cargo test --profile release-test --tests`: 성공 (exit 0)
- `cargo clippy --all-targets --all-features -- -D warnings`: 성공 (exit 0)
- `cargo build --release`: 성공 (exit 0)

명시적 `hp:ole id="0"`을 보존하는 메인터너 보정은 `1afe41afb`에 포함되어 #4941로
반영됐다. 원본 #4932 head에는 이 보정이 없으므로 개별 병합하지 않고, 전체 local validation을
통과한 #4941의 최신 head CI가 통과하면 #4941만 병합한다.
