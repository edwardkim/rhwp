---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-19
---

# PR #7268 검토

## 판정

**승인 — 이 문서에 명시한 변경 범위의 통합 검토 완료.** 통합 제출 head는 `14eb5e73ac78ada2ae6ca4a843f99e2810b43891`이다. 원 PR의 green CI는 통합 head의 검증으로 세지 않는다.

## Metadata와 체리픽

- 원 PR: [#7268](https://github.com/edwardkim/rhwp/pull/7268) — 수정(renderer): 배치·정렬 run 폭의 정수 반올림 제거 — 줄 나눔과 같은 폭을 쓴다 (#7254)
- 작성자 `planet6897`, 코드 검토 reviewer `jangster77` 지정 완료.
- 원 head `fc77366c4e2b27a3af002353431566f545a6fe10`, base `devel`.
- 규모: 22 files, +3478 / -3189.
- 통합 base `62f1048f469f767a1c9f200209d23cf48b90016a`, branch `codex/planet-review-20260919`.
- #7267 → #7268 → #7269 순서로 충돌 없이 누적 적용. 최초 제품 head `9e58e5170`.
- 검증 전용 worktree `/private/tmp/rhwp-planet-verify-20260919`, target `/Users/tsjang/rhwp/target/planet-review-20260919`.

## 이번 회차 분석

**run 폭 반올림 제거와 정렬·부분 재페인트 정합**

줄 구성과 run 배치·다음 run 원점·탭 정렬·부분 재페인트가 소비하는 폭을 추적한다. golden SVG 7쪽과 text-overlap baseline 증가 2건의 실제 표시를 독립 PDF로 심사한다. 전 문서 PDF 일치나 모든 측정 경로의 통합을 주장하지 않는다.

분석 → 코드 수정·검증 → 결과보고 → 커밋 순서로 진행한다. 로컬 검증 결과와 남은 범위를 아래 기록했다.
원격 code candidate CI 뒤 같은 통합 PR의 trailing 문서 commit으로 보존한다. Native/fresh WASM Visual Sweep의 compare·standalone
overlay·review를 산출하고 직접 열어 판정했다. 입력과 PDF는 기존 추적 파일을 재사용하며,
없는 기준 PDF와 검증에 꼭 필요한 원문만 추가한다. 불필요한 로그·TSV·JSON은 커밋하지 않는다.

## 기준 PDF 보존과 검증 경계

원문 `samples/hwp3-sample10-hwp5.hwp`는 2024 저장본이다. MCP engine 2024,
Hancom 13.0.0.3901에서 763쪽 PDF를 생성했고 start/status/download의 성공 및 해시를 확인했다.
원본 PDF SHA-256은 `92b6625272c34db068a2ae273449c76caac008ce77ce46c9e228b4546b2aad76`이다.
90,780,477 bytes로 파일당 50MiB 제한을 넘으므로, 페이지를 생략하거나 다시 그리지 않고
PyMuPDF `insert_pdf`로 전체 763쪽을 아래 세 파일에 나눴다. 각 파일의 처음·둘째·마지막 쪽은
원본과 page box 및 raster bytes 일치를 확인했다. 합친 PDF의 원본 byte hash 일치를 뜻하지 않는다.

| `pdf/pr7268/` 파일 | 원본 쪽 | 보존 쪽수 | SHA-256 |
| --- | --- | --- | --- |
| `hwp3-sample10-hwp5-p001-300-2024.pdf` | 1–300 | 300 | `650f210c1d4127b404beab774e445651d463b00ac774c84f68210f42773a391a` |
| `hwp3-sample10-hwp5-p301-600-2024.pdf` | 301–600 | 300 | `ee223ef5a83e1349ed24af1a56fc3c786221df6c0f45c0100d65acd64d824e25` |
| `hwp3-sample10-hwp5-p601-763-2024.pdf` | 601–763 | 163 | `52d42e6c96ce8c3631302ea9787e505f9639fd38ab7f01dc41f3140e4ee8954f` |

Visual Sweep의 PDF 대조 대상은 차례 점 리더와 쪽번호가 있는 **2~6쪽**이다.
763쪽 전부의 시각 일치를 검증했다는 뜻이 아니다. 제목 장식선 길이·글꼴·꼬리말 위치 차이는
devel에서도 나타난다. HWPX 파생본에는 이 HWP용 PDF를 독립 정답으로 오인해 적용하지 않는다.

## 기준선 증가 독립 재현

Mac의 base/통합 Native CLI로 두 형식 전 763쪽의 `layout-anomaly --overlap-tolerance 0.01`
결과를 비교했다. 2px 기준을 새로 넘는 쌍은 HWP5 41개, HWPX 41개이며, 두 형식 모두
수정 전 겹침 폭 0.7867~1.7867px → 수정 후 2.0267~2.8267px다. 새로 발생한 쌍과
사라진 쌍은 각각 0개다. 위치는 2쪽 2개·3쪽 6개·4쪽 11개·5쪽 14개·6쪽 8개다.
쪽수·off-canvas·overflow·객체 overlap·empty-page 집계는 각 형식의 A/B에서 같다.
이는 원 PR의 1,139문서 전수 A/B 주장과 구분한 메인터너의 두 문서 실측이다.

## 용지 오류 보정과 golden 판독

`exam_kor.hwp`는 A3인데 기존 `pdf/exam_kor-hwp-2020.pdf`가 A4였다. 동일 원문
(저장 product 2022, version `12.0.0.4204`)을 MCP engine 2020으로 재변환해 **20쪽,
841×1190pt A3**를 확인하고 기존 PDF 경로를 교체했다. 새 PDF SHA-256은
`d67a5a0c759dcc8609226f89ef96de5339e3b41c43c2655be3824dcde1732f22`다.
MCP job `32eb57c5-11ec-4aec-a8f8-b1dd26109ff3`의 status 성공을 확인했다.
base와 최종 Native/fresh WASM의 1·6쪽을 같은 새 PDF로 다시 캡처했다.

`estimate_text_width_exact`는 기존 measurer의 소수점 결과와 custom tab 의미를 유지한다.
paragraph run/segment 폭·다음 run x·탭 정렬·부분 재페인트 rendering 쿼리가 같은 폭을 쓴다.
줄 나눔용 별도 unrounded helper와 이름만 보고 교체하지 않았다(custom tab 의미가 다르다).
`converge_cell_overflow_char_spacing` 등 남은 소비 경로까지 일괄 통합한 변경은 아니므로
**#7254는 열어 둔다**.

7개 golden의 독립 PDF를 직접 대조했다. 복학원서의 `line_order_overlap` 및
`column_line_band_drift` flag는 base부터 있으며, 장식선 노드 순서와 기존 상단/표 세로 차이다.
본문·프레임 누락이나 신규 줄 재배치는 없다. 해당 flag 페이지의 review/overlay도 보존한다.
`tbox`의 텍스트 x는 +0.33px 바뀌어 PDF 잔차 +0.16→+0.49px이고 0.5px 안이다.
표본 전체 PDF 일치로 확대하지 않는다. sample10 2~6쪽의 점 리더와 쪽번호를 직접 확인했다.
2px를 넘은 쌍은 이미 존재하던 tab 포함 run bbox/쪽번호이며 새 겹침 쌍은 없다.
라틴 글꼴, 하단 쪽번호의 아라비아/로마 표기, 기존 세로 잔차는 남는다.

`mydocs/tech/investigations/issue-4961/font_decision_trace_e2e.json`은 로그가 아니라
`tests/cases/issue_4961_font_decision_trace.rs`가 소비하는 기존 기능 fixture다.
따라서 expectedLayoutHash 변경은 유지하고, 이번 진단 로그·JSON·TSV는 추가하지 않는다.

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
| runwidth: `samples/table_scattered_header_rowbreak.hwp` | `pdf/table_scattered_header_rowbreak-2024.pdf` | 1 | 91.327 / 10.786 |
| sample10: `samples/hwp3-sample10-hwp5.hwp` | `pdf/pr7268/hwp3-sample10-hwp5-p001-300-2024.pdf` | 2-6 | 89.114 / 16.316 |
| tbox: `samples/table-in-tbox.hwp` | `pdf/table-in-tbox-hwp-2020.pdf` | 1 | 83.253 / 19.421 |
| answer: `samples/21_언어_기출_편집가능본.hwp` | `pdf/21_언어_기출_편집가능본-hwp-2020.pdf` | 1 | 88.256 / 12.702 |
| form: `samples/hwpx/form-002.hwpx` | `pdf/hwpx/form-002-2022.pdf` | 1 | 80.115 / 30.483 |
| ktx: `samples/KTX.hwp` | `pdf/KTX-2022.pdf` | 2 | 93.879 / 25.361 |
| aift: `samples/aift.hwp` | `pdf/aift-2022.pdf` | 4 | 90.015 / 16.129 |
| exam: `samples/exam_kor.hwp` | `pdf/exam_kor-hwp-2020.pdf` | 1,6 | 89.374 / 16.631 |
| bokhak: `samples/복학원서.hwp` | `pdf/복학원서-hwp-2020.pdf` | 1 | 92.748 / 46.255 |
| tabletext: `samples/hwpx/table-text.hwpx` | `pdf/hwpx/table-text-hwpx-2020.pdf` | 1 | 99.015 / 70.015 |
| issue157: `samples/hwpx/issue_157.hwpx` | `pdf/hwpx/issue_157-hwpx-2020.pdf` | 2 | 94.170 / 31.325 |

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
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/pr7268_review/<filename>`
이미지로 직접 표시한다. UTF-8 파일과 `--body-file`로 게시 후 한국어·이미지 URL·SHA를 다시 읽어 확인한다.
원 PR은 통합 provenance를 설명하고 닫으며, 관련 이슈는 해결 범위만 반영한다.

- [native_runwidth_review_001.png](../assets/pr7268_review/native_runwidth_review_001.png)
- [native_runwidth_overlay_001.png](../assets/pr7268_review/native_runwidth_overlay_001.png)
- [wasm_runwidth_review_001.png](../assets/pr7268_review/wasm_runwidth_review_001.png)
- [wasm_runwidth_overlay_001.png](../assets/pr7268_review/wasm_runwidth_overlay_001.png)
- [wasm_sample10_review_002.png](../assets/pr7268_review/wasm_sample10_review_002.png)
- [wasm_sample10_overlay_002.png](../assets/pr7268_review/wasm_sample10_overlay_002.png)
- [wasm_sample10_review_005.png](../assets/pr7268_review/wasm_sample10_review_005.png)
- [wasm_sample10_overlay_005.png](../assets/pr7268_review/wasm_sample10_overlay_005.png)
- [wasm_tbox_review_001.png](../assets/pr7268_review/wasm_tbox_review_001.png)
- [wasm_tbox_overlay_001.png](../assets/pr7268_review/wasm_tbox_overlay_001.png)
- [wasm_bokhak_review_001.png](../assets/pr7268_review/wasm_bokhak_review_001.png)
- [wasm_bokhak_overlay_001.png](../assets/pr7268_review/wasm_bokhak_overlay_001.png)
- [wasm_exam_review_006.png](../assets/pr7268_review/wasm_exam_review_006.png)
- [wasm_exam_overlay_006.png](../assets/pr7268_review/wasm_exam_overlay_006.png)
