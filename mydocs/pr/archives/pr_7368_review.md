# PR #7368 검토 — 표 조각 컷과 저장 RowBreak 표

## 최종 판정

**머지 보류 사유 해결 — 메인터너 보정 로컬 검토 승인.** 원 PR head `2386ecfa24b19ff880cc2af39ae7dcc0fad57f52`를 최신 `upstream/devel` (`505661360e9a2d596f55300d0cb0c5222f0e14b4`)에 체리픽한 `review/seongeun82-7368-20260924`에서 배치·쪽번호를 추가 보정했다. 한컴 2020 PDF 기준 Native/fresh WASM Visual Sweep은 각각 HWP **6/6쪽**, HWPX **7/7쪽** 모두 2px 관용 실루엣 90% 문턱을 넘었다. Native 최저는 HWP 5쪽 92.84%, HWPX 1쪽 91.93%이며 WASM 최저는 각각 92.84%, 91.96%다. 원 PR의 4~6쪽 및 2~6쪽 배치 불일치는 더 이상 남지 않는다.

HWP 5쪽 상단 표에서 첫 글자 잉크 시작은 PDF y=142px, rhwp y=140px이며 마지막 줄까지 1~2px 차이가 남는다. 이전 증적의 큰 문단 여백 차이와 달리 최신 조각 높이·중앙 정렬 보정 후에는 누적 공백이나 쪽 소속 차이가 없다. 자동 쪽번호는 문서의 `쪽 번호` 스타일을 사용하고, 글꼴별 세로 기준을 보정했다. 2~6쪽에서 한컴 PDF와 rhwp의 쪽번호 잉크 경계가 동일한 x=381..411, y=1052..1061px이며 1쪽만 세로 1px 차이다. 이 차이를 숨기거나 100% 시각 일치로 주장하지 않는다.

`cargo-nextest 0.9.145`로 전체 Rust 회귀 **10,204/10,204 통과**, Clippy 3종, Native Skia 3종, fresh WASM 대체 빌드·Visual Sweep을 완료했다. 현재 Mac 로컬 환경 지침에 따라 Docker를 사용하지 않았으며, WASM은 아래에 명시한 저장소 스크립트의 `--no-opt` 빌드 결과다. 원 기여 commit 위에 최신 `upstream/devel`을 병합하고 보정 commit을 분리한 source 후보 `80d4f9c8d0d69b26945802d8ca101c65809e25dd`를 PR #7368에 push했다. 이 코드 head의 [GitHub CI](https://github.com/edwardkim/rhwp/actions/runs/35985929282)는 Build & Test·Lint·Native Skia를 포함해 완료됐고 CodeQL·Render Diff 등 별도 check에도 실패·대기가 없다. 문서 전용 trailing head CI·comment·merge는 아직 수행하지 않았다.

## 접수와 보정 범위

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#7368](https://github.com/edwardkim/rhwp/pull/7368), `seongeun82`, `fix/table-fragment-cut-v2`, Open / non-Draft |
| 관련 이슈 | [#7336](https://github.com/edwardkim/rhwp/issues/7336), 원 PR 본문 `Closes #7336` |
| 원 base / head | `7a95e46e025470a4d7a7b59ad68ec02958bda738` / `2386ecfa24b19ff880cc2af39ae7dcc0fad57f52` |
| 검토 base / 체리픽 | `505661360e9a2d596f55300d0cb0c5222f0e14b4` / `2ae659cae7b5b17cf7b76874c446ff094b704881` |
| source 후보 | 원 기여 head `2386ecfa24b19ff880cc2af39ae7dcc0fad57f52` 보존, 최신 `upstream/devel` `8619e6d4f5e10f4ab6478798bdf3e5660a8ba100` 병합 뒤 보정 4 commit, 후보 head `80d4f9c8d0d69b26945802d8ca101c65809e25dd` |
| 기여자 | 이전 merged PR 없음. 첫 기여자 절차에 따라 감사와 구체적인 보정 내역·남은 차이를 한국어로 설명한다. |

원 PR은 조각이 소비한 중첩 표를 그리는 행의 높이에 반영하고, 한 쪽보다 큰 저장 RowBreak 표를 통째 배치하지 않도록 한다. 메인터너 보정은 두 샘플의 저장 문단 쪽 리셋, 표 조각의 바깥 여백·rowspan 셀 높이와 중앙 정렬, HWPX 표 뒤 줄간격의 이중 계상, 쪽번호 스타일·세로 위치를 다룬다. 전체 회귀에서 문단 기준 HWPX 거대 셀과 HWP5 장문 표가 밀리는 것을 확인해 바깥 여백 재적용을 각각 **단 기준 자리차지 표**, **반복 제목행 뒤에서 표 끝까지 걸친 가운데 정렬 rowspan 셀을 가진 표**로 좁혔다. #1937의 마지막 각주와 쪽번호 겹침 1건 증가는 표 크기와 무관한 별도 원인이므로 각주 있는 쪽에는 새 쪽번호 세로 이동을 적용하지 않는다. 문서 파일명·표 행열 수 조건은 사용하지 않는다. `body_overflow_baseline.tsv`의 새 샘플 2행은 원 PR 변경이며 기존 샘플 값은 건드리지 않았다.

## 기준 자료와 검증

두 입력은 각각 한컴오피스 **2022**, **2018** 저장본으로 `rhwp info --json`에서 확인했다. 저장 버전 기준 엔진인 `hwp2024-mcp-convert --engine 2020` 출력물을 주 기준으로 사용했다. 2024 엔진 출력물은 교차 확인용이다. HWPX MCP 변환은 서버가 `input_preprocess=hwpx_form_controls_flattened`를 보고했으며 원본 HWPX 파일 해시는 바뀌지 않았다.

| 입력 | SHA-256 | 기준 PDF (SHA-256) | 쪽수 |
| --- | --- | --- | ---: |
| `samples/issue7336/nested_table_fragment_cut.hwp` | `53f4abb0f76f6366931e0c1b3610c58ca44ad3120869a4ea8e86a0ab7ff42543` | [`nested_table_fragment_cut-2020.pdf`](../../../pdf/issue7336/nested_table_fragment_cut-2020.pdf) (`bd788c647adfd9a8311a1c8bc6cf48c154a7d8fadaa802c436279dc928bf790d`) | 6 |
| `samples/issue7336/stored_frame_page_larger_rowbreak.hwpx` | `042633046ab469b971aa155417ecd568c8825d586d37f87a76194145edb903dc` | [`stored_frame_page_larger_rowbreak-2020.pdf`](../../../pdf/issue7336/stored_frame_page_larger_rowbreak-2020.pdf) (`5e19bd4c4d0cc3e7b177834e939a8e75df57e4387d43d1a9a1bd8f74840b993e`) | 7 |

- `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --no-fail-fast --test-threads 8`: Mac `cargo-nextest 0.9.145`, **10,204 통과·0 실패·50 skipped**, 종료 코드 0. 초기 전체 실행의 20개 실패는 위 여백 게이트 축소와 쪽번호 보정, 의도된 SVG golden·글꼴 해시 2건 갱신 후 재실행해 해소했다.
- 코드 후보 head `80d4f9c8d0d69b26945802d8ca101c65809e25dd`의 [CI run](https://github.com/edwardkim/rhwp/actions/runs/35985929282)에서 Build & Test aggregate·Lint·Native Skia·Frontend package gates가 성공했다. 별도 CodeQL·Render Diff·Proptest·Adapter check도 성공했고 최종 CI Impact Policy는 `SUCCESS`다. `WASM Build`는 정책상 skipped였으므로 아래 로컬 WASM 결과로만 기록한다.
- #7336 focused **11/11 통과**; 표 행열 수 제한을 제거한 최종 교차 검증에서 #7336/#1937·거대 셀·선택 범위·장문 표 사례 **35/35 통과**. `form-002` SVG golden은 쪽번호 텍스트 3개의 글꼴·좌표만 바뀌었고 한컴 PDF와 대조했으며, 변경된 글꼴 결정 추적 해시 2건도 실제 Native 출력으로 재계산했다. 두 관련 회귀 **2/2 통과**.
- `cargo fmt --all -- --check`, native·WASM32·workspace all-targets Clippy (`-D warnings`), `cargo build --locked --workspace`: 모두 통과.
- `cargo test --locked --profile release-test --target-dir target/pr-review --features native-skia --lib`: 주 라이브러리 **3,930 통과·13 ignored** 및 나머지 라이브러리 전부 통과. Native Skia `issue_2225_missing_picture_placeholder` **2/2**, `render_p37_direct_pdf_export` **4/4** 통과.
- 현재 Mac 로컬 환경 지침에 따라 Docker 대신 저장소 루트에서 `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt` 성공. `pkg/`와 `rhwp-studio/public/`의 glue·WASM SHA-256은 각각 일치하며 추적 파일 `rhwp-studio/public/rhwp.js`는 바뀌지 않았다. Docker 최적화 빌드 결과로 세지 않는다.
- 최종 Native sweep HWP `output/pr7368-native-final-hwp`: 6/6 완료, `pr_review_gate.status=passed`, 각 쪽 93.98, 99.93, 98.58, 99.88, 92.84, 99.60%. HWPX `output/pr7368-native-final-hwpx`: 7/7 완료, `pr_review_gate.status=passed`, 각 쪽 91.93, 95.68, 99.75, 99.89, 97.27, 98.52, 97.74%. 기존 증적과 비교 본문은 동일했고 최종 head `9b720b1dae9c34c76fd95dff883d97c040934ee9`에서 새 PNG로 교체했다.
- 같은 head의 새 `pkg/`에서 실행한 fresh WASM sweep HWP `output/pr7368-wasm-hwp`: 6/6 완료, `passed`, 각 쪽 93.98, 99.93, 98.58, 99.88, 92.84, 99.60%. HWPX `output/pr7368-wasm-hwpx`: 7/7 완료, `passed`, 각 쪽 91.96, 95.68, 99.75, 99.89, 97.27, 98.52, 97.74%. 아래 review·overlay PNG와 각 경로의 contact sheet를 보존했다.

## 페이지별 시각 증적

모든 페이지의 Native/fresh WASM review·overlay PNG는 [`pr7368_review`](../assets/pr7368_review/)에 보존했다. 특히 분할 전후인 HWP 4·5·6쪽과 HWPX 2·4·5·6쪽을 검토했다. [Native HWP](../assets/pr7368_review/hwp_review_contact_sheet.png)·[Native HWPX](../assets/pr7368_review/hwpx_review_contact_sheet.png)·[WASM HWP](../assets/pr7368_review/wasm_hwp_review_contact_sheet.png)·[WASM HWPX](../assets/pr7368_review/wasm_hwpx_review_contact_sheet.png) 전체 contact sheet도 제공한다. 최종 summary는 로컬 `output/pr7368-native-final-hwp/summary.json`, `output/pr7368-native-final-hwpx/summary.json`, `output/pr7368-wasm-hwp/summary.json`, `output/pr7368-wasm-hwpx/summary.json`이다. summary JSON·로그는 임시 산출물이라 커밋하지 않는다.

| 페이지 | 비교 이미지 | overlay |
| --- | --- | --- |
| HWP 1 | [review](../assets/pr7368_review/hwp_review_001.png) | [overlay](../assets/pr7368_review/hwp_overlay_001.png) |
| HWP 2 | [review](../assets/pr7368_review/hwp_review_002.png) | [overlay](../assets/pr7368_review/hwp_overlay_002.png) |
| HWP 3 | [review](../assets/pr7368_review/hwp_review_003.png) | [overlay](../assets/pr7368_review/hwp_overlay_003.png) |
| HWP 4 | [review](../assets/pr7368_review/hwp_review_004.png) | [overlay](../assets/pr7368_review/hwp_overlay_004.png) |
| HWP 5 | [review](../assets/pr7368_review/hwp_review_005.png) | [overlay](../assets/pr7368_review/hwp_overlay_005.png) |
| HWP 6 | [review](../assets/pr7368_review/hwp_review_006.png) | [overlay](../assets/pr7368_review/hwp_overlay_006.png) |
| HWPX 1 | [review](../assets/pr7368_review/hwpx_review_001.png) | [overlay](../assets/pr7368_review/hwpx_overlay_001.png) |
| HWPX 2 | [review](../assets/pr7368_review/hwpx_review_002.png) | [overlay](../assets/pr7368_review/hwpx_overlay_002.png) |
| HWPX 3 | [review](../assets/pr7368_review/hwpx_review_003.png) | [overlay](../assets/pr7368_review/hwpx_overlay_003.png) |
| HWPX 4 | [review](../assets/pr7368_review/hwpx_review_004.png) | [overlay](../assets/pr7368_review/hwpx_overlay_004.png) |
| HWPX 5 | [review](../assets/pr7368_review/hwpx_review_005.png) | [overlay](../assets/pr7368_review/hwpx_overlay_005.png) |
| HWPX 6 | [review](../assets/pr7368_review/hwpx_review_006.png) | [overlay](../assets/pr7368_review/hwpx_overlay_006.png) |
| HWPX 7 | [review](../assets/pr7368_review/hwpx_review_007.png) | [overlay](../assets/pr7368_review/hwpx_overlay_007.png) |

fresh WASM 출력도 동일 페이지의 한컴 PDF와 별도로 비교했다.

| 페이지 | WASM 비교 이미지 | WASM overlay |
| --- | --- | --- |
| HWP 1 | [review](../assets/pr7368_review/wasm_hwp_review_001.png) | [overlay](../assets/pr7368_review/wasm_hwp_overlay_001.png) |
| HWP 2 | [review](../assets/pr7368_review/wasm_hwp_review_002.png) | [overlay](../assets/pr7368_review/wasm_hwp_overlay_002.png) |
| HWP 3 | [review](../assets/pr7368_review/wasm_hwp_review_003.png) | [overlay](../assets/pr7368_review/wasm_hwp_overlay_003.png) |
| HWP 4 | [review](../assets/pr7368_review/wasm_hwp_review_004.png) | [overlay](../assets/pr7368_review/wasm_hwp_overlay_004.png) |
| HWP 5 | [review](../assets/pr7368_review/wasm_hwp_review_005.png) | [overlay](../assets/pr7368_review/wasm_hwp_overlay_005.png) |
| HWP 6 | [review](../assets/pr7368_review/wasm_hwp_review_006.png) | [overlay](../assets/pr7368_review/wasm_hwp_overlay_006.png) |
| HWPX 1 | [review](../assets/pr7368_review/wasm_hwpx_review_001.png) | [overlay](../assets/pr7368_review/wasm_hwpx_overlay_001.png) |
| HWPX 2 | [review](../assets/pr7368_review/wasm_hwpx_review_002.png) | [overlay](../assets/pr7368_review/wasm_hwpx_overlay_002.png) |
| HWPX 3 | [review](../assets/pr7368_review/wasm_hwpx_review_003.png) | [overlay](../assets/pr7368_review/wasm_hwpx_overlay_003.png) |
| HWPX 4 | [review](../assets/pr7368_review/wasm_hwpx_review_004.png) | [overlay](../assets/pr7368_review/wasm_hwpx_overlay_004.png) |
| HWPX 5 | [review](../assets/pr7368_review/wasm_hwpx_review_005.png) | [overlay](../assets/pr7368_review/wasm_hwpx_overlay_005.png) |
| HWPX 6 | [review](../assets/pr7368_review/wasm_hwpx_review_006.png) | [overlay](../assets/pr7368_review/wasm_hwpx_overlay_006.png) |
| HWPX 7 | [review](../assets/pr7368_review/wasm_hwpx_review_007.png) | [overlay](../assets/pr7368_review/wasm_hwpx_overlay_007.png) |

## PR 본문 직접 증적 계획

trailing 문서 commit을 push한 뒤 PR 본문에 실제 최종 head SHA와 위 두 기준 PDF·각 페이지 Native/WASM review·overlay 링크를 넣는다. 대표 HWP 5쪽과 HWPX 2쪽은 Native 및 WASM review·overlay를 `![HWP 5쪽 Native 비교](https://raw.githubusercontent.com/seongeun82/rhwp/<최종-head-sha>/mydocs/pr/assets/pr7368_review/hwp_review_005.png)`처럼 fork head repository의 Markdown 이미지로 직접 표시한다. 나머지 페이지는 위 표와 같은 직접 링크로 제공한다. URL이 실제 head에서 열리고 이미지가 보이는지 게시 뒤 확인한다.

## Merge 후 contributor PR comment 계획

CI 통과와 merge 뒤에는 첫 기여자에게 한국어 **존댓말**로 감사하고 rhwp 첫 기여를 환영합니다. 댓글은 원 기여 구현과 메인터너 보정을 먼저 구분한 다음, **왜 보정이 필요했는지**를 아래 순서로 구체적으로 설명합니다. 첫 기여자의 책임으로 단정하거나 단순히 “시각 차이 수정”이라고 축약하지 않습니다.

1. 원 기여 head `2386ecfa`에서 조각이 소비한 중첩 표 높이 누락과 저장 RowBreak 표의 과도한 통째 배치를 고쳐 주셨습니다. 표 내용 소실과 4쪽으로 줄어든 HWPX의 핵심 원인을 해결해 주신 기여임을 먼저 말씀드립니다.
2. 그 후 두 실물 파일의 **모든 쪽을 한컴 2020 PDF와 Native/fresh WASM review·overlay로 비교**하자, 페이지 수·텍스트·작은 기하 테스트가 잡지 못한 HWP 5쪽 상단 표 뒤 문단 간격, HWPX 표 분할 경계와 후행 문단 위치, 여러 쪽의 자동 쪽번호 글꼴·세로 위치 차이가 보였습니다. 원 PR의 두 원인 수정만으로 PDF 배치 일치를 확인할 수 없어 추가 보정이 필요했습니다.
3. 보정 commit `08b4ec81d`에서 저장 문단의 쪽 리셋, 조각 바깥 여백과 rowspan 셀 높이·중앙 정렬, HWPX 표 뒤 줄간격 이중 계상, 쪽번호 스타일과 글꼴별 세로 기준을 바로잡았습니다. 전체 회귀에서 바깥 여백 적용이 너무 넓어 거대 셀·장문 표가 밀리고 각주 있는 쪽에서 쪽번호가 겹치는 사례가 확인되어 `ea6f3c87b`에서 의미 기반 적용 범위와 각주 예외를 조정했습니다. `255b9a8dc`에서는 3×2 표 크기에 맞춘 임의 조건을 제거하고 반복 제목행 뒤에서 끝까지 걸친 가운데 정렬 rowspan이라는 실제 조판 조건만 남겼습니다. 메인터너 보정은 기여 구현을 대체한 것이 아니라 PDF 비교와 전체 회귀에서 발견한 별도 문제를 해결한 것입니다.
4. 최종 Native/WASM HWP 6쪽·HWPX 7쪽의 2px 관용 실루엣 gate는 모두 통과했지만 HWP 5쪽 글자 잉크 1~2px 차이는 남아 있습니다. 로컬 전체 회귀 10,204개와 **실제 최종 head CI URL·결과**, merge SHA, 이슈 #7336 상태를 과장 없이 적겠습니다.

게시할 때는 위 설명을 자연스러운 한국어 존댓말 문단으로 작성하고 [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결합니다. Native HWP 5쪽과 WASM HWPX 2쪽의 **review와 standalone overlay 네 장**을 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7368_review/` 아래의 Markdown 이미지로 직접 표시합니다. 이슈 #7336에도 존댓말로 실제 merge 결과·핵심 보정 이유·대표 review/overlay 이미지를 남깁니다. PR 본문의 `Closes #7336`에 따라 이슈가 실제로 닫혔는지 확인합니다. 댓글은 UTF-8 본문 파일과 `--body-file`로 게시하고 한국어·이미지 URL·실제 merge SHA를 다시 확인합니다. 최종 head 검증 전에는 승인·merge를 주장하지 않습니다.
