---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-18
---

# PR #7242 검토

**검토 승인 — 표·Footer 위치와 남은 글자·문단 테두리 보류 사유를 메인터너 보정으로 해소.** #7239·#7240 통합 후보에 누적 적용했다.

최신 판정은 아래 renderer 보정 회차를 따른다. 이전 입력 정상화만으로 범위를 좁혀 승인한 판정은 이번 판정으로 대체한다. 기존 기록·실패 PNG는 수정 전 이력으로 보존한다.

## 현재 통합 코드와 판정 범위 — 2026-09-18

- **최종 판정: 승인 유지.** 테두리 보정 commit `bb401f0a7` 위에 #7243 이어받기 높이 보정을
  적용했다. #7242 원 입력과 #6044 실물의 전체 33쪽 SVG는 이번 보정 전후 바이트 동일하며
  원 입력 1쪽·실물 20쪽 Native/fresh WASM compare·overlay·review를 새로 확인했다.
- #7239·#7240·#7243도 각각의 보정 범위를 승인했다. 최신 통합 코드의 정확한 source/binary hash,
  전체 회귀 **10,023 passed / 50 skipped**, Native Skia **4,112 passed / 13 ignored**,
  그림 **2 passed**, 직접 PDF **4 passed**, lint·build·시각 증거는
  [#7243 최종 실행·증적](pr_7243_review.md#최종-실행증적)에 연결한다.
- 아래 #7242의 원 보정·실행·캡처 기록은 당시 증거로 보존한다. 원격 최종 head의 CI·mergeability는
  별도 확인 대상이며, 이번 로컬 보정에서 push·merge하지 않았다.

## 2026-09-18 후속 보정 — 실제 테두리 누락·잘림 수정

### 원인, 공통 소유 범위와 수정

입력 교체로 Footer 위치가 맞아도 표 줄의 글자 테두리와 문단 외곽선의 차이는 남아 있었다.
이전 회차의 범위를 좁힌 승인은 이 차이의 해결 증거가 아니었다. 이번에는 정상 저장된
같은 파일·같은 PDF를 유지하고 renderer를 수정한다.

1. `standalone_table_char_border_fill`이 **문단 전체에 글자가 있는지**로 표 장식을 생략했다.
   실제 표 소속 저장 줄과 Footer 줄이 다른데도 표의 글자 테두리가 누락됐다.
   실제 배치가 사용하는 `control_line_seg_index`로 소속 줄을 선택하고, 그 줄의
   인접 공백만 같은 글자 테두리 범위에 포함한다. 같은 줄의 가시 텍스트·복수 개체 등
   기존 별도 줄 소유 경로를 추가 장식하지 않는다.
2. 표 제어 뒤 글자의 위치로 스타일을 조회하면 뒤 공백의 스타일이 표의 스타일을 가렸다.
   `control_utf16_positions`의 **원 제어 슬롯**으로 글자모양을 조회한다. 테두리 없는 공백은
   장식 확장을 중단하되 표 자체의 테두리는 유지한다.
3. `para_border_ranges`가 같은 문단의 text/table/text 조각도 `border_connect`로 판단했다.
   이 속성은 서로 다른 문단의 연결 조건이다. 동일 문단 조각은 같은 외곽선으로 합친다.
4. 58개 공백뿐인 저장 줄을 정렬 공통의 자동 음수 자간 처리로 본문 폭 384px에 압축했다.
   실제 문단 정렬은 **LEFT**다. 자동 줄바꿈으로 소비된 공백 전용 중간 줄은 자연 공백 폭을
   유지한다. 문단 끝·강제 개행의 공백은 작성한 내용이므로 원래 줄 폭 맞춤 계약을 유지한다.
   명시적인 양쪽/배분 정렬과 인라인 개체 줄의 기존 규칙도 변경하지 않는다.

생산·소비 연결: 원 제어 슬롯의 글자 스타일 + 실제 저장 줄의 공백 advance → paint 전용
`TableCharBorder` → 최종 물리 표 bbox를 사용하는 `paint_standalone_table_char_border`.
장식 폭·여백을 행 높이/예약 높이에 되먹이지 않는다. 문단 범위는 기존 text/table layout의
실제 점유 끝을 수집한 뒤 같은 문단 소유끼리 합친다. 공백 줄의 추가 자간은 실제 TextRun
advance와 글자 테두리가 함께 소비한다. 표 높이·Footer 기준선·문단 뒤 간격은 기존 검사로 보존한다.
분할 컷·행 예약·이월 결정은 바꾸지 않는다.

빈 선행 문단의 테두리 생성 순서도 조사했으나, 이 입력은 수정 전에도 테두리가 존재했다.
해당 실험 변경은 제외했다. 빈 문단 검사는 정상 대조군이며 결함을 검출한 검사로 세지 않는다.

### 독립 PDF 증거와 수정 전후 검사

[동일 한컴 PDF](../../../pdf/pr7242/native-8-0-2020.pdf)의 vector stroke 좌표를 96dpi로 환산했다.
표 글자 테두리는 x=48..446.08, 하단 y=331.133px이며 **표 자체 외곽** y=312.417px와 구분한다.
표 줄의 두 공백을 포함하고, 그 뒤 58개 공백 줄의 글자 테두리 폭은 387.36px다.
문단 외곽은 y=127.175..365.687px다. 별도 글꼴/가느다란 stroke의 래스터 차이를
테두리 누락과 혼동하지 않고, PDF 좌표 오차 1px 이내로 검사한다.

- 기존 검사를 포함한 최초 9개 중 테두리 소유·원 제어 슬롯·같은 문단 외곽선 **3개는 수정 전 FAIL**, 수정 후 PASS.
  기존 4개와 가시 텍스트가 같은 줄인 반례·빈 문단 정상 대조는 통과했다.
- 공백 폭 검사는 별도로 **수정 전 FAIL(width=384px) → 수정 후 PASS**를 확인했다.
- 실행 명령: `node scripts/run-rust-test.mjs stored_table_text_tail`. 최초 수정 전 실행은
  `c877f6e48`의 renderer 3파일을 검증 checkout에 넣고 같은 검사를 실행했다.
  빌드는 성공했으며 실제 테두리 assertion으로 실패했고, 보정 소스 복원 후 재실행했다.
- 최종 `stored_table_text_tail`: **12 passed**. 기존 #7200의 단독 표 장식 여백·장식 끔,
  중첩 표/셀 정렬 대조 `stored_nested_content_flow`: 최종 전체 nextest에서 **7 passed**.
- 기존 샘플·PDF·수치 baseline·허용치는 이번 회차에서 바꾸지 않았다. 이전 불일치 입력은
  원 PR head 및 이전 회차에 기록한 Git SHA로 보존한다. 잘못된 LineSeg의 일반 자동 복구를
  구현했다고 주장하지 않는다.

### 전체 회귀에서 검출한 반례와 추가 보정

첫 전체 실행은 **10,018 passed / 1 failed / 50 skipped**였다. 실패한
`svg_snapshot::issue_157_page_1`은 공백으로 만든 밑줄 끝점이 747→748px로 길어진 실제 회귀다.
독립 기준 `pdf/hwpx/issue_157-2022.pdf` 2쪽은 x2=559.859pt, 즉 746.479px이므로
이번 변경이 오차를 키웠다. 해당 golden은 갱신하지 않았다.

잘못된 가정은 공백만 있으면 모두 같은 폭 계약이라는 것이었다. 자동 줄바꿈이 소비한
구분 공백과 문단 끝·강제 개행의 작성 공백을 기존 `is_last_line_of_para`/`has_forced_break`로
구분했다. 후자는 종전 줄 폭 맞춤을 유지한다. 공개 #157의 최종 SVG 밑줄 좌표 검사도 추가해
748px에서 **FAIL → 747px에서 PASS**를 확인했다. #7242의 중간 공백 줄 폭은 보존된다.
이 중간 후보의 focused는 11개 통과였다. 추가 보정은 아래 최종 검증과 시각 증거에 포함했다.

저장 줄 정보가 없는 개체 전용 문단도 별도로 대조했다. 빈 문자열과 `U+FFFC`는 모두
개체만 있는 문단인데, 새 fallback의 `trim().is_empty()`는 자리표시자를 가시 텍스트로
잘못 분류했다. 기존 비가시 제어 문자·자리표시자 계약을 유지하도록 수정했다.
정식 테스트에 넣은 동일 검사 본문을 변경 전 debug 라이브러리에 직접 연결해 실행했을 때
빈 문자열은 통과하고 `U+FFFC`의 글자 테두리가 누락되어 **1 failed**였다.
이 경계까지 추가한 최종 focused 검사 **12개가 통과**했다. 아래 최종 공통 검증과 캡처는
이 마지막 보완까지 포함한 source를 사용한다. 두 번째 중간 후보의 10,020 passed 및
Native Skia 통과는 마지막 변경의 검증으로 재사용하지 않는다.

### 최종 실행·증적

- 검증 source: `c877f6e48` 위의 이번 renderer 3파일·정식 test 변경. 검증 checkout의
  `src/`, `crates/`, `tests/cases/`, `Cargo.toml`, `Cargo.lock`을 작업 브랜치와 바이트 대조했다.
  검증 checkout의 오래된 Git HEAD를 이번 source SHA로 사용하지 않는다.
- base: `236a601da803b53429e9090eef652c661dd3bfe2`, branch:
  `codex/pr7239-7240-review-20260917`. Mac arm64, `DEVELOPER_DIR=/Library/Developer/CommandLineTools`,
  `CARGO_TARGET_DIR=/Users/tsjang/rhwp/target/pr7239-7240-review-20260917`.
- 새 Native binary SHA256: `91e84d346e42592e9c45fd2c2116ea796718cac64845b9d213b1e435aecfce0a`.
- fresh WASM SHA256: `d96c54eeed2fca9f98841db65e91a1301a603ea4c4c040cd981e9a4c9583dc59`;
  JS `a7353a7603b7e07db2d33ff93fff6b213ea79e01da91c190cbb607e752c6b5a7`.
  `scripts/wasm-pack-locked.sh --target web --out-dir <scratch>/wasm-border-final --no-opt` 성공.
  host 빌드이며 wasm-opt 실행으로 보고하지 않는다.
- 보안 회귀의 추가 문서 입력은 지침에 따라
  `RHWP_SECURITY_SWEEP_SAMPLES_JSON='["samples/stored-table-text-tail/native-8-0.hwpx"]'`로 지정했다.
최종 필수 검증은 모두 exit 0이다. 이전 중간 후보의 통과 결과를 재사용하지 않았다.

| 명령·검사 | 최종 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --locked -- -D warnings` | 통과 |
| `cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown -- -D warnings` | 통과 |
| `cargo build --locked --workspace` | 통과 |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | 통과 |
| `node scripts/rust-test-suite-manifest.mjs --check --base-ref 236a601da803b53429e9090eef652c661dd3bfe2` | 통과 |
| `cargo nextest run --locked --cargo-profile release-test --tests --no-fail-fast` | **10,021 passed / 50 skipped / 0 failed** |
| `cargo test --locked --profile release-test --features native-skia --lib` | **4,112 passed / 13 ignored / 0 failed** (workspace 라이브러리 합계) |
| `node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --target-dir <review-target> --features native-skia` | **2 passed** |
| `node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --target-dir <review-target> --features native-skia` | **4 passed** |

`<review-target>`은 위의 전용 target 경로다. 문서 메타데이터·로컬 증적 링크·`git diff --check`도
통과했다. 로그·JSON·TSV·generated suite는 커밋하지 않는다. 기존 입력·PDF·golden·래칫을
변경하지 않고 코드·정식 검사·검토 기록·최종 PNG만 이번 보정 커밋에 포함한다.

코드·test SHA256:

| 파일 | SHA256 |
| --- | --- |
| `src/renderer/layout.rs` | `b685c9a0b39d4f214e4693c2c2e5f3ee205efaf03d92693b7153fb61c375b38d` |
| `src/renderer/layout/table_layout.rs` | `0ffe465c96a48dbcd0001238af7adf8c3dec10eb504e16f6e4ee231934562718` |
| `src/renderer/layout/paragraph_layout.rs` | `b8f55e6593370a2cbf1f39ffe362bcc6db6347529ccab6168a2096b2e6afcbd7` |
| `tests/cases/stored_table_text_tail.rs` | `c7243f63f234e1b41831cca28a24f68c6669b9b8ee1ef6bd54ea67a0a36c189b` |

최종 코드로 Native/fresh WASM 각 9문서·9쪽의 compare·standalone overlay·review를 재생성했다.
각 페이지의 review 또는 overlay를 직접 판독했다. Native 명령은 다음과 같고,
WASM 실행에는 `--wasm-pkg <scratch>/wasm-border-final`을 추가했다.

```sh
venv/bin/python scripts/visual_sweep.py --file-target <key> <input> <pdf> \
  --rhwp-bin <scratch>/rhwp-border-final --pages <page> --dpi 96 \
  --out <scratch>/border-final-native/<key>
```

| 입력·PDF·쪽 | Native | fresh WASM |
| --- | --- | --- |
| [issue157](../../../samples/hwpx/issue_157.hwpx) / [PDF](../../../pdf/hwpx/issue_157-2022.pdf) p2 | [compare](../assets/pr7242_review/native_border_issue157_compare_002.png) · [overlay](../assets/pr7242_review/native_border_issue157_overlay_002.png) · [review](../assets/pr7242_review/native_border_issue157_review_002.png) | [compare](../assets/pr7242_review/wasm_border_issue157_compare_002.png) · [overlay](../assets/pr7242_review/wasm_border_issue157_overlay_002.png) · [review](../assets/pr7242_review/wasm_border_issue157_review_002.png) |
| [tail](../../../samples/stored-table-text-tail/native-8-0.hwpx) / [PDF](../../../pdf/pr7242/native-8-0-2020.pdf) p1 | [compare](../assets/pr7242_review/native_border_tail_compare_001.png) · [overlay](../assets/pr7242_review/native_border_tail_overlay_001.png) · [review](../assets/pr7242_review/native_border_tail_review_001.png) | [compare](../assets/pr7242_review/wasm_border_tail_compare_001.png) · [overlay](../assets/pr7242_review/wasm_border_tail_overlay_001.png) · [review](../assets/pr7242_review/wasm_border_tail_review_001.png) |
| [real_tail](../../../samples/issue6044/156513948.hwpx) / [PDF](../../../pdf/pr6940-156513948-source-2020.pdf) p20 | [compare](../assets/pr7242_review/native_border_real_tail_compare_020.png) · [overlay](../assets/pr7242_review/native_border_real_tail_overlay_020.png) · [review](../assets/pr7242_review/native_border_real_tail_review_020.png) | [compare](../assets/pr7242_review/wasm_border_real_tail_compare_020.png) · [overlay](../assets/pr7242_review/wasm_border_real_tail_overlay_020.png) · [review](../assets/pr7242_review/wasm_border_real_tail_review_020.png) |
| [width_top](../../../samples/stored-nested-content-flow/width-top.hwp) / [PDF](../../../pdf/pr7200/width-top-recomposed-2020.pdf) p1 | [compare](../assets/pr7242_review/native_border_width_top_compare_001.png) · [overlay](../assets/pr7242_review/native_border_width_top_overlay_001.png) · [review](../assets/pr7242_review/native_border_width_top_review_001.png) | [compare](../assets/pr7242_review/wasm_border_width_top_compare_001.png) · [overlay](../assets/pr7242_review/wasm_border_width_top_overlay_001.png) · [review](../assets/pr7242_review/wasm_border_width_top_review_001.png) |
| [margin-min-700](../../../tests/fixtures/pr7200_hancom_recomposed/margin-min-700.hwp) / [PDF](../../../pdf/pr7200/margin-min-700-2020.pdf) p1 | [compare](../assets/pr7242_review/native_border_margin-min-700_compare_001.png) · [overlay](../assets/pr7242_review/native_border_margin-min-700_overlay_001.png) · [review](../assets/pr7242_review/native_border_margin-min-700_review_001.png) | [compare](../assets/pr7242_review/wasm_border_margin-min-700_compare_001.png) · [overlay](../assets/pr7242_review/wasm_border_margin-min-700_overlay_001.png) · [review](../assets/pr7242_review/wasm_border_margin-min-700_review_001.png) |
| [margin-min-800](../../../tests/fixtures/pr7200_hancom_recomposed/margin-min-800.hwp) / [PDF](../../../pdf/pr7200/margin-min-800-2020.pdf) p1 | [compare](../assets/pr7242_review/native_border_margin-min-800_compare_001.png) · [overlay](../assets/pr7242_review/native_border_margin-min-800_overlay_001.png) · [review](../assets/pr7242_review/native_border_margin-min-800_review_001.png) | [compare](../assets/pr7242_review/wasm_border_margin-min-800_compare_001.png) · [overlay](../assets/pr7242_review/wasm_border_margin-min-800_overlay_001.png) · [review](../assets/pr7242_review/wasm_border_margin-min-800_review_001.png) |
| [margin-both-1000](../../../tests/fixtures/pr7200_hancom_recomposed/margin-both-1000.hwp) / [PDF](../../../pdf/pr7200/margin-both-1000-2020.pdf) p1 | [compare](../assets/pr7242_review/native_border_margin-both-1000_compare_001.png) · [overlay](../assets/pr7242_review/native_border_margin-both-1000_overlay_001.png) · [review](../assets/pr7242_review/native_border_margin-both-1000_review_001.png) | [compare](../assets/pr7242_review/wasm_border_margin-both-1000_compare_001.png) · [overlay](../assets/pr7242_review/wasm_border_margin-both-1000_overlay_001.png) · [review](../assets/pr7242_review/wasm_border_margin-both-1000_review_001.png) |
| [margin-both-2000](../../../tests/fixtures/pr7200_hancom_recomposed/margin-both-2000.hwp) / [PDF](../../../pdf/pr7200/margin-both-2000-2020.pdf) p1 | [compare](../assets/pr7242_review/native_border_margin-both-2000_compare_001.png) · [overlay](../assets/pr7242_review/native_border_margin-both-2000_overlay_001.png) · [review](../assets/pr7242_review/native_border_margin-both-2000_review_001.png) | [compare](../assets/pr7242_review/wasm_border_margin-both-2000_compare_001.png) · [overlay](../assets/pr7242_review/wasm_border_margin-both-2000_overlay_001.png) · [review](../assets/pr7242_review/wasm_border_margin-both-2000_review_001.png) |
| [host-char-border-off](../../../tests/fixtures/pr7200_hancom_recomposed/host-char-border-off.hwpx) / [PDF](../../../pdf/pr7200/host-char-border-off-2020.pdf) p1 | [compare](../assets/pr7242_review/native_border_host-char-border-off_compare_001.png) · [overlay](../assets/pr7242_review/native_border_host-char-border-off_overlay_001.png) · [review](../assets/pr7242_review/native_border_host-char-border-off_review_001.png) | [compare](../assets/pr7242_review/wasm_border_host-char-border-off_compare_001.png) · [overlay](../assets/pr7242_review/wasm_border_host-char-border-off_overlay_001.png) · [review](../assets/pr7242_review/wasm_border_host-char-border-off_review_001.png) |

직접 판독 결과: 공개 tail의 8개 셀 내용, 표 외곽, 표 줄 두 공백까지의 글자 테두리,
연속 문단 외곽선과 Footer가 유지된다. 이번에 누락·분리됐던 두 테두리를 복구했다.
Footer baseline은 354.213px(PDF 354.489px), 표 하단은 312.160px(PDF 약 312.417px),
표 글자 테두리 하단은 331.040px(PDF 331.133px)다. 공백 58자의 폭은
386.667px(PDF 장식 폭 387.360px)으로 본문 폭 384px에 잘리지 않는다.
얇은 선의 진하기·글리프·부분적인 subpixel 차이는 남고 전체 화소 일치를 주장하지 않는다.

6개 #7200 대조군에서는 표 장식 여백과 장식 끔의 적용 경계를 함께 확인했다.
`host-char-border-off`의 기존 글줄/표 하단 위치 차이는 남아 있으며, 전체 PDF 일치 대조군으로
표현하지 않는다. 실제 문서 156513948 20쪽은 표 이후 주석이 겹치지 않고 유지되지만
기존 글꼴·셀 안 숫자 위치·테두리·쪽 번호 차이가 남는다. 자동 ink match는 승인 기준이 아니다.
이번 #7242 테두리 결함의 해결과 이들 문서의 전체 fidelity 완료를 구분한다.
#157은 밑줄 끝점 747px로 기존 golden을 보존했고, Native/WASM에서 주주총회 참석장·위임장의
내용과 표·밑줄 배치를 확인했다. 기존 글꼴·일부 표 위치 차이는 남는다. #157 외 8페이지는
추가 문단 끝 보정 전후의 재캡처 review PNG가 화소 기준으로 동일하다.
최종 PNG 54개는 개체 자리표시자 fallback 보완 뒤의 binary/package로도 전부 재생성했다.
저장 줄을 가진 위 9페이지는 직전 판독 이미지와 화소가 같으며, 공개 tail의 최종 Native overlay와
fresh WASM review도 다시 직접 확인했다. 저장 줄 없는 경계는 정식 회귀 검사로 별도 확인했다.

**최종 판정: #7242 검토 승인.** 정상 공개 입력의 표·Footer 위치와 글자/문단 테두리 누락·잘림을
해소했고, 변경 전 실패·변경 후 통과 및 정상 대조군을 확인했다. 최종 소스의 전체 회귀,
Native Skia, lint와 fresh WASM 시각 증거까지 완료했다. 남은 글꼴·래스터·다른 기존 문서의
세부 위치 차이는 위에 명시했다. 임의의 잘못된 저장 정보 자동 복구나 통합 PR 전체 승인으로
범위를 확대하지 않는다. 이번 회차는 로컬 메인터너 보정이며 원격 push·comment·merge 결과가 아니다.

## 2026-09-18 메인터너 보정 — 공개 샘플 자체의 정상화

### 원인과 수정

수동 합성 원본의 누락된 호스트 줄과 셀 메트릭이 PDF와 큰 위치 차이를 만들었다.
유효하지 않은 저장 정보를 수용하도록 renderer 조건을 완화하지 않는다. 독립적으로
재저장·재변환까지 완료한 한컴 파일을 **기존 `native-8-0.hwpx` 경로에 적용**했다.
정상 대조군만 추가한 이전 회차와 달리, 이제 제출·검증 대상 파일 자체가 정상 저장본이다.
중복 `hancom-resaved.hwpx`는 제거했다. 이름만 바꾼 새 파일이나 중복 PDF는 추가하지 않았다.

- 교체 전 SHA256: `8f569cf88da9b719db5d593d8c5e7afcdb38140f28957a754b1e5939743530c5`.
- 교체 후 SHA256: `3aa0379ab1b4d158800d33c73ae26eed909e5f08e3614dca228044780c8c5e64`.
- 이전 원본: commit `086078148db2a9f66b21c98a2f3d356c40365fec`의 동일 경로.
  원 PR head `03ba57cb804610e899a3e6e1ce2cb3cfcb21df6e`에도 바이트가 동일한 원본이 남아 있다.
  아래 `native_tail_*`/`wasm_tail_*` 실패 증거도 유지한다.
- 한컴 재저장 job 및 원본/정상본의 동일 PDF raster 확인은 아래 이전 회차에 기록되어 있다.
  PDF는 `pdf/pr7242/native-8-0-2020.pdf`를 재사용했다.
- 경계 테스트는 정상 파일에서 수동 LineSeg·작은 선언 높이를 **메모리 안에서 생성**한다.
  native/pure, 2/8문단, gap, 문단 뒤 간격, 도형 공존의 기존 assertion을 유지했다.
  공개 샘플을 직접 읽는 PDF 좌표 검사는 그대로 유지하며 경로만 통일했다.
- 메모리 경계 입력에서만 표 뒤 공백을 70→2개로 줄여, 15자+표 제어 8단위+2자 뒤의
  `textpos=25`가 Footer 자체의 시작이 되도록 했다. 공개 파일의 70개 공백·4줄은 보존한다.
  가로 시작점 assertion 추가 후 보정 전 **1 fail / 3 pass**(x399), 보정 후 **4 pass**다.
  이는 테스트 입력 구성의 수정 전후 증거이며 renderer 결함을 고쳤다는 증거가 아니다.
- `oracle_page_count_baseline.tsv`에 기존 공개 경로의 PDF 1쪽/rhwp 1쪽 한 행을 추가했다.
  새 로그성 TSV가 아니라 CI가 소비하는 기존 페이지 수 회귀 원장이다. 허용치 완화는 없다.

이는 입력과 검증 설계의 메인터너 보정이다. renderer 소스 변경은 없다. 교체 전의 잘못된
저장 정보를 자동 재조판하는 기능까지 고쳤다고 주장하지 않는다. 진단 중 한컴 줄 정보와
낮춘 선언 높이를 혼합한 파일도 만들었으나, 그 합성 입력의 8.43px 높이 차이를 근거로
기존 문서에 영향을 주는 마지막 줄간격 정책을 바꾸지 않았다. 해당 진단 파일은 커밋하지 않는다.

### 독립 기대값과 실제 결과

| 항목 | 교체 전 공개 입력 | 정상 저장본의 base → 통합 코드 | 한컴 PDF |
| --- | --- | --- | --- |
| 호스트 줄 | 수동 3줄, textpos 83 누락 | 실제 4줄, 마지막 textpos 83 | Footer가 네 번째 글줄 |
| Footer | x399, y310.0 text bbox | 기준선 367.92 → 354.2133px | 기준선 354.4891px |
| 표 실제 외곽 | 높이 170.7px, 위치 불일치 | bbox y140.5333 / h171.6267, base와 동일 | 외곽 y140.77..312.42px |
| 페이지·내용 | 1쪽 | 1쪽, Cell 1..8 및 Footer 유지 | 1쪽 |

PDF 좌표 검사는 이전 회차에 base `layout.rs`에서 의도한 기준선 assertion으로 FAIL,
통합 코드에서 PASS를 확인했다. 이번 교체 파일은 그때 검증한 정상본과 바이트가 동일하며
동일 assertion을 기존 샘플 경로에 적용했다. 이번에도 base/통합 binary의 같은 입력을
대조해 표 bbox는 동일하고 Footer text bbox가 y356.6→342.9px로 이동함을 재확인했다.

문단/문자 테두리의 일부 길이·위치와 글꼴 raster 차이는 남는다. 표 자체와 문단 테두리를
구분해 판독했으며 전체 화소 일치로 보고하지 않는다. #7242의 표 뒤 글줄 흐름 계약과
공개 입력의 정상성에 대한 증거다. 비공개 원 실패 문서까지 검증했다는 뜻은 아니다.

### 이번 회차 검증과 증적

제품 코드: `086078148db2a9f66b21c98a2f3d356c40365fec`와 동일.
변경 대상은 공개 입력·테스트 helper·문서·PNG이며 renderer diff는 없다.
Native SHA256 `4467723c7a604d708a4aefb6e7cefdaacae2ca5391c46555a32ce6a1c78a0ac0`,
fresh WASM SHA256 `8dc9187e1a4b884f127d11e0c4c496cacbcbec57ebeac521980247a095be9b3b`.
이 제품 소스로 이전 회차에 새로 빌드한 산출물을 사용했고 이번 입력으로 **캡처는 다시 실행**했다.
이번 회차에 WASM 빌드를 또 실행했다고 기록하지 않는다.

명령: `venv/bin/python scripts/visual_sweep.py --file-target tail samples/stored-table-text-tail/native-8-0.hwpx pdf/pr7242/native-8-0-2020.pdf --rhwp-bin <scratch>/rhwp-isolation-final26 --pages 1 --dpi 96 --out <scratch>/...`.
WASM은 같은 명령에 `--wasm-pkg <scratch>/wasm-recovery-26`을 추가했다.
실물 대조는 `samples/issue6044/156513948.hwpx`, `pdf/pr6940-156513948-source-2020.pdf`, 20쪽이다.
compare·standalone overlay·review를 직접 확인하고 아래에 보존한다.

| 입력·쪽 | Native | WASM |
| --- | --- | --- |
| 정상화한 공개 입력 p1 | [compare](../assets/pr7242_review/native_fixed_tail_compare_001.png) · [overlay](../assets/pr7242_review/native_fixed_tail_overlay_001.png) · [review](../assets/pr7242_review/native_fixed_tail_review_001.png) | [compare](../assets/pr7242_review/wasm_fixed_tail_compare_001.png) · [overlay](../assets/pr7242_review/wasm_fixed_tail_overlay_001.png) · [review](../assets/pr7242_review/wasm_fixed_tail_review_001.png) |
| 실물 대조 p20 | [compare](../assets/pr7242_review/native_fixed_real_tail_compare_020.png) · [overlay](../assets/pr7242_review/native_fixed_real_tail_overlay_020.png) · [review](../assets/pr7242_review/native_fixed_real_tail_review_020.png) | [compare](../assets/pr7242_review/wasm_fixed_real_tail_compare_020.png) · [overlay](../assets/pr7242_review/wasm_fixed_real_tail_overlay_020.png) · [review](../assets/pr7242_review/wasm_fixed_real_tail_review_020.png) |

- `node scripts/run-rust-test.mjs stored_table_text_tail`: 최종 **4 passed**.
- fmt, Native Clippy, WASM lib Clippy, workspace build, workspace all-target Clippy 및
  suite manifest의 고정 base `236a601da` 비교: **모두 통과**.
- release-test 코퍼스 필터(IR/overflow-cell/off-canvas/text-overlap/oracle/security):
  **69 passed / 0 failed**, 96.570초(컴파일 제외). 보안 입력 환경변수에 교체한 공개 경로를 명시했다.
- oracle 원장 새 행을 포함한 재실행: **16 passed**, exit 0. partition 13에 nextest LEAK 표시가
  1회 있었고 해당 partition만 재실행해 **1 passed / LEAK 없음**, exit 0을 확인했다.
  프로세스 정리 경고의 원인까지 해결했다고 주장하지 않는다. 공개 파일의 `page_count() == 1`도 위 focused에서 통과했다.
- 같은 입력의 Native SVG 반복 출력이 바이트 동일했다. Visual Sweep은 Native/WASM 각 2쪽,
  compare·standalone overlay·review **12 PNG**를 새로 생성했다.
- clipping 원장의 외부 controlset에는 이 샘플이 없어 해당 게이트의 통과를 주장하지 않는다.
- 전체 10,013개 및 Native Skia 통과는 제품 코드가 같은 이전 `086078148` 회차의 기록이며,
  이번에는 변경 범위인 fixture/test helper의 관련 검사와 lint를 실행했다. 전체 회귀를 재실행했다고 쓰지 않는다.
- 최종 head의 원격 CI는 push/PR 이후 확인할 항목이다. 이 회차에서 원격 push·comment·merge는 하지 않았다.

**판정: #7242 검토 승인.** 제출된 공개 샘플 자체를 정상화했고, 같은 경로를 직접 읽는
PDF 좌표 검사가 표 실제 외곽·후속 글줄을 입증한다. 수동 계약 입력도 줄 시작점이 맞도록
보정했다. 이것이 이전의 단순 대조군 추가와 다른 보류 해제 근거다. 임의의 잘못된 입력의
자동 복구, 전체 화소 일치, 비공개 문서 검증 또는 통합 PR 전체 승인을 뜻하지 않는다.
원격 최종 CI·다른 통합 PR의 판정은 해당 절차에서 별도로 확인한다.

## 접수·적용 범위

- 원 PR: [#7242](https://github.com/edwardkim/rhwp/pull/7242), LJYeon12. 기존 #7107·#7109·#7165·#7168·#7200 기여 이력이 있어 첫 기여자 절차는 비해당이다.
- source head `03ba57cb804610e899a3e6e1ce2cb3cfcb21df6e`, devel 대상 non-draft, reviewer `jangster77` 지정.
- `codex/pr7239-7240-review-20260917`, 기준 `236a601da803b53429e9090eef652c661dd3bfe2`.
- source 7개 커밋을 `cherry-pick -x`로 적용. `bf8070c27` → `fc668664a` → `0e3513b66` → `bb45be16b` → `1e3aed178` → `403c4ef7f` → `baa29e96e`. 텍스트 충돌 없음.
- 최종 source 범위: `src/renderer/layout.rs`, `tests/cases/stored_table_text_tail.rs`, `samples/stored-table-text-tail/README.md`, `native-8-0.hwpx`.
- 원 작성자 CI/사적 문서 검증은 제출자의 증거다. 이 통합 head의 검증과 구분한다. 비공개 원문·PDF를 공개하거나 확보했다고 간주하지 않는다.

## 구현·증거 대조

`layout_column_content`의 PageItem 순회 → 표 배치의 실제 `y_offset` → `PartialParagraph`의 저장 host 줄 끝과 다음 줄 시작의 gap → 실제 문단 layout → text/table/shape 점유 합집합 → 마지막 소유 item에서 문단 뒤 간격을 적용한다.

- 저장 줄이 같은 줄이면 원 앵커를 유지한다. 표 소속 줄 바로 다음 저장 줄만 흐름 끝과 gap을 사용한다.
- 문단 뒤 간격은 visible text tail의 반환값에서 제거해 deferred map으로 넘기고 마지막 item에서 한 번 적용한다.
- 합성 입력은 수동 저장 메타데이터임을 명시한다. 계약 검사는 2/8 셀 문단, 0/600 HU gap, HWP5-origin/pure 계보, 0/600/1200 HU 뒤 간격과 shape 공존을 검사한다.
- 기존 실제 입력 [156513948.hwpx](../../../samples/issue6044/156513948.hwpx)과 [한컴 PDF](../../../pdf/pr6940-156513948-source-2020.pdf)의 영향 20쪽을 Native/fresh WASM compare·overlay·review로 직접 판독한다.
- 제출 합성 입력은 별도 한컴 2020 변환: job `13f12b15-2c8a-403c-85cd-7df24135f0bd`, start→status succeeded→download, engine `2020`, Hancom `11.0.0.9136`, 1쪽, input_preprocess none. [기준 PDF](../../../pdf/pr7242/native-8-0-2020.pdf) SHA256 `90d6e84f4c24a5d0e79cde09cfd5d9addf1d8e92da322a13cb03f0dab9a2c3c3`.

## 초기 검증 기록

아래는 재빌드 전의 기록이며 최종 결과는 후단에 기록했다. 초기 Native focused 1 pass/2 fail은 오래된 build 산출물 사용 가능성을 발견해 판정을 유보했다. 소스 복사 시 과거 mtime이 보존되어 Cargo가 이미 빌드한 라이브러리로 판단한 경로를 확인했다. 소스 mtime 갱신 후 재컴파일과 새 binary 식별로 다시 검증하며 초기 결과를 최종 코드의 회귀 증거로 세지 않는다.

## Merge 후 contributor PR comment 계획

최상단 후속 renderer 보정 표의 `border_` compare·overlay·review 54개를 최종 증거로 사용한다. `fixed_` 12개는 입력 정상화 단계의 이전 증거다. 이전 실패 PNG는 보정 전 설명에만 연결한다. 통합 PR의 최종 head CI와 merge가 끝나면 실제 merge SHA·CI URL, 원 기여와 메인터너 보정, 검증 범위·남은 차이를 한국어로 설명하고 감사한다. [Visual Sweep 정본](../../../mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결한다. 최종 Native/fresh WASM compare·standalone overlay·review PNG를 merge SHA raw URL로 본문에 표시한다. 비공개 검증 자료를 공개 자료로 바꾸어 쓰지 않는다. UTF-8 본문 파일과 `--body-file`로 게시하고 원문과 이미지 URL을 다시 확인한다. 관련 공개 issue는 없어 임의 종료하지 않는다.

## 최종 통합 후보와 시각 증거

- code head: `75a48488676d79a0357ba1cae6c863ac2120b668`, base `236a601da803b53429e9090eef652c661dd3bfe2`.
- Native SHA256: `46d87aedbeca44eb31a31ddccd2c6b7e8deebd4008bc2bbe98598af67bdc8ca9`.
- fresh WASM SHA256: `6488efc93f6635ef0fe193e09d7982cfd48231d99679007cd8d3116f61c2dbc5`, JS `a7353a7603b7e07db2d33ff93fff6b213ea79e01da91c190cbb607e752c6b5a7`.
- Mac arm64/Rust 1.93.1, 별도 verify checkout의 source/test를 위 head와 바이트 대조했다. review target은 `target/pr7239-7240-review-20260917`. Docker 표준 경로 대신 host `scripts/wasm-pack-locked.sh --target web --out-dir <scratch>/wasm-final --no-opt`를 실행했다. wasm-opt 통과로 주장하지 않는다.
- `venv/bin/python scripts/visual_sweep.py --file-target <key> <입력> <PDF> --rhwp-bin <scratch>/rhwp-current --pages <아래 쪽> --dpi 96 --out <scratch>/native-sweep`, WASM은 `--wasm-pkg <scratch>/wasm-final` 추가. 최종 head로 재캡처한 compare·standalone overlay·review를 직접 판독했다.
- 전체 nextest·Native Skia 3종은 이번 후보에서 미실행이다. #7242 시각 보류 사유가 있어 지침의 작은 경계/영향 페이지 확인을 먼저 완료했고, 대규모 회귀를 통과 근거로 대신하지 않는다. 최종 승인·PR 제출 준비 완료가 아니다.

## 최종 판정 근거 — 시각 증거 미충족

**머지 보류.** 새 Native 빌드에서 focused 3개는 통과했고 초기 두 실패는 stale library 재사용이었다. 그러나 계약 검사만으로 시각 승인을 하지 않는다.

- 합성 입력의 표 bbox는 Native/WASM 모두 `(48.0,139.4,384.0,170.7)`. Footer는 base `(399.0,232.7)`에서 수정 후 `(399.0,310.0)`으로 이동해 표 내부 겹침은 해소했다. 한컴 PDF의 Footer text bbox는 약 `(115.04,343.47)`px로, 표 높이·Footer 수평/수직 위치가 명백히 다르다. 이 입력은 수동 저장 정보가 있는 합성이며 그 사실만으로 불일치를 승인하지 않는다.
- 실물 `156513948.hwpx`는 32쪽 중 31쪽 SVG가 base와 동일, 20쪽만 변했다. 주석 3줄이 +2.0267px 이동하고 나머지 text/표 paint는 유지된다. Native/fresh WASM/PDF를 직접 비교했으며 글꼴과 기존 표 테두리·아래 주석 오차가 남는다. 작성자가 보고한 상대 간격 개선을 원 비공개 사례의 PDF 정합성 검증으로 대신하지 않는다.
- `fidelity_compare.py 0 31 --source samples/issue6044/156513948.hwpx --reference-pdf pdf/pr6940-156513948-source-2020.pdf --text-only --export-all-svg --layout-ledger`로 전수 후보도 수집했다. 자동 지표는 승인 근거가 아니다. 불필요한 JSON/TSV/log는 커밋하지 않는다.
- 해제 조건: 합성 저장 정보의 유효성을 독립 기준과 확인하고, 정상 한컴 생성/재저장 대조군 또는 공개 가능한 원 실패 입력에서 같은 경로의 표 하단·후속 글줄 간격·문단 종료를 직접 입증해야 한다. 큰 차이를 임의 좌표 보정/출력 숨김으로 제거하지 않는다.
- 원 head upstream CI [35218364377](https://github.com/edwardkim/rhwp/actions/runs/35218364377)는 성공(31 success/3 skipped)이지만 이 통합 후보의 미충족 시각 증거를 대체하지 않는다.

### 직접 판독한 PNG

| 입력·쪽 | Native | fresh WASM |
| --- | --- | --- |
| tail p1 | [compare](../assets/pr7242_review/native_tail_compare_001.png) · [overlay](../assets/pr7242_review/native_tail_overlay_001.png) · [review](../assets/pr7242_review/native_tail_review_001.png) | [compare](../assets/pr7242_review/wasm_tail_compare_001.png) · [overlay](../assets/pr7242_review/wasm_tail_overlay_001.png) · [review](../assets/pr7242_review/wasm_tail_review_001.png) |
| real_tail p20 | [compare](../assets/pr7242_review/native_real_tail_compare_020.png) · [overlay](../assets/pr7242_review/native_real_tail_overlay_020.png) · [review](../assets/pr7242_review/native_real_tail_review_020.png) | [compare](../assets/pr7242_review/wasm_real_tail_compare_020.png) · [overlay](../assets/pr7242_review/wasm_real_tail_overlay_020.png) · [review](../assets/pr7242_review/wasm_real_tail_review_020.png) |

Merge 후 코멘트에는 위 **모든 영향 페이지**의 compare·overlay·review 링크를 실제 merge SHA의 raw URL로 치환한다. 대표 review만 넣고 standalone overlay를 빠뜨리지 않는다. 지금은 remote push/comment/merge를 수행하지 않았다.

### 최종 head 공통 검증 결과

- fmt check, Native Clippy, WASM32 lib Clippy, workspace build, workspace all-target Clippy(`-D warnings`), suite manifest base 비교: **모두 통과**.
- 최종 head focused 재실행: WMF fuzz **2**, golden **1**, column core **7**, column CLI **4**, 기존 page-break **11**, stored tail **3**, scaffold height **2** — **30 passed / 0 failed**. 각 원본에 `node scripts/run-rust-test.mjs <module>`를 test profile로 실행했다.
- Native/fresh WASM **각 9쪽** compare·standalone overlay·review, 총 18쪽 직접 확인. PNG 54개와 #7242 base 대조 PNG 6개를 개별 PR asset 경로에 보존했다. 총 60개이며 원 입력과 기준 PDF를 연결했다.
- test source·수치 baseline·허용치를 통과 목적으로 완화하지 않았다. source-side unit test 변경은 없어 unit-tier 비교는 비해당이다.
- 문서별 metadata 및 로컬 링크 검사, `git diff --check` 통과. 불필요한 log/JSON/TSV는 Git에 추가하지 않는다.

- 새 sample 3개(`stored-table-text-tail/native-8-0`, `issue7234/short_table_cell_row_height`, `issue7234/tall_table_cell_row_height`)의 hidden-text/injection/unicode 검사: **1 passed**. `RHWP_SECURITY_SWEEP_SAMPLES_JSON`으로 실제 대상을 지정해 실행했다.

## 메인터너 보정 회차 — 한컴 정상 저장 대조군과 원본 가정 검증

분석: 원 합성 입력을 다시 PDF로 변환한 job `ebf51a0c-107f-41e6-97d1-5741cb05baf4`는
engine 2020/Hancom 11.0.0.9136에서 성공했다. 새 PDF와 기존 PDF의 96dpi raster SHA256이
`959db583a1de4d482d105d0fe30573190a2eb92b69711f3688c172a9d7a05875`로 동일하므로 기존 PDF를 재사용한다.
원본 `native-8-0.hwpx`의 텍스트는 표 앞 15개, 뒤 70개 공백과 Footer이며 수동 저장한 3줄에는
실제 줄바꿈 한 줄이 빠졌다. 한컴 재저장에서는 textpos 0/15/25/83의 4줄이 된다.
셀 내부의 실제 8줄 메트릭도 추가되며 표 높이는 6000→12872 HU로 바뀐다.

수정 범위: 원본과 기존 실패 PNG는 유지한다. 단순 이름 변경 복제가 아닌 한컴이 실제 다시
계산·저장한 `samples/stored-table-text-tail/hancom-resaved.hwpx`를 독립 대조군으로 추가한다.
원본→HWP job `d4852d00-9b82-471d-9fa7-e2c24731272b`, HWP→HWPX job
`0047a7ab-79f3-45d2-a160-fdcd3a6a165b`, 모두 engine 2020, preprocess none, 성공 상태를 확인했다.
한컴 PDF의 Footer 기준선 265.866821pt, x=86.28pt와 실제 표 외곽 y=140.77..312.42px를
정식 회귀 검사의 독립 기대값으로 추가한다. 기존 보고의 큰 외곽 상자는 표 테두리와 문단
테두리를 혼동했으므로 구분한다. 합성 원본의 잘못된 저장 줄을 엔진이 복원했다고 주장하지 않는다.

결과: 정상 저장 대조군도 PDF로 재변환(job `6a75daf6-716f-40b3-9b87-96fac5c121ae`, 성공)해
기존 PDF와 raster가 동일함을 확인했다. 중복 PDF는 추가하지 않았다. Native/fresh WASM의
표 실제 외곽과 Footer 기준선은 PDF와 0.5px 이내다. 기존 코드의 Footer 기준선 367.92px에서
#7242 적용 후 354.2133px로 바뀌며 한컴 PDF 354.4891px에 맞는다. 페이지 수 1쪽, 셀 텍스트
8개와 공백 뒤 Footer의 네 번째 글줄을 보존했다. 문자/문단 테두리의 일부 기존 차이는 남는다.

| 정상 저장 대조군 1쪽 | compare | overlay | review |
| --- | --- | --- | --- |
| Native | [비교](../assets/pr7242_review/native_hancom_compare_001.png) | [겹침](../assets/pr7242_review/native_hancom_overlay_001.png) | [직접 판독](../assets/pr7242_review/native_hancom_review_001.png) |
| fresh WASM | [비교](../assets/pr7242_review/wasm_hancom_compare_001.png) | [겹침](../assets/pr7242_review/wasm_hancom_overlay_001.png) | [직접 판독](../assets/pr7242_review/wasm_hancom_review_001.png) |

실행 binary/package는 위 source head의 제품 코드와 동일하다(이번 회차는 fixture·검사·증적 추가).
대조군에서 원 PR의 표 뒤 흐름 보정은 확인했으나 **원 합성 파일 자체의 자동 재조판 불일치는
아직 해소하지 않았다**. 정상 대조군 통과를 원본의 해결로 바꾸어 판정하지 않는다.

실제 실행: 새 PDF 기준선 검사만 `src/renderer/layout.rs`를 base `236a601da`로 되돌린
별도 verify checkout에서 실행해 **1 failed**(Footer baseline 367.92)를 확인했다. 제품 소스를
복구한 뒤 원 모듈 전체 **4 passed**. 임계값·baseline 완화는 없다. Native/fresh WASM 대조군
compare·overlay·review를 새로 산출하고 직접 판독했다. 원본 합성 입력 불일치가 남으므로
이 회차는 정상 저장 경로의 증거 보완이며 전체 보류 해제로 표시하지 않는다.
