---
kind: pr-review
pr: 6675
source: planet6897
reviewed_at: 2026-09-03
---

# PR #6675 review - declared-row height and padding

## 판정: 승인

`#3386`의 declared row height를 실제 배치 높이에 반영하는 변경을 최신 `upstream/devel` 위 통합 후보에 provenance-preserving cherry-pick으로 적용했다. source head는 `3d6d1c539ced1595080402d101ca55bf75670ab2`이며, 사전 reviewer는 `jangster77`이다.

## 검토 및 검증

- 원 PR의 기능 commit과 rustfmt follow-up을 `-x`로 적용했다.
- 공통 통합 검증: `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --profile release-test --tests`, `wasm-pack build --release --target web`가 모두 성공했다. integration test 결과는 78개 `test result: ok`, 실패 표식 없음이다.
- visual sweep은 2020 Hancom 기준 PDF `pdf/issue1663_coanchored_float_orphan-2020.pdf`와 sample 2/2쪽을 비교했다. 구조 경고 페이지는 없으며, 이는 pixel-perfect 판정이 아니라 행 높이/외곽 overflow 회귀가 없음을 확인하는 증적이다.

## 시각 증적

- [PR #6675 review contact sheet](../assets/pr_6675_6692_planet6897_integration_20260903/stable/pr_6675_review_contact_sheet.png)
- [batch visual sweep record](pr_6675_6692_planet6897_visual_sweep.md)

## 공통 메인터너 보정

통합 후보에서만 `vello 0.10`과 `vello_svg`가 끌어오는 `vello 0.9`의 Scene 타입이 충돌해 all-features 검증이 불가능했다. `Cargo.toml`/`Cargo.lock`을 `vello 0.9`로 정렬한 최소 보정으로 해결했으며, 이 PR의 row-layout 변경과는 독립적이다.

## Merge 후 contributor PR comment 계획

통합 PR merge와 해당 `devel` CI 성공 후에만 원 PR에 한 번 댓글을 남긴다. 댓글에는 통합 merge SHA, 실제 PR/devel CI, 위 stable PNG와 visual record의 링크, 원 PR을 직접 merge하지 않고 통합 PR로 수용했다는 사실을 포함한다.
