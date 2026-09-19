---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-19
---

# PR #7269 검토

## 판정

**메인터너 보정 후 수용 가능.** 원 head의 후속 표 간격 회귀는 `0532933b9`에서 보정했다. 검토·통합 대상은 증적 포함 head `14eb5e73ac78ada2ae6ca4a843f99e2810b43891`이며, 원 contributor head를 직접 merge하지 않는다. 원 PR의 green CI는 통합 head의 검증으로 세지 않는다.

## Metadata와 체리픽

- 원 PR: [#7269](https://github.com/edwardkim/rhwp/pull/7269) — fix(layout): 자리차지 표의 윗변 원점을 조판·렌더가 한 결과로 소비한다 (#7203 −13px 무리)
- 작성자 `planet6897`, 코드 검토 reviewer `jangster77` 지정 완료.
- 원 head `1bedbfeeeb7c9cd65114966cc0e73df24c3cd61d`, base `devel`.
- 규모: 5 files, +347 / -142.
- 통합 base `62f1048f469f767a1c9f200209d23cf48b90016a`, branch `codex/planet-review-20260919`.
- #7267 → #7268 → #7269 순서로 충돌 없이 누적 적용. 최초 제품 head `9e58e5170`.
- 검증 전용 worktree `/private/tmp/rhwp-planet-verify-20260919`, target `/Users/tsjang/rhwp/target/planet-review-20260919`.

## 이번 회차 분석

**저장 float 표 원점과 공간 수용 조건 공유**

조판과 렌더가 같은 저장 사다리·여백 원점을 소비하는지 확인한다. hwpctl 7개 신규 anchor·기존 28쪽·TAC·nonzero offset·56345 연속 표를 확인한다. 하단 절반 조건 제거의 과거 반례 1351000을 대조군으로 추가했다. #7203의 나머지 분할·되감기 차이를 종료 대상으로 확대하지 않는다.

분석 → 코드 수정·검증 → 결과보고 → 커밋 순서로 진행한다. 로컬 검증 결과와 남은 범위를 아래 기록했다.
원격 code candidate CI 뒤 같은 통합 PR의 trailing 문서 commit으로 보존한다. Native/fresh WASM Visual Sweep의 compare·standalone
overlay·review를 산출하고 직접 열어 판정했다. 입력과 PDF는 기존 추적 파일을 재사용하며,
없는 기준 PDF와 검증에 꼭 필요한 원문만 추가한다. 불필요한 로그·TSV·JSON은 커밋하지 않는다.

## 메인터너 보완 분석

- 전수 tree 비교에서 16~18쪽의 표 분할 변경을 발견해 추가 sweep으로 확인했다.
  PDF 16쪽은 머리행만 남고 17쪽 pi=309 윗변은 472.76px다. 통합본의 약 472.9px가
  기준 devel 약 448.3px보다 가깝다. 이 긍정 변화와 18쪽 Example의 본문 내 표시를
  독립 PDF 측정값 기반 회귀 검사에 추가했다.
- #5585 기존 반례 테스트는 개인 Windows 경로가 없으면 즉시 return했다.
  Downloads 원문의 해시로 samples/·tests/fixtures 전체를 확인해 기존 추적 중복이 없음을
  확인했다. 해당 원문을 고정 fixture로 연결하고 누락 시 실패하도록 보완했다.
  새 한컴 2020 PDF도 86쪽이며 입력 preprocess는 없다.

## 발견한 회귀와 원인 보정

`56345_regulatory_impact_analysis.hwp` 11쪽에서 후속 pi=187 표가 devel 대비
27.1px 내려갔다. pi=186→187 저장 사다리는 2432HU(32.43px)이지만 통합본은
59.55px를 소비했다. 별도 회귀 검사로 수정 전 FAIL을 확인했다(원 focused 10개는 PASS).

저장 앵커 선택 → 앞 개체의 배제 영역 적용 → 실제 점유 끝 계산까지는 절대 좌표를 쓰는데,
마지막 float lane 소비에서 `global_y_before + reserved_height`를 적용해 같은 간격을
다시 더했다. 첫 번째 보정은 실제 lane 하단을 소비했으나 간격이 28.88px로 과소 소비되고,
정상 hwpctl 25쪽 pi=483이 6.7px 밀려 폐기했다(추가 focused 5개 중 2개 실패).

두 번째 보정은 공유 helper가 구분하던 **전체 흐름 상자 원점** 계약을 최종 소비 지점까지
전달한다. 다음 저장 앵커가 선언 높이+양쪽 바깥 여백만큼 정확히 전진하면 그 높이를
한 번 소비하며, 단순 테두리 원점은 기존 offset 기반 float 흐름 계약을 유지한다.
두 번째 보정 후 focused 5개가 모두 PASS했다. 최초 실패의 59.55px 간격을
저장 사다리의 32.43px로 복원했고, 7개 저장 앵커·TAC·nonzero offset·16~18쪽 분할/본문
대조군을 함께 통과했다. 최종 Native/fresh WASM overlay 및 전체 검증을 완료했다.

## 추가 정상 대조군의 출처

`tests/fixtures/stored_float_anchor_control/1351000_policy_indicators.hwp`는 기존 #5585
테스트가 참조하던 동일 원문이며 SHA-256은
`b28ef2f22b24b962c41559389c3d29fda25acde14c245ee9808c2777aaeec0a9`다.
저장 메타데이터 product는 null, version은 `7.0.1.215`이므로 지침에 따라 engine 2020을 썼다.
생성 환경은 Hancom `11.0.0.9136`, 입력 preprocess 없음, 드라이버 원본 PDF 86쪽이다.
페이지 상자 복구 전 드라이버 원본 SHA-256은
`6c536e36b23bfa6ac396c22062f1cc5f559e04e3d7e35979391dad8b4c128d87`이다.
검증 원문·PDF를 저장소에 함께 보존하며, Downloads나 개인 Windows 경로에 의존하지 않는다.
최초 base/통합의 86쪽 tree 구조·Table/Image/RawSvg 기하가 동일했다. 시각 대조는 22~24쪽이다.

## 최종 소비 계약과 남은 범위

공유 생산 함수 `stored_topbottom_flow_advance_hu` → `stored_topbottom_object_span`의
원점/점유 끝 → 조판의 저장 앵커 선택 → `layout.rs`의 `stored_flow_advance` 전달 →
최종 float lane의 `global_y_before + advance` 소비까지 추적했다. 일반 테두리 원점은
Some advance를 받지 않으므로 기존 offset·caption·physical-extra 소비를 유지한다.
문서 ID나 화면 좌표 상수로 분기하지 않는다.

16쪽 머리행/17쪽 continuation의 컷은 기존 분할기가 결정하고 이 변경은 저장 앵커의
공간 수용 산식을 공유한다. 실제 PDF의 16쪽 header 24.62px / 17쪽 표 270.74px,
뒤 문단 pi=309 y472.76px와 18쪽 Example y990.08px를 별도 focused에서 검사했다.
최종 hwpctl 105쪽과 지표 86쪽 tree는 최초 통합 대비 동일하며, 56345는 11쪽의
후속 표와 하위 내용만 약 −27.1px 복원했다. 실패하던 59.55px 간격은 32.43px가 됐다.

hwpctl 7개 anchor의 PDF 잔차는 1px 이내이며, 16~18쪽의 표 분할/뒤 본문도 개선됐다.
18쪽 별도 pi=322의 y는 132.3→149.6px, PDF 153.27px라 3.67px가 남는다.
56345 11쪽은 이번에 추가된 밀림을 해소했지만 기준 devel부터의 약 28px 차이는 남고,
전체 쪽수도 rhwp 20 / PDF 21이다. 19·20쪽을 추가 판독했으며 이 문서 전체의 일치를
완료로 선언하지 않는다. 지표 문서에는 기존 셀 테두리/내부 텍스트 잔차가 남는다.
#7203의 다른 분할·되감기/후속 내용 범위를 해결한 것으로 닫지 않는다.
닫힌 #6111의 원래 빈 ClickHere 문제와 11쪽 기존 차이를 혼동해 재개하지 않는다.

## 한컴 PDF 페이지 상자 복구

지표 문서는 첫 변환과 재시도 모두 86쪽 A4 세로 MediaBox로 기록됐지만, 내용 스트림의
첫 clip은 **86쪽 모두 `[0,246.331,841.348,841]` PDF 좌표**였다. 변환기
`HwpPrintPaper`가 width/height를 Min/Max로 정렬하며 방향을 보존하지 않는 코드도 확인했다.
원본 PDF의 오른쪽 바깥에 텍스트·벡터가 남아 있었으므로 원문이나 그래픽을 바꾸지 않고
각 page의 MediaBox만 위 한컴 자체 clip에 맞췄다. rhwp의 좌표나 overlay 점수로 맞춘 값이 아니다.
스케일·이동·재래스터화는 하지 않았고 **86개 page 내용 스트림 SHA-256이 복구 전후 전부 동일**하다.

- 드라이버 원본: `pdf/pr7269/1351000_policy_indicators-2020-driver-original.pdf`,
  SHA-256 `6c536e36b23bfa6ac396c22062f1cc5f559e04e3d7e35979391dad8b4c128d87`.
- 복구본: `pdf/pr7269/1351000_policy_indicators-2020.pdf`,
  SHA-256 `6fb5991ba4a99e460a2b964493893e2f6e5434fb15074f64043596026911ad6e`.
- 복구 후 841.348×594.669pt, 86쪽. 23쪽 추출 words는 페이지 밖에 있던 문자가 복구되어
  142→175개다. 이것만으로 조판 일치를 판정하지 않고 22~24쪽을 다시 Visual Sweep했다.

재현은 PyMuPDF에서 원본 각 page의 첫 `q ... W* n` clip 위 값을 확인한 뒤
`page.set_mediabox(pymupdf.Rect(0,246.331,841.348,841))`만 적용해 새 파일로 저장한다.
모든 page의 `sha256(page.read_contents())`가 원본과 같은지 검사한다. 드라이버 원본은
잘못된 용지 상자 재현 자료이며, 비교 기준으로 사용하는 것은 명시적으로 복구한 PDF다.

## 최종 제품과 검증 상태

제품 source는 `0532933b9b543542ff61b12f1031ee3b55636641`이다. 실행 파일은 detached
`9e58e5170` 검증 worktree에 메인터너 보정을 적용한 내용으로 빌드했고, 커밋 직전 source/test
바이트를 주 작업공간과 대조했다. 캡처 manifest의 git_head와 바이너리의 빌드 시점을 혼동하지 않는다.

- Native CLI SHA-256: `2b9f9f6eed0e15063b71a7e2a4ba5fdd6b3f4e246354012c3fe4842d8fe8d8af`.
- fresh WASM SHA-256: `c5d606da57489e04f09a239010788e4e52eff12b3c59fca12378f527bb9299ba`.
- JS glue SHA-256: `a7353a7603b7e07db2d33ff93fff6b213ea79e01da91c190cbb607e752c6b5a7`.
- 최초 통합 focused 10개, 메인터너 보정 후 float focused 5개·필수 fixture 대조군 2개 PASS.
- Native Skia lib 4,112 PASS / 13 ignored, missing-picture 2 PASS, direct-PDF 4 PASS.
- OVR 5문서 PASS: KTX 27쪽/9객체, exam_math 20쪽/9객체, answer 15쪽/3객체,
  aift 74쪽/27객체, biz_plan 6쪽/0객체. 최대 좌표 오차는 answer 0.5px, 나머지 0px다.
  biz_plan의 0객체를 개체 위치 검증으로 확대하지 않는다.
- 최종 Native/fresh WASM 18문서 **1,216쪽** tree 구조·텍스트·bbox 차이 0.
  실제 PDF raster 비교는 각 backend **58쪽**이며, 1,216쪽 전체 PDF 일치라는 뜻이 아니다.
- 전체 nextest **10,085 PASS / 50 skipped** (실행 502.287초), fmt·Native/WASM Clippy PASS.
  workspace build·all-target Clippy·manifest/unit-tier 게이트도 모두 PASS했다.
  manifest는 1,379 source / 48 integration target, unit-tier는 4,205 test를 확인했다.
  원격 CI·최종 head checks·merge는 별도 게이트이며 로컬 통과로 대체하지 않는다.

macOS 26.6.2 / Rust 1.93.1, `DEVELOPER_DIR=/Library/Developer/CommandLineTools`,
`CARGO_TARGET_DIR=/Users/tsjang/rhwp/target/planet-review-20260919`, build jobs 4를 썼다.
10 CPU/32GB에서 다른 시각 검증과의 메모리 경합을 고려해 nextest threads 6을 지정했다.
동일 target의 Cargo 작업은 순차 실행했다. Docker daemon을 사용할 수 없어 WASM은
`scripts/wasm-pack-locked.sh --target web --out-dir <fresh-pkg> --no-opt` host 경로다.
Docker 최적화 빌드 검증이라고 보고하지 않는다.

검증 명령:

```sh
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/run-rust-test.mjs issue_7203_float_table_stored_anchor_top -- --cargo-profile release-test --features native-skia
node scripts/run-rust-test.mjs issue_5585_sibling_table_anchor_offset -- --cargo-profile release-test --features native-skia
cargo test --locked --profile release-test --features native-skia --lib
node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --features native-skia
node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --features native-skia
cargo nextest run --locked --cargo-profile release-test --tests --test-threads 6 --no-fail-fast
cargo fmt --all -- --check
cargo clippy --locked -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown -- -D warnings
cargo build --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
node scripts/rust-test-suite-manifest.mjs --check --base-ref 62f1048f469f767a1c9f200209d23cf48b90016a
node scripts/rust-unit-test-tiers.mjs --check --base-ref 62f1048f469f767a1c9f200209d23cf48b90016a
```

Visual Sweep은 아래 표의 입력·PDF·쪽을 각각 다음 명령에 넣어 재현한다.
`--wasm-pkg`가 없는 실행은 Native, 있는 실행은 fresh WASM이다. compare·standalone
 overlay·review를 모두 산출했고, 원본 크기 그대로 대조했다. 점수는 자동 보조값이며 승인 기준값이 아니다.

```sh
venv/bin/python scripts/visual_sweep.py --file-target <key> <input> <pdf> \
  --rhwp-bin <rhwp-maintainer> --pages <pages> --out <scratch-output>
# fresh WASM: 위 명령에 --wasm-pkg <wasm-maintainer> 추가
```

## Visual Sweep 범위와 보조 지표

| key / 입력 | 기준 PDF | 쪽 | Native pixel / ink (%) |
| --- | --- | --- | --- |
| hwpctl: `samples/hwpctl_API_v2.4.hwp` | `pdf/hwpctl_API_v2.4-hwp-2020.pdf` | 12,13,16-18,22,24,25,28,29,32,36,37,41,42,44,49 | 95.398 / 21.527 |
| rowbreak: `samples/rowbreak-problem-pages.hwp` | `pdf/rowbreak-problem-pages-hwp-2024.pdf` | 12 | 87.618 / 19.912 |
| reg56345: `samples/issue6111/56345_regulatory_impact_analysis.hwp` | `pdf/issue6111/56345_regulatory_impact_analysis-hwp-2020.pdf` | 11,19,20 | 91.803 / 14.884 |
| indicators: `tests/fixtures/stored_float_anchor_control/1351000_policy_indicators.hwp` | `pdf/pr7269/1351000_policy_indicators-2020.pdf` | 22-24 | 85.927 / 7.053 |

## 통합 PR과 원격 CI

[통합 PR #7270](https://github.com/edwardkim/rhwp/pull/7270)의 code candidate
`14eb5e73ac78ada2ae6ca4a843f99e2810b43891`에서 아래 pull_request workflow 모두 성공했다.

- [CI](https://github.com/edwardkim/rhwp/actions/runs/35423396970): Build & Test aggregate,
  archive A/B/C/D, lint, Native Skia, frontend package gate 성공.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/35423396984): Rust·Python·JavaScript 분석 및 GHAS check 성공.
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/35423396639) 성공.
- [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/35423396917) 성공.
- [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/35423396930) 성공.

이 문서와 `mydocs/orders/20260919.md`는 위 candidate 뒤에 붙이는 동일 PR의 trailing
문서 기록이다. source·test·golden·baseline은 바꾸지 않는다. trailing head의 fast-pass 및
required check 성공과 mergeability는 push 뒤 별도로 확인하며, merge 사실을 미리 기록하지 않는다.

## 대표 증적과 Merge 후 contributor PR comment 계획

최종 head CI와 merge 후 실제 merge SHA·CI URL, 보정의 소비 계약과 실제 검증 범위·남은 차이를
한국어로 설명하고 감사한다. [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결한다.
아래 **모든 review와 standalone overlay**를 실제 merge SHA에 고정한
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/pr7269_review/<filename>`
이미지로 직접 표시한다. UTF-8 파일과 `--body-file`로 게시 후 한국어·이미지 URL·SHA를 다시 읽어 확인한다.
원 PR은 통합 provenance를 설명하고 닫으며, 관련 이슈는 해결 범위만 반영한다.

- [native_hwpctl_review_025.png](../assets/pr7269_review/native_hwpctl_review_025.png)
- [native_hwpctl_overlay_025.png](../assets/pr7269_review/native_hwpctl_overlay_025.png)
- [wasm_hwpctl_review_025.png](../assets/pr7269_review/wasm_hwpctl_review_025.png)
- [wasm_hwpctl_overlay_025.png](../assets/pr7269_review/wasm_hwpctl_overlay_025.png)
- [native_hwpctl_review_017.png](../assets/pr7269_review/native_hwpctl_review_017.png)
- [native_hwpctl_overlay_017.png](../assets/pr7269_review/native_hwpctl_overlay_017.png)
- [wasm_hwpctl_review_017.png](../assets/pr7269_review/wasm_hwpctl_review_017.png)
- [wasm_hwpctl_overlay_017.png](../assets/pr7269_review/wasm_hwpctl_overlay_017.png)
- [wasm_hwpctl_review_018.png](../assets/pr7269_review/wasm_hwpctl_review_018.png)
- [wasm_hwpctl_overlay_018.png](../assets/pr7269_review/wasm_hwpctl_overlay_018.png)
- [native_reg56345_review_011.png](../assets/pr7269_review/native_reg56345_review_011.png)
- [native_reg56345_overlay_011.png](../assets/pr7269_review/native_reg56345_overlay_011.png)
- [wasm_reg56345_review_011.png](../assets/pr7269_review/wasm_reg56345_review_011.png)
- [wasm_reg56345_overlay_011.png](../assets/pr7269_review/wasm_reg56345_overlay_011.png)
- [wasm_reg56345_review_020.png](../assets/pr7269_review/wasm_reg56345_review_020.png)
- [wasm_reg56345_overlay_020.png](../assets/pr7269_review/wasm_reg56345_overlay_020.png)
- [wasm_indicators_review_023.png](../assets/pr7269_review/wasm_indicators_review_023.png)
- [wasm_indicators_overlay_023.png](../assets/pr7269_review/wasm_indicators_overlay_023.png)
