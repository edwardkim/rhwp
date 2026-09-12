---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-12
---

# PR #7040 — 누적 체리픽 검토

## 최종 판정

**머지 보류**. 아래 구현·검증 blocker를 해소하기 전 이 변경을 수용하지 않는다.

| 항목 | 확인값 |
| --- | --- |
| 원 PR / 이슈 | [#7040](https://github.com/edwardkim/rhwp/pull/7040) / [#7008](https://github.com/edwardkim/rhwp/issues/7008) |
| 작성자 / reviewer | lpaiu-cs / jangster77; 원 PR reviewer 선행 지정 완료 |
| base / draft | devel / false |
| 원 source head | `6b675ac94c8bff70912dae3a3e61ffc62c49b0c6` |
| 규모 | 3 files, +245 / -32 |
| mergeability | 조사 시 MERGEABLE / CLEAN; volatile 참고값 |
| 검토 경로 | collaborator_external_pr 체리픽 통합 + intake_and_review + local_validation + multi_pr_update_branch + visual_fixture_evidence |
| 구현 계획 | [원 PR별 적용·후속 계획](pr_7040_review_impl.md) |

실제 diff로 범위를 판단했다. `src/renderer/float_placement.rs`, `src/renderer/height_measurer.rs`, `tests/cases/issue_7008_cell_tac_nested_side_by_side.rs`.
문서 전용 PR이 아니며, renderer/진단 또는 관련 기준값에 영향이 있어 공통 조판 검토를 적용했다.
원 PR CI는 모두 성공했으며 아래에서 재사용 근거까지 구분한다.
원격 head는 종료 전 재확인했으며 갱신됐다면 기존 판정을 최신 head에 이월하지 않는다.

## 원 PR 최신 head CI

GitHub Actions를 2026-09-12에 다시 조회했다. 원 PR 4건의 최신 CI는 모두 **SUCCESS**다.

| 원 PR | 최신 head CI | 실행 또는 재사용 근거 |
| --- | --- | --- |
| #7040 | [34674768254](https://github.com/edwardkim/rhwp/actions/runs/34674768254) | head `6b675ac94`에서 Lint·Native Skia·Build & Test 성공 |
| #7048 | [34672055783](https://github.com/edwardkim/rhwp/actions/runs/34672055783) | `b0d657d75`의 [성공 CI 34665904362](https://github.com/edwardkim/rhwp/actions/runs/34665904362) 재사용 |
| #7050 | [34672053443](https://github.com/edwardkim/rhwp/actions/runs/34672053443) | `e28ba0dc7`의 [성공 CI 34659681269](https://github.com/edwardkim/rhwp/actions/runs/34659681269) 재사용 |
| #7053 | [34672052090](https://github.com/edwardkim/rhwp/actions/runs/34672052090) | `5e83a52d5`의 [성공 CI 34670322954](https://github.com/edwardkim/rhwp/actions/runs/34670322954) 재사용 |

세 재사용 경로는 preflight의 `direct-source-build-and-test-green:success`와
`current-base-merge-tree-match`를 확인했다. 재사용 원본 run의 Lint·Native Skia·Archive A/B/C/D·
Build & Test도 모두 성공했다. 최신 head의 worker skip은 이 검증된 재사용 경로이며 누락으로 판정하지 않는다.
이후 아래 로컬 누적 후보의 실패는 원 PR CI와 구분한다. CI 녹색을 취소하거나 단독 source 실패로 바꾸어
기록하지 않는다. 코드 계약 검토 결과와 로컬 누적 환경의 차이는 각각 별도의 검토 근거다.

## 발견 사항과 해제 조건

### P1 — 서로 다른 문자 위치 척도로 저장 줄을 선택한다

`src/renderer/float_placement.rs:555-566`은 `control_text_positions()`의 **텍스트 character 인덱스**를
`LineSeg.text_start`의 **컨트롤 슬롯을 포함한 UTF-16 위치**와 직접 비교한다.
`src/model/paragraph.rs:1689-1703`, `:1826-1867`, `:1901-1943`에 각 좌표 계약과 변환 경로가 있다.
HWPX 첫 문단의 축 보정도 새 함수에서는 사용하지 않는다.

최소 모델에서 공개 `control_text_positions()`와 PR의 줄 선택식을 실행했다.
원시 문단 축은 첫 표 슬롯 `[0,8)`, A 위치 8, 둘째 표 슬롯 `[9,17)`, B 위치 17이다.
`text="AB"`, `char_offsets=[8,17]`, `char_count=19`, 표 컨트롤 2개,
저장 줄 시작 `[0,9]`이면 표 줄 소속은 `[0,1]`이어야 한다.
실제 반환 character 위치는 `[0,1]`이고 새 식의 결과는 **`[0,0]`**이었다.
저장 줄에는 각각 높이 750HU, 기준선 600HU를 주고 둘째 줄 y는 900HU로 두었다.
이는 좌표 계약 재현이며, 변형 문서를 한컴에 저장·출력해 확인한 렌더링 회귀라는 주장은 아니다.

해제 조건: 컨트롤의 원시 시작과 저장 줄 시작을 같은 좌표축으로 정규화하고, 앞선 컨트롤 슬롯·
서로 다른 저장 줄·비 BMP 문자·HWPX 보정 축의 경계를 검증한다. 단순 상수 보정으로 덮지 않는다.

### P1 — 측정과 배치가 같은 줄 구성 결과를 소비하지 않는다

`para_nested_table_line_indices()`는 `height_measurer.rs:1919`에서만 호출된다.
배치의 `table_layout.rs:5664`는 문단 전체 `para_float_group_is_side_by_side()`를 호출한다.
새 이름의 공통 술어를 호출하더라도 전달하는 표 집합의 단위가 다르므로 공통 줄 구성 결과가 아니다.
저장 줄이 없으면 측정은 모든 표에 줄 0을 부여하고 `all(tac)`일 때 최댓값을 취한다.
이는 종전 devel의 TAC 적층 높이 합산과 다르며, PR 설명의 ‘종전 폴백 유지’와도 다르다.
재조판의 개행·사용 가능 너비·여백을 반영한 실제 줄 나눔을 소비하지 않는다.

줄별 표 선언/측정 높이의 최댓값을 더하는 `height_measurer.rs:1939-1964`에도
그 줄의 실제 기준선·점유 구간·줄간격이 없다. 다른 측정 경로가 일부 높이를 보완한다는 것과
이번 공통 규칙이 성립한다는 것은 구분해야 한다.

해제 조건: 이번 중첩 표 범위 안에서 줄 소속과 점유 메트릭을 측정·배치가 같이 사용하고,
저장 사다리와 재조판 경로를 구분해 개행·너비 부족·텍스트/여백 혼재 사례를 검증한다.
전체 엔진 재작성을 요구하지 않는다. 정상 한컴 출력이 없는 경로는 미검증으로 남긴다.

## 직접 확인한 개선과 한계

원본 `21_언어_기출_편집가능본.hwp`의 새 테스트 3개는 통과했다.
통합 render tree의 머리 표는 `(x=118.0, y=131.6, w=886.6, h=183.2)`px이고,
하단 314.8px로 본문 첫 글줄을 덮지 않았다. 두 중첩 표의 y는 모두 254.7px였다.
한컴 2022 PDF 1쪽과 비교 PNG를 직접 열어 괘선이 본문 위에 놓이는 것을 확인했다.
성명·수험번호 상자의 절대 y 차이와 글꼴·본문 배치 차이는 남는다.
같은 줄 표와 실제 여백 혼재 원본 증거를, 개행·폭 부족 정상 문서 증거로 확대하지 않는다.
이번 diff에는 baseline 완화가 없다. 기존 리뷰가 지적했던 완화 삭제는 확인했지만 위 구현 결함을 해소하지는 않는다.

## 공통 조판 원칙 준수

[공통 계약](../../manual/pr_review/intake_and_review.md#27-조판-원칙-준수-검토)을 실제 호출 경로와 대조했다.

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 미충족 | 문자 척도 혼용과 NO_LS의 문단 단위 TAC 가정 |
| 측정·배치 일관성 | 미충족 | 줄 매핑은 측정만 소비; 배치는 문단 전체 집합 |
| 줄 소속과 점유 높이 | 미충족 | 실제 줄의 baseline·점유·줄간격을 공통 산출하지 않음 |
| 사례와 증거의 독립성 | 미검증 | 원본 3개 테스트/1쪽 시각 확인; 개행·폭 부족 정상 문서 없음 |
| 기준값 변경 | 비해당 | 현 head에는 baseline 변경 없음 |
| 주장과 검증 범위 | 미충족 | 종전 폴백 유지/공통 소비 주장과 실제 호출 경로가 다름 |

## 검증 환경과 결과

- macOS arm64, logical CPU 10, RAM 32 GiB, Rust 1.93.1, 기본 nextest 동시성.
- 기준 devel `ea5d1ff70b1d50301d1e6fdd26248e9d9c10c1fa`; 실제 코드 검증 head `522a2e80db04cbd84264406ccb8bdd33a21dcc55`.
- `target/pr-review`의 기존 소유·공유 상태와 실행 중 Cargo/Rust 작업 부재를 확인했다.
  공유 debug/release 및 다른 review target을 삭제하지 않고 고정 review target을 재사용했다.
- 검증에 사용해 보관한 `candidate-rhwp`의 SHA-256은
  `1e618148bf00501c613b3cd19269ed0453dbe90b630f84786ed7cf4b8366808b`다.
  대조 실험 후 복원한 source는 code head와 diff가 없다. 재빌드한 작업용 CLI와 보관한 검증 바이너리는 구분한다.
- `--prepare`, `cargo fmt --all`, fmt check, native Clippy, WASM32 Clippy,
  workspace build, workspace all-target Clippy, manifest check가 순차로 모두 exit 0이었다.
  파생 generated suite는 stage하지 않았다. source-side cfg(test)는 변경하지 않아 unit-tier 추가 gate는 비해당이다.
- 집중 nextest: **14 PASS / 1 FAIL / 9,548 filtered/ignored**. 실행 15건 중 실패는 #7048 새 회귀 1건.
- 전체 nextest: **9,516 PASS / 1 FAIL / 46 skipped**, 실행 342.021초, exit 100.
  실패는 동일 #7048 테스트뿐이다. 전체 성공이라고 기록하지 않는다.
- 새 sample 1개를 `RHWP_SECURITY_SWEEP_SAMPLES_JSON`으로 명시한 security 검사 PASS.
  기존 samples 전수 래칫은 통과했지만 baseline 증가의 독립 타당성은 별도 판정이다.
- Native Skia lib: exit 0, 4 binaries, 4112 PASS / 0 FAIL / 13 ignored.
- native-placeholder: exit 0;      Summary [   1.012s] 2 tests run: 2 passed, 190 skipped
- native-pdf: exit 0;      Summary [   0.766s] 4 tests run: 4 passed, 189 skipped
- WASM 진단 build: exit 0. Docker CLI는 있으나 daemon에 연결하지 못해
  공식 wrapper의 native `--no-opt` 경로를 사용했다. 최적화된 배포 빌드 통과로 주장하지 않는다.
- native↔WASM SVG parity: exit 0; 세부 결과는 아래 WASM 항목.
- OVR5 전수 base/head geometry 비교, 다른 OS, 원 제보 법령 187쪽, 누락한 다중 줄 정상 한컴 출력은 미실행이다.
  구현 blocker와 전체 테스트 실패가 남은 이 통합 branch의 merge gate를 완료한 것으로 처리하지 않는다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --locked --target-dir target/pr-review -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir target/pr-review -- -D warnings
cargo build --locked --workspace --target-dir target/pr-review
cargo clippy --locked --workspace --all-targets --target-dir target/pr-review -- -D warnings
node scripts/rust-test-suite-manifest.mjs --check
cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --no-fail-fast \
  -E 'test(/issue_7008|issue_7023|issue_7049|issue_6986|layout_anomaly_glyph_band/)'
RHWP_SECURITY_SWEEP_SAMPLES_JSON='["samples/issue6986/cell-page-and-total-page-in-one-run.hwpx"]' \
  cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --no-fail-fast
cargo test --locked --profile release-test --target-dir target/pr-review --features native-skia --lib
node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --target-dir target/pr-review --features native-skia
node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --target-dir target/pr-review --features native-skia
CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web \
  --out-dir /tmp/rhwp-nondraft-review-20260912-YtWNPm/wasm-pkg --no-opt
```

원시 로그·JSON·probe source는 `/tmp/rhwp-nondraft-review-20260912-YtWNPm`에 남겼다. 저장소에는 요약과 최종 증적만 포함한다.
최소 계약 probe는 아래 명령으로 native debug 라이브러리에 연결해 실행했다. `review_probes.rs`의
SHA-256은 `e1d63642ce4dff5b617fc84efe6886cda4e2217e1e5a06260acb3e7ba313c423`이다.
이 하네스는 제품 source를 수정하지 않으며, #7040은 공개 모델 위치 API와 새 비교식,
#7048은 실제 공개 진단·SVG 출력 API를 실행한다.

```bash
rustc --edition=2021 /tmp/rhwp-nondraft-review-20260912-YtWNPm/review_probes.rs   --extern rhwp=target/pr-review/debug/librhwp.rlib -L dependency=target/pr-review/debug/deps   -o /tmp/rhwp-nondraft-review-20260912-YtWNPm/review_probes
/tmp/rhwp-nondraft-review-20260912-YtWNPm/review_probes
```


## 시각 증적과 provenance

[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 사용했다. macOS Chrome webfont raster, 96 DPI다. 픽셀/잉크 일치율은 후보 지표이며 호환성 점수가 아니다.

- 원본 `samples/21_언어_기출_편집가능본.hwp`, SHA-256 `905454045ca2e236839a7cab59750678116d08af3db31dbf846819af355b8d15`.
- 기준 `pdf/21_언어_기출_편집가능본-2022.pdf`, 851275 bytes, SHA-256 `f2d858d7974393661d91a658e6b384b951114ef52783379f426a963effd97b72`, SHA-1 `f4a5f1d4f36eae88e0e68dae33160a7c1ab8d599`. Creator:         Hwp 2022 12.0.0.4426; Producer:        Hancom PDF 1.3.0.550; Pages:           15; PDF version:     1.6.

![PR 7040 직접 확인 패널](../assets/pr7040_integrated_21_review_p001.png)

- 원 산출: `/tmp/rhwp-nondraft-review-20260912-YtWNPm/visual/pr7040-21/review/review_001.png`; 최종 SHA-256 `cbf9bc83ab98fdcd7f3e43e1ae04b40adf710231174abbe820ac33892b0dad98`.
- pr7040-21: 1쪽 직접 확인, flagged 0쪽; pixel match 88.134%, ink match 12.610%.

## WASM 확인

```text
6 documents / 28 pages: native and WASM SVG all MATCH; exit 0.
Original fixtures: 4 documents x page 1.
Synthetic PAGE/TOTAL_PAGE order variants: 2 documents x 12 pages.
```

## 원격 후속 처리

현재 판정은 보류다. 이 기록을 GitHub approve/merge/close로 해석하지 않는다.
구체적인 위반 위치·실행 결과·미검증 범위를 보완 요청 근거로 사용하고, 수정 head에서 다시 검토한다.
