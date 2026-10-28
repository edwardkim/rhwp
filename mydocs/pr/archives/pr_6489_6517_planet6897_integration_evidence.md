# planet6897 CI 통과 PR 통합 검증 및 시각 증적

- 검토 브랜치: review/planet6897-ci-green-20260831
- 기준: upstream/devel e718f40bbd335b2c8db6e89345f91a5e88c70e16
- 통합 head: 76532b4da0e720026fb24211ad0c382884d3b970
- 적용 방식: 최신 원 PR head를 git cherry-pick -x로 누적 적용
- 원 PR: #6489, #6491, #6492, #6496, #6497, #6498, #6512, #6515, #6516, #6517
- 제외: #6514는 원 PR CI 실패로 적용하지 않았다.

## 공통 코드 검증

| 검증 | 결과 |
| --- | --- |
| git diff --check upstream/devel...HEAD | 통과 |
| cargo fmt 및 workspace Clippy (-D warnings) | 통과 |
| WASM lib Clippy (-p rhwp --lib) | 통과 |
| workspace build 및 Rust test-suite manifest | 통과: 48/48 integration target |
| 선정 회귀 9개 | 통과: 22 tests |
| cargo nextest run --tests --no-fail-fast | 통과: 8,885/8,885, 46 skipped |
| Native Skia lib + 선정 회귀 | 통과: 3,952 tests |
| rhwp-wasm-build | 통과: release WASM, wasm-bindgen, wasm-opt, pkg/ 생성 |

일반 workspace cargo build --target wasm32-unknown-unknown는 src/cli/outputs/vector.rs의 기존 WASM 비호환 CLI 메서드 참조로 실패했다. 통합 diff에는 해당 파일이나 src/wasm_api.rs 변경이 없으며, 실제 배포용 rhwp-wasm-build와 rhwp library WASM build는 통과했다.

## 현재 통합 head 시각 sweep: #6492 + #6515

- 입력: samples/3-09월_교육_통합_2022.hwpx
- PDF oracle: pdf/3-09월_교육_통합_2022-hwpx-2024.pdf
- 페이지 수: rhwp 23 / oracle 23
- 자동 후보: 15쪽. frame overflow(9, 10, 13, 19), content-bottom drift(14), marker drift(20), line-band drift, equation/line-order overlap(23) 등이 보고됐다.
- 자동 지표 평균 pixel match: 92.2478%, 평균 ink proxy: 16.19328%.
- 사람이 확인한 review 페이지: 9, 13, 19, 23. 각 이미지는 rhwp, oracle, overlay를 함께 보존한다.
- 이 결과는 자동 후보 선별 자료다. 현재 통합 head와 upstream/devel의 동일 입력 비교를 하지 않았으므로 전역 글꼴·기존 레이아웃 차이를 #6492/#6515가 새로 만들었다는 판정 근거로 쓰지 않는다.

## 보존 자산

- 요약: mydocs/pr/assets/pr_6489_6517_planet6897_integration_20260831/visual-6492-6515/summary.json
- 사람이 확인한 페이지: review/review_009.png, review/review_013.png, review/review_019.png, review/review_023.png

원 PR이 이미 포함한 before/after/oracle PNG는 원 PR head 당시 증적이며, 이 문서의 현재 통합 head 증적과 혼동하지 않는다.
## 2026-08-31 메인터너 보정 증적

이 절의 `현재 후보`는 integration head `76532b4da0e720026fb24211ad0c382884d3b970` 위의 **미커밋 메인터너 보정 작업트리**다. 따라서 새 SHA를 꾸며내지 않고 검증 명령과 산출물 경로로만 식별한다.

- 전체 regression: `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --no-fail-fast` -> `8888 passed, 0 failed, 46 skipped`.
- Native Skia library: `3948 passed, 0 failed, 13 ignored`; focused #6494는 3/3, inline picture suite는 86/86, SVG snapshot은 8/8이다.
- #6492/#6515: base `e718f40b`와 현재 후보를 동일 HWPX/Hancom 2024 PDF로 9, 12, 13, 19, 23쪽 비교했다. 현재 후보는 base의 5개 flag 페이지를 3개로 줄였고 12쪽 tail 및 13쪽 flow collapse를 해소했다. 19·23쪽 잔여 차이는 base에도 있다.
- #6491, #6496, #6516, #6517은 `rhwp info --json`의 실제 저장 메타데이터에 따라 Hancom 2020 profile을 사용했다. #6497의 기준 PDF는 Hwp 2024/Hancom PDF 1.3.0.550 생성본이다.
- #6516 p5 자동 flow-collapse는 그림이 큰 표의 false positive다. focused structural regression과 직접 review image로 두 지도 모두 cell-contained/same-band임을 확인했다.
- #6496 N-up 원본은 logical 9쪽과 physical 5쪽의 1:1 sweep 대상이 아니다. #6496/#6517 Windows font proof는 `HYGothic-Medium`과 `HYSinMyeongJo-Medium`의 실제 TTF family 및 Windows SVG/raster를 사용했다.

안정 이미지: `mydocs/pr/assets/pr_6489_6517_planet6897_integration_20260831/maintainer-20260831/`.
