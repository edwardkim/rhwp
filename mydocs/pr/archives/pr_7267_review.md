---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-19
---

# PR #7267 검토

## 판정

**승인 — 이 문서에 명시한 변경 범위의 통합 검토 완료.** 통합 제출 head는 `14eb5e73ac78ada2ae6ca4a843f99e2810b43891`이다. 원 PR의 green CI는 통합 head의 검증으로 세지 않는다.

## Metadata와 체리픽

- 원 PR: [#7267](https://github.com/edwardkim/rhwp/pull/7267) — fix(ole/emf): EMF-in-WMF 미리보기를 청크로 복원하고 EMF 페이지 변환을 적용 (#7266)
- 작성자 `planet6897`, 코드 검토 reviewer `jangster77` 지정 완료.
- 원 head `46054cf143f67c9ed9ea800219c09d2739d5b679`, base `devel`.
- 규모: 4 files, +560 / -12.
- 통합 base `62f1048f469f767a1c9f200209d23cf48b90016a`, branch `codex/planet-review-20260919`.
- #7267 → #7268 → #7269 순서로 충돌 없이 누적 적용. 최초 제품 head `9e58e5170`.
- 검증 전용 worktree `/private/tmp/rhwp-planet-verify-20260919`, target `/Users/tsjang/rhwp/target/planet-review-20260919`.

## 이번 회차 분석

**OLE 미리보기 청크 복원과 EMF 논리→장치 변환**

WMFC 주석 헤더가 DIB 바이트 사이에 남던 결함을 확인한다. 청크 복원→EMF parse→page/world/clip→RawSvg 소비 경로, 단일 청크·순수 EMF·CONTENTS 대체 경로와 기존 EMF path 표본을 검토한다. 관세청 로고와 156564340 포스터, 156627451 대조군을 동일 원문 한컴 PDF에 대조한다.

분석 → 코드 수정·검증 → 결과보고 → 커밋 순서로 진행한다. 로컬 검증 결과와 남은 범위를 아래 기록했다.
원격 code candidate CI 뒤 같은 통합 PR의 trailing 문서 commit으로 보존한다. Native/fresh WASM Visual Sweep의 compare·standalone
overlay·review를 산출하고 직접 열어 판정했다. 입력과 PDF는 기존 추적 파일을 재사용하며,
없는 기준 PDF와 검증에 꼭 필요한 원문만 추가한다. 불필요한 로그·TSV·JSON은 커밋하지 않는다.

## 독립 기준과 대조 결과

`2817919_emfplus_ole_preview.hwpx`의 저장 메타데이터는 Hancom Office 2020
`11.0.0.6402`다. MCP engine 2020(Hancom `11.0.0.9136`)으로 1쪽 PDF를 생성했다.
`pdf/pr7267/2817919_emfplus_ole_preview-2020.pdf`의 SHA-256은
`5c4cf526c5ce5a4fa1b84b9849a7550d5819f3ecb0591d3f1867d8ebcf4080ff`다.
입력은 기존 samples 파일을 그대로 사용했고, start/status/download 성공과 해시를 확인했다.

최초 Native/fresh WASM에서 관세청 로고가 깨진 띠 대신 전체 심벌·문자로 복원됨을 확인했다.
본문의 Arial/한컴 기준 글꼴 차이, 줄바꿈·테두리 잔차는 devel에도 남아 있으며 이 로고 수정의
전체 문서 일치로 확대하지 않는다. 포스터 1·4쪽 및 quantum 15쪽 전체가 대조 범위다.

## 구현 주장과 실제 소비 경로

`ole_container`의 WMFC 주석 청크 해제 → EMF signature/전체 길이 확인 → EMF record parse →
`player`의 page transform/world transform/clip → RawSvg 순서를 대조했다. 손상/비지원 청크는
기존 fallback을 유지하며, 순수 EMF와 4-byte length-prefix CONTENTS 경로는 별도 회귀 검사가 있다.
최종 캡처에서도 로고의 전 높이·윤곽·문자가 보존되고 기존 포스터 그림의 신규 누락은 없다.
3문서 18쪽의 공개 증거만 검증했으며 작성자의 40문서 주장 전체를 재실행했다고 쓰지 않는다.
#7266의 WMFC/CONTENTS 미리보기 복원 범위와 본문 글꼴·테두리의 기존 차이는 구분한다.

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
| logo: `samples/issue5637/2817919_emfplus_ole_preview.hwpx` | `pdf/pr7267/2817919_emfplus_ole_preview-2020.pdf` | 1 | 89.100 / 10.589 |
| poster: `samples/issue6896/156564340-ip-dispute-mediation.hwpx` | `pdf/156564340-ip-dispute-mediation-2020.pdf` | 1,4 | 91.055 / 47.641 |
| quantum: `samples/issue6866/156627451-quantum-science-press-note.hwpx` | `pdf/156627451-quantum-science-press-note-2020.pdf` | 1-15 | 90.596 / 28.885 |

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
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/pr7267_review/<filename>`
이미지로 직접 표시한다. UTF-8 파일과 `--body-file`로 게시 후 한국어·이미지 URL·SHA를 다시 읽어 확인한다.
원 PR은 통합 provenance를 설명하고 닫으며, 관련 이슈는 해결 범위만 반영한다.

- [native_logo_review_001.png](../assets/pr7267_review/native_logo_review_001.png)
- [native_logo_overlay_001.png](../assets/pr7267_review/native_logo_overlay_001.png)
- [wasm_logo_review_001.png](../assets/pr7267_review/wasm_logo_review_001.png)
- [wasm_logo_overlay_001.png](../assets/pr7267_review/wasm_logo_overlay_001.png)
- [wasm_poster_review_004.png](../assets/pr7267_review/wasm_poster_review_004.png)
- [wasm_poster_overlay_004.png](../assets/pr7267_review/wasm_poster_overlay_004.png)
- [wasm_quantum_review_012.png](../assets/pr7267_review/wasm_quantum_review_012.png)
- [wasm_quantum_overlay_012.png](../assets/pr7267_review/wasm_quantum_overlay_012.png)
