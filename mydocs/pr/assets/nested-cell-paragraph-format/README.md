# 중첩 표 문단 서식 검증

## 입력과 재현

개인 문서 대신 저장소의 공개 재현 문서 두 개를 사용했다. 두 원본은 수동 XML로 만든 HWPX이며 한컴에서 저장한 원본이 아니다. 해당 파일을 한컴에서 독립적으로 인쇄한 기존 PDF와 비교한다. 원 보고 문서 전체의 재현·호환성을 주장하지 않는다.

| 이름 | 원본 | 대응 Print PDF | 검증 범위 |
| --- | --- | --- | --- |
| nested-same-line | [HWPX](../../../../tests/fixtures/issue7419-boundaries/nested-same-line.hwpx) | [PDF](../../../../pdf/issue7419/nested-same-line-print-2024.pdf) | 전체 1쪽 |
| nested-width-wrap | [HWPX](../../../../tests/fixtures/issue7419-boundaries/nested-width-wrap.hwpx) | [PDF](../../../../pdf/issue7419/nested-width-wrap-print-2024.pdf) | 전체 1쪽 |

Studio에서 원본을 열고 안쪽 표의 문단 또는 F5 셀 블록을 선택해 가운데·오른쪽 정렬을 적용한다. 후보에서는 안쪽 문단과 툴바가 바뀌며 Undo/Redo로 복원된다. 수정 전에는 중첩 대상 목록이 비어 명령이 적용되지 않는다. 별도 실제 WASM 회귀는 깊이 2·3, 셀 블록·같은 셀 문단 범위·캐럿, 제외 셀·빈 블록, 캡션, 잘못된 대상, HWP/HWPX 저장 왕복을 검사한다.

시각 비교의 후보 입력은 각 문서의 중첩 문단 4개에 center → right → left → justify를 적용한 후 원래 문단 모양 ID로 복원하고 HWPX로 저장한 `*-restored.hwpx`이다. 저장본을 다시 열어 그린다. base는 원본을 그대로 그린다. 따라서 이 비교의 주장은 **편집·복원·저장 왕복 후 원래 배치 보존**이다. 가운데·오른쪽 정렬 상태의 한컴 출력 일치를 주장하지 않는다.

## 출력 환경과 증거

- 비교 base·제품 source SHA, Native 실행 파일과 WASM 해시, 입력/PDF 해시, 실제 검사 수는 [evidence.json](evidence.json)에 기록한다.
- Native: `native-skia`, `release-test` CLI의 실제 Skia PNG, Print, compat 2024, 96 DPI.
- WASM: `scripts/wasm-pack-locked.sh --target web --out-dir pkg` 및 `wasm-opt`를 거친 패키지, Codex 인앱 브라우저 Canvas2D Print. 외부 Playwright 브라우저는 사용하지 않았다.
- 기준 PDF의 기록된 환경: 한컴 2024 **13.0.0.3901**, PrintMethod 0, `frame_print_to_pdf_ex_one_up`. [기존 출력 기록](../../../working/task_m100_7419_stage1.md)을 따른다. Windows OS 빌드는 기존 기록에 없다.
- 글꼴: 함초롬바탕 1.980 Build 20140819. PDF에서 사용한 47개 글리프의 윤곽·advance·left bearing과 `cvt `/`fpgm`/`prep`이 일치함을 확인했다. 당시 한컴에 설치된 전체 글꼴 파일과 동일하다는 주장은 아니다. 파일을 변경하거나 이 PR에 배포하지 않는다. 다운로드 출처와 SHA-256은 evidence.json에 고정한다.
- `*-native.tsv`, `*-wasm.tsv`는 후보, `*-base-native.tsv`, `*-base-wasm.tsv`는 base의 **전체 페이지** 결과다. raw 실루엣은 저장소의 2px 지표이며 정확한 픽셀 일치율이 아니다.
- 같은 입력·쪽의 `*-review.png`와 `*-overlay.png`를 함께 확인했다. 표의 포함 관계, 제목·본문·줄바꿈·앞뒤 간격을 대조했다. Native와 브라우저의 글리프 래스터화 차이는 남는다.

## 재실행 명령

작업공간의 고정 공용 target 경로를 `RHWP_REVIEW_TARGET`으로 지정한다. 아래 명령은 저장소 루트에서 실행한다. Studio 검사는 Node 22.23.4와 최신 web/Node WASM 바인딩을 사용했다. Node 바인딩 생성에는 Cargo.lock과 같은 wasm-bindgen CLI 0.2.127을 사용한다.

```sh
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all -- --check
cargo clippy --locked --target-dir "$RHWP_REVIEW_TARGET" -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir "$RHWP_REVIEW_TARGET" -- -D warnings
cargo build --locked --workspace --target-dir "$RHWP_REVIEW_TARGET"
cargo clippy --locked --workspace --all-targets --target-dir "$RHWP_REVIEW_TARGET" -- -D warnings
node scripts/rust-test-suite-manifest.mjs --check --base-ref db5cd5bcc60a69e42f419f18a7f57a33b6fb7fdc
CARGO_TARGET_DIR="$RHWP_REVIEW_TARGET" scripts/wasm-pack-locked.sh --target web --out-dir pkg
wasm-bindgen --target nodejs --out-dir pkg-node "$RHWP_REVIEW_TARGET/wasm32-unknown-unknown/release/rhwp.wasm"
npm --prefix rhwp-studio test
npm --prefix rhwp-studio run build
cargo nextest run --locked --cargo-profile release-test --target-dir "$RHWP_REVIEW_TARGET" --tests --no-fail-fast
cargo test --locked --profile release-test --target-dir "$RHWP_REVIEW_TARGET" --features native-skia --lib
node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --target-dir "$RHWP_REVIEW_TARGET" --features native-skia
node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --target-dir "$RHWP_REVIEW_TARGET" --features native-skia
cargo build --locked --profile release-test --target-dir "$RHWP_REVIEW_TARGET" --features native-skia --bin rhwp
```

Native Visual Sweep는 각 저장본과 대응 PDF를 `--compat 2024 --dpi 96`으로 전체 출력한다. `RHWP_FONT_PATH`에 위 해시의 글꼴 폴더를 지정한다. 브라우저는 원본을 불러와 위 편집·복원 순서를 실행하고 `renderPageToCanvasFilteredWithProfile(page, canvas, 1, 'all', 'print')`를 호출한다. 두 출력 모두 `scripts/visual_sweep.py --silhouette-only --png-pair <rhwp_png> <pdf_png> --out <result>`로 페이지별 TSV를 산출한다.

## 간격 변경의 추가 검사

공개 문서 두 개에서 문단 앞 간격 600, 줄간격 200, 두 값 동시 변경의 여섯 조합을 실제 WASM과 Studio CommandHistory로 검사한다. 변경을 유지한 HWP/HWPX 저장·재열기의 속성과 쪽수, Undo/Redo의 문단 속성 및 동적으로 캡처한 SVG 출력 복원을 확인한다. 이는 자체 일관성 검사이며 독립 한컴 Print 비교가 아니다. 실행한 source와 결과는 evidence.json에 기록한다.

## 한계

한컴 Windows 환경과 변환 서비스가 없어 **RHWP가 편집·저장한 HWP/HWPX를 한컴에서 새로 열고 Print하는 검사는 수행하지 못했다.** 기존 원본 Print PDF는 이 검사의 대체 증거가 아니다. 이 한계를 명시하는 조건으로 제출하며, 해당 항목은 미실행으로 남긴다. 한컴의 제품·버전·출력 경로별 차이는 [환경 의존성 안내](https://github.com/edwardkim/rhwp/wiki/한컴-PDF-환경-의존성)를 따른다.

다른 셀을 가로지르는 일반 텍스트 선택과 글자 모양까지 포함하는 중첩 셀 스타일 적용은 이번 범위에 포함하지 않는다. CanvasKit 및 실제 한컴 입력기의 동작을 이 검증으로 입증하지 않는다.
