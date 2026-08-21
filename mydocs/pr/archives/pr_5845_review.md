# PR #5845 검토 기록

## 판정

**수용 예정**. 원 PR head `530fd2790476a7233f860d37ac68e4b5e411743b`를 최신
`upstream/devel` (`1b8c39b6c32baf00574564016bd9d9f7d1d88198`) 위 통합 후보에 반영했다.
통합 후보의 GitHub CI가 녹색이 되는 것을 최종 merge 조건으로 둔다.

## 변경과 검토

- 탭 점선 리더의 pitch와 dot diameter를 글꼴 크기 기반 상수로 통일했다.
- SVG, web Canvas, native Skia가 같은 `tab_dot_leader_stroke` 계약을 사용하므로 출력 경로별
  점 모양 불일치가 생기지 않는다.
- 최신 원 PR CI는 Build & Test, CodeQL, Render Diff, Native Skia를 포함해 성공했다.
- 별도 메인터너 보정은 필요하지 않았다. 원 PR의 최신 head와 통합 반영 SHA를 재확인했다.

## 로컬 검증

최신 통합 후보 `05e3789ebb222ddb7b4653462ff85d6678de6006`에서 실행했다.

- `cargo test --locked --profile release-test --target-dir target/pr-review --test issue_5843_tab_dot_leader_pitch`
  - 4 passed, 128 skipped
- `cargo fmt --all -- --check`
- `cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings`
- `scripts/wasm-pack-locked.sh --target web --out-dir pkg`
- `cargo test --locked --profile release-test --target-dir target/pr-review --features native-skia --lib`
- `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`
  - 8121 passed, 3 slow, 39 skipped (217.679s)

## 시각 증적

- 입력: `samples/KTX.hwp`, 기준: `pdf/KTX-2022.pdf` (Hancom Office 2022 PDF), p2.
- 후보 sweep: `output/fidelity-pr5845-ktx-p002-20260822`, 1/1 page complete,
  pixel diff 8.81%.
- 기준선 sweep: `output/fidelity-base-pr5845-ktx-p002-20260822`, 1/1 page complete,
  pixel diff 8.14%.
- 전체 pixel 값은 의도된 dot pitch/diameter 변경 때문에 단독 합격 기준으로 쓰지 않았다. 사람이
  비교 PNG에서 리더가 끊기지 않고 페이지 번호의 우측 정렬을 유지하며, 기준 PDF와 같은 점선 역할을
  수행하는 것을 확인했다.
- 대표 PNG: `mydocs/pr/assets/pr_5845_ktx_p002_visual_review.png`
  (`sha256: 408f641e3db595455a62dde49ca8e0c3095aa3b574c662caf77ec04db7e053cc`).

관련 이슈: #5843. 통합 PR CI 성공과 실제 merge SHA 확인 뒤 issue/원 PR comment 및 close를 처리한다.
