# PR #5849 검토 기록

## 판정

**수용**. 원 PR head `1942ad24267b883b11b5c5d75387102f8ecb4e6c`를 최신
`upstream/devel` (`1b8c39b6c32baf00574564016bd9d9f7d1d88198`) 위 통합 후보에 반영했다.
통합 후보 #5853의 GitHub CI와 CodeQL이 모두 성공했다. trailing docs-only commit의 fast-pass
확인 뒤 admin merge를 실행한다.

## 변경과 검토

- #5715 유령 사다리 gap, #5798 단 밖 T&B 결재 표의 flow band, #5821 압축 장평,
  #5822 저장 frame drift, #5828 landscape rowbreak bleed를 하나의 renderer 보정으로 처리한다.
- #5828은 기존의 bleed 허용을 삭제하지 않고 같은 높이 행에만 반복 흡수를 막는다. 이로써
  이질 행 높이의 기존 허용 범위를 보존한다.
- 최신 원 PR CI는 Build & Test, CodeQL, Render Diff, Native Skia를 포함해 성공했다.
- 별도 메인터너 보정은 필요하지 않았다. 원 PR의 최신 head와 통합 반영 SHA를 재확인했다.

## 로컬 검증

최신 통합 후보 `05e3789ebb222ddb7b4653462ff85d6678de6006`에서 실행했다.

- focused regression: #5715, #5798, #5821, #5822, #5828 각각 1 passed
- `cargo fmt --all -- --check`
- `cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings`
- `scripts/wasm-pack-locked.sh --target web --out-dir pkg`
- `cargo test --locked --profile release-test --target-dir target/pr-review --features native-skia --lib`
- `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`
  - 8121 passed, 3 slow, 39 skipped (217.679s)

## 시각 증적

[PDF/SVG visual sweep 가이드](../../manual/verification/visual_sweep_guide.md)를 적용했다.

- #5798 입력 `samples/issue5798/offcolumn_float_table_no_band.hwp`, Hancom 2020 기준 PDF
  `pdf/pr_5849/hancom2020/offcolumn_float_table_no_band-2020.pdf`
  (`sha256: 33ceccc8022012cb64fc9165d16956af9a7a202fd42c623734c183b7fed492a6`).
  후보는 1/1 page complete, 6.42%; 실제 `upstream/devel` 기준선은 1/1 page complete, 6.77%였다.
  기준선의 상단 결재란 flow reservation으로 인한 본문 하강은 후보에서 사라졌다. 하단 결재란의
  위치 차이는 후보 전후 공통인 기존 fidelity gap으로 남는다.
- #5821 입력 `samples/issue5821/condensed_ratio_title_box.hwpx`, Hancom 2020 기준 PDF
  `pdf/pr_5849/hancom2020/condensed_ratio_title_box-2020.pdf`
  (`sha256: 5d9bc61611a6d01d1c9ea552a78fb472fb9c9afee1e2a2f23193bc3110d17b9c`).
  1/1 page complete, 4.97%, PDF/SVG text difference 0, glyph-risk 0을 확인했다.
- #5822의 private full fixture HWP 2020 MCP 변환은 server `run_status=137`으로 PDF가 생성되지 않아
  독립 시각 대조를 완료하지 못했다. 이는 후보 renderer 실패가 아니라 외부 변환 증적의 제한이며,
  저장 frame focused regression은 통과했다.
- 대표 PNG:
  `mydocs/pr/assets/pr_5849_issue5798_p001_visual_review.png`
  (`sha256: 9b44b20cf46553c74e9ec12669c87ca13112fa8d6335b83156c65d05fc2fd67d`),
  `mydocs/pr/assets/pr_5849_issue5821_p001_visual_review.png`
  (`sha256: 5f7e63d17d7e975db9ae9b2370705bb383d08541ffce721c593625d77e1e5128`).

관련 이슈: #5715, #5798, #5821, #5822, #5828. 통합 PR CI 성공과 실제 merge SHA 확인 뒤
각 issue/원 PR comment 및 close를 처리한다.

## 통합 CI 결과

통합 PR #5853 code head `92fc6e87f1fe146004b971d7b5bef16a3eb2f7a8`에서 Build & Test,
Lint, Native Skia, Canvas visual diff, 모든 test archive shard, CodeQL(Rust 14m09s)을 포함한
필수 검사가 성공했다. WASM Build와 Frontend unit gates는 이 변경 범위에서 skip으로 판정됐다.
