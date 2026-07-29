# PR #2508 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2508](https://github.com/edwardkim/rhwp/pull/2508) |
| 작성자 / base | kevin9327 / `devel` |
| 검토자 | @jangster77 (검토 전 지정) |
| 규모 / 검토 스냅샷 | 2026-07-20 GitHub 조회: +8/-0, 1 file, `maintainerCanModify=true`, `mergeStateStatus=BEHIND` (동적 참고값) |
| 범위 | cskwork/lpaiu-cs 기여자의 canonical identity 매핑 |
| 판단 | 누적 통합 PR에 수용 |

## 변경 범위와 통합
- PR 본문은 릴리스 기여 기록이 중복되지 않도록 두 기여자의 canonical email을 `.mailmap`에 등록한다.
- PR 코멘트는 검토 시점에 없었다.
- 기여자 원 변경 `2d533f5eb`을 적용했다.

## 렌더 영향 판정
- Git identity metadata만 변경하므로 visual sweep 대상이 아니다.

## 검증
- 누적 통합 브랜치에서 `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`, `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `wasm-pack build --target web --out-dir pkg`를 통과했다.
- `git check-mailmap`으로 cskwork와 lpaiu-cs의 canonical identity가 기대값으로 해석되는지 확인했다.

## 리스크와 권고
- 기존 author history를 rewrite하지 않고 표시용 alias만 정규화한다.
- **권고**: 누적 통합 PR에 수용. 최신 통합 PR head의 CI가 성공한 뒤에만 merge한다.

## 후속 정정 (2026-07-20)
- merge 뒤 [PR #2508의 인라인 검토](https://github.com/edwardkim/rhwp/pull/2508#discussion_r3611951113)에서
  `99919585`가 `lpaiu-cs` 계정 ID가 아니라는 지적이 제기됐다.
- 실제 author 이력은 `lpaiu-cs <lpaiu.cs@gmail.com>` 40건과 같은 이메일의
  `김준영 (Juneyoung, Kim)` 2건이다. 따라서 잘못된 noreply canonical 두 줄을 제거하고,
  이름 표기만 실이메일 canonical로 통합한다.
- `git check-mailmap`과 전체 author 이력 집계로 정정 결과를 확인한다.
