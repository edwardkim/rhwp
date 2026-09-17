---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-18
---

# PR #7243 검토

**현재 판정: 정상 원본 교체·회귀 검사 보정 및 기존 282 HU 입력의 91.08px 넘침 복구, 최종 공통 검증 통과.** 새 입력만으로 종전 결함을 숨기지 않고 기존·새 입력을 함께 검증했다. 정상 86712의 남은 PDF 위치·글꼴 차이와 #7242의 원 합성 입력 보류는 별개로 남는다. 통합 PR 전체 승인 또는 이슈 종료로 보고하지 않는다.

아래 최초 검토·중간 후보 결과는 해당 시점의 기록이다. 최신 소스·검증 결과는 문서 끝 **최종 후보 26** 절이 우선한다. PNG 링크는 후보 26 캡처로 갱신했으며 앞 회차의 바이너리 해시·수치를 최신 PNG의 생성 정보로 사용하지 않는다.

## 접수·체리픽

- [원 PR #7243](https://github.com/edwardkim/rhwp/pull/7243), planet6897, devel 대상 non-draft. 기존 기여자, reviewer `jangster77` 지정.
- source `d465c0a47916b973e8be99225b88c1ee59205915` → 통합 `75a48488676d79a0357ba1cae6c863ac2120b668`, `cherry-pick -x`, 충돌 없음.
- 브랜치 `codex/pr7239-7240-review-20260917`, 기준 `236a601da803b53429e9090eef652c661dd3bfe2`. #7239·#7240·#7242를 함께 포함한다.

## 발견 사항

### P1 — 원 이슈의 기존 재현 파일은 계속 넘치므로 종료 범위가 과도함

`src/scaffold/builder.rs`는 새 셀의 선언 높이를 282→1282 HU로 바꾼다. 정상적인 최소 높이 생성 개선이지만 이미 저장된 `samples/issue7216/tall_table_after.hwpx`의 셀은 여전히 282 HU다. 원 이슈는 그 파일을 열 때 행 컷과 실제 렌더 높이가 달라지는 엔진 결함을 포함한다. PR 설명도 기존 overflow baseline과 엔진 불일치가 남는다고 명시한다.

한컴이 기존 282 HU 셀을 보존하며 정상 조판한다는 작성자 근거와도 일치한다. 새 fixture로 입력을 바꾼 통과를 원 이슈의 해소로 세지 않는다. 통합 PR에서는 `Refs #7234`로 범위를 한정하고 이슈를 열어 두어야 한다. 원 이슈 전체를 종료하려면 기존 입력의 컷·측정·실제 배치를 공통 계약으로 해결하고 원본·대조군을 검증해야 한다.

### P2 — 시각 검증 비해당 주장은 부적절함

renderer 파일이 바뀌지 않아도 생성한 선언 높이가 행 분할과 최종 배치를 바꾼다. 저장소 조판 지침에 따라 Native/fresh WASM Visual Sweep을 추가한다. 제출 PDF가 종전 PDF와 같다는 사실은 rhwp 출력의 일치를 대신하지 않는다.

## 독립 검증

- 새 테스트를 builder 변경 전 코드에 이식: **0 passed / 2 failed**. 셀 선언 높이 assertion과 실제 본문 바닥 넘침(칸 바닥 1100.0, 본문 바닥 1009.1)으로 실패했다.
- builder 변경 적용 후: **2 passed**. 첫 조각의 본문 내 수용과 1쪽 행 46/2쪽 행 47 계약을 확인했다. 테스트 허용치·baseline 변경 없음.
- 기존 Git 입력·PDF를 재사용: [긴 표](../../../samples/issue7234/tall_table_cell_row_height.hwpx), [긴 표 PDF](../../../samples/issue7234/tall_table_cell_row_height-2020.pdf), [짧은 표](../../../samples/issue7234/short_table_cell_row_height.hwpx), [짧은 표 PDF](../../../samples/issue7234/short_table_cell_row_height-2020.pdf).
- Native Visual Sweep 직접 판독: 긴 표 2쪽, 짧은 표 1쪽. 긴 표는 1쪽 마지막 46행·2쪽 첫 47행으로 PDF와 일치하고 본문 바닥 넘침이 해소됐다. 표·후속 문단의 소폭 위치와 글꼴·선 굵기 차이는 남아 전체 시각 일치로 보고하지 않는다.
- fresh WASM·최종 공통 lint 결과는 아래 최종 기록에 추가한다. 원 작성자 CI 9993 passed를 통합 head 결과로 재사용하지 않는다.

## Merge 후 contributor PR comment 계획

실제 통합 merge·최종 CI가 완료된 경우 기여 감사와 적용 SHA, 새 scaffold 산출물 개선과 메인터너의 기존 282 HU 입력 복구를 구분해 한국어로 설명한다. 최초 검토의 91.08px 넘침은 후속 보정으로 해소됐으므로 미해결이라고 반복하지 않는다. 정상 86712 교체·회귀 검사 정정과 실제 남은 시각 차이도 명시하고, 이슈 종료 여부는 최종 해결 범위를 대조해 결정한다. [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결하고 긴 표 1·2쪽과 짧은 표 1쪽의 Native/fresh WASM compare·overlay·review PNG를 merge SHA raw URL로 본문에 포함한다. UTF-8 파일과 `--body-file`로 게시 후 한국어·이미지 URL·실제 head를 검증한다. contributor fork branch를 삭제하지 않는다.

## 최종 통합 후보와 시각 증거

- code head: `75a48488676d79a0357ba1cae6c863ac2120b668`, base `236a601da803b53429e9090eef652c661dd3bfe2`.
- Native SHA256: `46d87aedbeca44eb31a31ddccd2c6b7e8deebd4008bc2bbe98598af67bdc8ca9`.
- fresh WASM SHA256: `6488efc93f6635ef0fe193e09d7982cfd48231d99679007cd8d3116f61c2dbc5`, JS `a7353a7603b7e07db2d33ff93fff6b213ea79e01da91c190cbb607e752c6b5a7`.
- Mac arm64/Rust 1.93.1, 별도 verify checkout의 source/test를 위 head와 바이트 대조했다. review target은 `target/pr7239-7240-review-20260917`. Docker 표준 경로 대신 host `scripts/wasm-pack-locked.sh --target web --out-dir <scratch>/wasm-final --no-opt`를 실행했다. wasm-opt 통과로 주장하지 않는다.
- `venv/bin/python scripts/visual_sweep.py --file-target <key> <입력> <PDF> --rhwp-bin <scratch>/rhwp-current --pages <아래 쪽> --dpi 96 --out <scratch>/native-sweep`, WASM은 `--wasm-pkg <scratch>/wasm-final` 추가. 최종 head로 재캡처한 compare·standalone overlay·review를 직접 판독했다.
- 전체 nextest·Native Skia 3종은 이번 후보에서 미실행이다. #7242 시각 보류 사유가 있어 지침의 작은 경계/영향 페이지 확인을 먼저 완료했고, 대규모 회귀를 통과 근거로 대신하지 않는다. 최종 승인·PR 제출 준비 완료가 아니다.

### 최종 후보 직접 확인

`layout-anomaly samples/issue7216/tall_table_after.hwpx --json`: 기존 입력 **overflowCount=1 / overBottom=91.08px**. 새 `samples/issue7234/tall_table_cell_row_height.hwpx`: **overflowCount=0**, 둘 다 2쪽이다. 따라서 `Closes #7234` 보류 사유는 코드 추정이 아니라 실제 실행 결과다. 새 입력의 Native/WASM raster는 3쪽 모두 서로 동일하고, 긴 표 46/47 행 분할과 짧은 표 1쪽을 PDF와 직접 확인했다. #7243 원 CI [35220748013](https://github.com/edwardkim/rhwp/actions/runs/35220748013)는 조회 당시 진행 중이며 그 결과를 로컬 검증으로 세지 않는다.

### 직접 판독한 PNG

| 입력·쪽 | Native | fresh WASM |
| --- | --- | --- |
| tall p1 | [compare](../assets/pr7243_review/native_tall_compare_001.png) · [overlay](../assets/pr7243_review/native_tall_overlay_001.png) · [review](../assets/pr7243_review/native_tall_review_001.png) | [compare](../assets/pr7243_review/wasm_tall_compare_001.png) · [overlay](../assets/pr7243_review/wasm_tall_overlay_001.png) · [review](../assets/pr7243_review/wasm_tall_review_001.png) |
| tall p2 | [compare](../assets/pr7243_review/native_tall_compare_002.png) · [overlay](../assets/pr7243_review/native_tall_overlay_002.png) · [review](../assets/pr7243_review/native_tall_review_002.png) | [compare](../assets/pr7243_review/wasm_tall_compare_002.png) · [overlay](../assets/pr7243_review/wasm_tall_overlay_002.png) · [review](../assets/pr7243_review/wasm_tall_review_002.png) |
| short p1 | [compare](../assets/pr7243_review/native_short_compare_001.png) · [overlay](../assets/pr7243_review/native_short_overlay_001.png) · [review](../assets/pr7243_review/native_short_review_001.png) | [compare](../assets/pr7243_review/wasm_short_compare_001.png) · [overlay](../assets/pr7243_review/wasm_short_overlay_001.png) · [review](../assets/pr7243_review/wasm_short_review_001.png) |

Merge 후 코멘트에는 위 **모든 영향 페이지**의 compare·overlay·review 링크를 실제 merge SHA의 raw URL로 치환한다. 대표 review만 넣고 standalone overlay를 빠뜨리지 않는다. 지금은 remote push/comment/merge를 수행하지 않았다.

### 최종 head 공통 검증 결과

- fmt check, Native Clippy, WASM32 lib Clippy, workspace build, workspace all-target Clippy(`-D warnings`), suite manifest base 비교: **모두 통과**.
- 최종 head focused 재실행: WMF fuzz **2**, golden **1**, column core **7**, column CLI **4**, 기존 page-break **11**, stored tail **3**, scaffold height **2** — **30 passed / 0 failed**. 각 원본에 `node scripts/run-rust-test.mjs <module>`를 test profile로 실행했다.
- Native/fresh WASM **각 9쪽** compare·standalone overlay·review, 총 18쪽 직접 확인. PNG 54개와 #7242 base 대조 PNG 6개를 개별 PR asset 경로에 보존했다. 총 60개이며 원 입력과 기준 PDF를 연결했다.
- test source·수치 baseline·허용치를 통과 목적으로 완화하지 않았다. source-side unit test 변경은 없어 unit-tier 비교는 비해당이다.
- 문서별 metadata 및 로컬 링크 검사, `git diff --check` 통과. 불필요한 log/JSON/TSV는 Git에 추가하지 않는다.

- 새 sample 3개(`stored-table-text-tail/native-8-0`, `issue7234/short_table_cell_row_height`, `issue7234/tall_table_cell_row_height`)의 hidden-text/injection/unicode 검사: **1 passed**. `RHWP_SECURITY_SWEEP_SAMPLES_JSON`으로 실제 대상을 지정해 실행했다.

## 메인터너 보정 회차 — 기존 입력의 내용 컷과 물리 높이

분석: 저장 셀 높이 282 HU가 여백 합과 같아 `row_cut_content_height`의 패딩 축소가
내용 컷을 15.2px로 계산한다. 실제 전체 행은 `MeasuredTable.row_heights` 17.0933px로
그린다. `prepare_block_table_continuation`의 `whole_row_fit_h` → `scan_block_table_split_rows`
→ 누적 consumed → `layout_partial_table`의 행 높이 소비까지 대조한다. 잘린 내용의
오프셋과 온전한 행의 물리 높이를 섞지 않고, 온전한 행 예약이 실제 높이를 쓰도록 수정한다.
기존 padding 임계값을 전역 변경하거나 셀의 저장 높이를 덮어쓰지 않는다. 기존 282 HU 파일과
새 1282 HU 파일, 짧은 표, 알려진 다쪽 반례를 검사하고 영향을 받은 쪽의 Native/fresh WASM
compare·overlay·review를 다시 생성한다. 후보가 회귀하면 같은 회차에서 원인을 재검토한다.

보정 구현: `row_cut_content_height`와 패딩 축소 규칙은 유지한다. 전체 행을 배치하는 경로의
`resolve_row_heights_trusting_declared(..., allow_declared_trust=false)` 결과를 예약에서도
소비한다. 실제로 그 결과를 사용하는 행인지는 `whole_fragment_row_uses_measured_height`를
배치/예약이 공유한다. 선언 높이가 패딩을 수용하지 못하는 셀의 전체 행에만 물리 높이를
추가 예약한다. 중첩 표의 별도 내용 컷, 시작/끝 컷과 기존 native rewind 경로를 구분한다.
온전한 행의 consumed 증가에는 해당 실제 높이가 들어가고 부분 행의 내용 오프셋은 바꾸지 않는다.

수정 전 검사는 기존 `tall_table_after.hwpx`의 실제 칸 바닥 1099.96px > 본문 1009.12px로
실패했다(**3 passed / 1 failed**). 보정 후 **4 passed**: 기존/새 저장본 모두 46/47행
분할, 본문 내 수용, 1..60행 각각 한 번, 마지막 쪽의 표 다음 문단을 확인했다. 작은 남은 공간
검사는 수정 전에도 통과하는 정상 대조이며 결함 검출 검사로 세지 않는다.

기존 입력의 `layout-anomaly`: overflow/overlap/offCanvas/textOverlap 각각 **0**, 2쪽.
Native/fresh WASM 기존 긴 표 1·2쪽과 짧은 표 1쪽, 새 긴 표 1·2쪽과 짧은 표 1쪽을 모두
새로 캡처해 직접 대조했다. 46/47행 경계·표 외곽·표 다음 문단이 보존된다. PDF 대비
표 시작과 후속 문단의 기존 수 px 위치·글꼴/선 굵기 차이는 남으며 전체 화소 일치 주장은 아니다.

| 기존 입력·쪽 | Native | fresh WASM |
| --- | --- | --- |
| tall_table_after p1 | [compare](../assets/pr7243_review/native_old_tall_compare_001.png) · [overlay](../assets/pr7243_review/native_old_tall_overlay_001.png) · [review](../assets/pr7243_review/native_old_tall_review_001.png) | [compare](../assets/pr7243_review/wasm_old_tall_compare_001.png) · [overlay](../assets/pr7243_review/wasm_old_tall_overlay_001.png) · [review](../assets/pr7243_review/wasm_old_tall_review_001.png) |
| tall_table_after p2 | [compare](../assets/pr7243_review/native_old_tall_compare_002.png) · [overlay](../assets/pr7243_review/native_old_tall_overlay_002.png) · [review](../assets/pr7243_review/native_old_tall_review_002.png) | [compare](../assets/pr7243_review/wasm_old_tall_compare_002.png) · [overlay](../assets/pr7243_review/wasm_old_tall_overlay_002.png) · [review](../assets/pr7243_review/wasm_old_tall_review_002.png) |
| short_table_after p1 | [compare](../assets/pr7243_review/native_old_short_compare_001.png) · [overlay](../assets/pr7243_review/native_old_short_overlay_001.png) · [review](../assets/pr7243_review/native_old_short_review_001.png) | [compare](../assets/pr7243_review/wasm_old_short_compare_001.png) · [overlay](../assets/pr7243_review/wasm_old_short_overlay_001.png) · [review](../assets/pr7243_review/wasm_old_short_review_001.png) |

새 입력의 위 표 PNG도 보정 코드로 재생성했다. Native binary SHA256
`61f9ce9b2845ea5a4fb8ba401e31e4a2a0be4932b5a2209aef18ccd9d4ebf4e6`, fresh WASM
`ad45bbf5f51c7dc8aec895c28ade067267ec33afec08344bc4ed6f4e2cdeecfe`.
기준 `fc5df351b` + 본 회차 제품 소스 3개 파일 변경이며 최종 게이트를 아래에 추가한다.


## 정상 원본 복구 회차 — 입력 계보와 회귀 oracle 정정

사용자가 지정한 정상 원본으로 `samples/86712_regulatory_analysis.hwp`를 교체했다.
앞 회차의 테스트·WASM 결과를 현재 수정 전체의 통과로 사용하지 않는다. 최종 재실행 결과는 문서 끝에 따로 기록한다.

### 분석: 잘못된 입력이 들어온 경로

| Git 이력 | 확인 내용 | 처리 |
| --- | --- | --- |
| `1a9131b47` / PR #1936 | HWP 123392바이트 최초 추가. 이후 이 경로의 파일 교체 이력 없음 | 정상 원본 151040바이트로 교체 |
| `193cd714d` | HWP5-origin pagination과 NO_LS 대응. 여러 다른 입력도 근거로 사용 | 일괄 revert 금지, 저장 줄과 실제 소비 경계로 재검증 |
| `2f9017966` / #2007 | 한 쪽보다 작은 1×1 자식 표를 통째로 다루는 조건 | 정상 원본의 저장 쪽 경계를 버리는 호출 경로 보정 |
| `bde419f3e` / #2195 | 86712=65쪽을 목표로 빈 문단·중첩 표·폰트 측정 변경 | 65쪽 기준 폐기. KoPub 1em 보정은 이미 #6389에서 복구됨 |
| `8ae55c243` / #2240 | 잘못된 HWP에서 rhwp export-hwpx로 파생 입력 재생성 | 정상 HWP로 같은 HWPX 경로 재생성 |
| `0df1911c7` / #2279 | 선행 묶음 80% 이월·내부 표 전 컷 보정과 p28/p29 기대값 | 새 PDF로 기대값 정정. 보정 제거만으로 시각 차이가 해결되지는 않아 단순 삭제하지 않음 |

잘못된 HWP의 원시 파서 결과는 재귀 문단 2101개 중 LineSeg 없는 문단 1810개,
저장 줄 370개다. 정상 원본은 같은 2101개 문단에 LineSeg 2785개, 줄 정보 없는 문단 0개다.
따라서 이 파일을 NO_LS 실물 oracle로 계속 사용하는 것은 잘못이다.

### 독립 기준과 재생성

- 정상 HWP: SHA-256 `ee82c7755617003cb972ba398da9cffadfed24ac0fa068eee1a5347da7658a88`,
  한컴 2024 `13.0.0.3901`. 이전 HWP SHA-256은
  `32e2ed30e5d744ad747f04f090c022eca8270f9dd2d55e0613e2ad61058099e9`이며 Git 이력에 보존한다.
- 직접 HWP→PDF: `hwp2024-mcp-convert`, engine 2024, preprocess none,
  job `228e18dc-1080-4d62-bc52-725420647edc`, succeeded/status/download 확인.
  [기준 PDF](../../../pdf/86712_regulatory_analysis-hwp-2024.pdf) **64쪽**,
  SHA-256 `bc1025b0607bbac01fea960997fa54430fd8dcc2604831b02940e7815cbcf84f`.
- HWP5-origin marker 회귀 검사를 위해 정상 원본에서 `rhwp export-hwpx --verify-pages --json`으로
  기존 `samples/issue1891/86712_regulatory_analysis.hwpx`를 재생성했다(64→64).
  107825바이트, SHA-256 `0f4f055c74a3d39f70e417ca6c700880d9645a202798cd0e5c11f6e32c180a19`.
- 재생성 HWPX의 직접 한컴 PDF: job `fc942264-2674-4c93-89ae-22fd70020a60`, engine 2024,
  succeeded/status/download 확인. [HWPX 기준 PDF](../../../pdf/86712_regulatory_analysis-hwpx-2024.pdf)
  **64쪽**, SHA-256 `5a0b0038d4bc33a391f9f76d2e657d78ce63d1a34c288428ed37d5350cd31955`.
  HWP와 HWPX PDF는 10쪽 들여쓰기·후속 내용 위치가 다르므로 동일 시각 oracle로 합치지 않는다.
- 과거 `samples/issue1891/86712_regulatory_analysis-2024.pdf`,
  `pdf/issue1891/86712_regulatory_analysis-hwpx-2020.pdf`,
  `pdf/issue1921/86712_regulatory_analysis-2024.pdf`와 옛 검토 PNG는 과거 계보의 기록이다.
  이번에 교체한 입력의 기준으로 재사용하지 않으며 활성 회귀의 PDF 참조는 위 두 직접 변환본이다.

### 회귀 검사 전수 참조 점검

| 소비자 | 정정·재검증 |
| --- | --- |
| `issue_1891` | 정상 HWP/HWPX 64쪽 왕복. 65쪽/NO_LS 설명 제거 |
| `issue_2279_layout_oracles` | 기존 ignored 65쪽 p28/p29 검사 대신 p25→p26 저장 이어받기·p27/p28 내용 소유 검사 활성화. 정상 PDF에서 대안 표 전체가 p10임을 확인하고 p11 분할 기대값 제거. 본문 줄 간격은 PDF 22.32~22.44pt로 검증 |
| `issue_5804`, `issue_5830` | 새 HWPX PDF p33/p34를 근거로 변경. 하이픈 advance 관측 0.496~0.573em, 기존 허용 범위 확대 없음 |
| `issue_7080` | 정상 HWP로 U+318D 검사 재실행. 새 PDF의 MalgunGothic 45개·BatangChe 1개에서 1.0em 재측정; 과거 Batang 0.992em 설명 제거 |
| `issue_3500`와 same-id corpus | 추출기로 정상 원본 재추출: HWP 0건, HWPX의 section 0/para 1 `[(0,0),(0,0)]` 항목 동일. 코퍼스 변경 불필요 |
| `render_page_samples.tsv`, `oracle_page_count_baseline.tsv` | 새 독립 PDF에 따른 64쪽으로 정정 |
| `ir_field_sweep_baseline.tsv` | 원본/재생성 HWP5 및 HWPX 직접 왕복 모두 발산 0. 잘못된 입력에 종속된 list_header_width_ref 허용 3행(840/30/56) 제거 |
| overflow-cell / body-overflow / text-overlap / off-canvas baseline | 각 검사 정의에 따른 정상 HWP/HWPX 전 64쪽 위반 허용을 0건으로 강화. body-overflow는 본문 하단 전용이며 CLI의 우측 넘침 6건과 구분한다 |
| `scripts/tests/test_fidelity_compare.py`의 `86712...065.svg` | 실제 문서를 읽지 않는 임시 합성 SVG 이름. 실제 문서 baseline과 분리해 판단 |

수정한 소비자 6개 모듈을 검증 전용 checkout에서 재실행했다:
`issue_2279_layout_oracles` 4, `issue_1891` 4, `issue_5804` 3, `issue_5830` 2,
`issue_7080` 5, `issue_3500` 12 — **30 passed**.
이는 회복 후보 1의 결과이며 뒤이은 엔진 수정의 최종 회귀 통과로 세지 않는다.
정상 HWP 보안 검사 6개도 통과했다. harness 배정이 바뀌어 0건 실행된 시도는 모두 제외하고,
`node scripts/rust-test-suite-manifest.mjs --prepare` 후 실제 실행 수를 확인했다.

### 엔진 복구와 소비 경로

정상 원본에서 25쪽 중첩 표의 다음 쪽 두 줄이 누락되는 원인을 추적했다.
`cell_units → nested_table_mixed_fragment_heights → 부모 단일 셀 분할 허용 → child RowCut 전달`
세 소비 지점에서 저장 프레임 경계를 보존하는 보정으로 p25 겹침과 p26 누락이 개선됐다.
후속 p28/p29에서는 본문 절반보다 작은 저장 프레임이 무시됐다.
정상 저장값 `26188 + 1300 + 141 + 141 = 27770 HU`(직전 줄 잉크 끝 + 상하 여백 = 선언 상자)로
작은 첫 조각의 경계를 보존했다. 같은 문단의 저장 reset 전에 이미 두 줄이 들어갔는데도
한 줄을 고아 줄로 되감던 처리를 바로잡아 p26/p27의 2+2줄, p27 하단 내부 표 제목,
p28 정성적 편익과 p29 이어받기를 복구했다. 실제 1줄 고아 줄인 synam 대조군은 보존한다.

법령 비교표 p4~8·32~33은 다음 경로를 함께 수정했다.

1. `cell_units`가 보존한 실제 저장 빈 줄을 `advance_row_cut_inner`와 block cut이 높이 없는
   보조 공백으로 건너뛰고, `row_cut_content_height`와 paint는 다시 높이에 더했다.
   `empty_unit_has_stored_line_box`로 실제 저장 줄의 높이를 컷 단계에서 보존한다.
2. 빈 줄만의 reset은 강제 분할하지 않는 기존 규칙을 유지하되, 다른 셀도 같은 저장 원점/높이에서
   함께 되감긴 경계는 `cut_has_shared_stored_frame`으로 공동 물리 경계임을 확인한다.
3. 선택된 컷의 끝 줄 뒤 간격은 `native_multirow_saved_reset_trailing_trim`에서 빼며,
   같은 문단 안의 저장 물리 경계도 포함한다. 빈 줄의 단독 경계는 앞 행 + 저장 줄 끝 + 패딩이
   선언 object frame을 정확히 닫는 경우만 인정한다. 이 높이를 스캔 예약과 partial-table 배치가
   함께 소비하므로 외곽선 좌표를 clamp하거나 텍스트를 숨기지 않는다.
4. 빈 문단을 일괄 제외했던 중간 후보는 hwpctl p56과 #7032 회귀를 만들어 **철회**했다.
   최종 후보에서 해당 13개 대조 검사는 모두 통과했다.

`kps-ai`의 행 번호를 `0..15`/`15..32`로 바꾼 중간 시도는 철회했다. 이 입력은
교체한 `86712`와 무관하며, 넓게 적용한 whole-row 예약 변경의 영향이었다. 기존 검사를
유지하고 이번 보정은 저장 object frame과 실제 전체 행 높이 합이 같은 경로에서만
패딩 차이를 복원하도록 좁힌다. 중간 후보의 kps-ai 캡처는 최종 통과 증거로 사용하지 않는다.

### 회복 후보 10 검증 기록

- 정상 입력 HWP/HWPX 각각 **64쪽, cell overflow 0 / off-canvas 0 / body-bottom overflow 0 /
  text overlap 0**. body 검사 tolerance는 기존 2px이며 증가시키지 않았다.
- 정상 원본의 파서 회귀는 두 입력 각각 재귀 문단 2101개에 저장 줄이 존재하고 합성 tag가 없음을
  확인한다. 과거 잘못된 입력은 1810개 문단이 이 계약을 위반한다.
- `issue_2279_layout_oracles` **7 passed**. 비교표의 물리 본문 수용 검사는 보정 전 후보에서
  p4 4.31px / p5 5.91px 초과로 실제 실패했고, 최종 후보는 HWP/HWPX 모두 통과했다.
- 최종 focused 19개 모듈 **80 passed**, same-id char-shape 검사 **12 passed**,
  정상 HWP/HWPX를 명시한 security corpus **6 passed**. 잘못된 모듈 이름으로 실행하지 못한
  1회는 제외하고 `issue_3500_char_shapes_roundtrip`으로 재실행했다.
- 검증 checkout HEAD는 `36235b8ad`이나 수정 소스를 복사해 검증했다. 기준 source HEAD
  `fc5df351b` + 이 회차 변경이며, 제품 소스 네 파일이 주 checkout과 일치함을 SHA-256으로 확인했다.
  생성 harness/manifest는 검증 전용이며 제출에 포함하지 않는다.
- 후보 10 전체 release-test: **9997 passed / 16 failed / 50 skipped**, 실행 514.064초, exit 100.
  Native Skia lib: **3927 passed / 3 failed / 13 ignored**, exit 101. 실패를 통과로 세지 않는다.
  다른 문서의 거대 셀·짧은 중첩 표·말미 빈 줄에 대한 회귀가 있어 이 엔진 후보는 제출하지 않는다.
  실패한 다른 문서의 기대값을 바꾸지 않고 보정 범위를 재검토한다.
- 후보 10 fmt, Native/WASM/workspace Clippy, workspace build, suite manifest base 비교 통과.
  fresh WASM 최적화 빌드 완료(SHA-256 `ff4f0205d9492e6fdd3f7f08ec19c60cfe943a3392f9dc740a895e0d8df5b38f`).
  정상 HWP 15쪽 및 정상 HWPX 3쪽을 각각 자기 입력의 PDF와 대조했다. 이후 변경의 증거로 재사용하지 않는다.

### 시각 확인 범위와 남은 차이

Native에서 정상 HWP 4~8·10·25~29·32~34·64쪽을 새로 캡처했다. 25→26 이어받기 누락,
27→28 내부 표/정성적 편익 소유, 29쪽 겹침, 법령 비교표의 과대한 아래쪽 확장은 복구됐다.
글꼴 공급 환경의 차이와 p26 이후 표의 수 px 위치 차이는 아직 보이며, 0건 진단을 화소 일치로
표현하지 않는다. KoPub을 한컴 PDF에 관측된 바탕 계열로 치환한 대조 캡처도 만들었으나,
이 치환만으로 모든 위치 차이가 사라지지는 않았다. 기존 원본이 잘못되어 완전 일치한 것처럼
취급했던 과거 증거는 현재 정상 문서의 승인 근거가 아니다.

### 정상 대조군과 입력 교체의 구분

2026-09-17 `git fetch upstream devel` 후 기준은 그대로 `236a601da803b53429e9090eef652c661dd3bfe2`다.
새 정상 파일 등록으로 무효가 된 `86712`의 65쪽·NO_LS 전제는 정상 원본의 PDF를 기준으로
수정한다. 파일을 바꾸지 않은 다른 문서의 회귀 기대값은 그대로 둔다. 동일 입력에
upstream 조판 코드를 적용한 대조 실행에서 `table_giant_cell_overfill.hwpx`는 48쪽,
`18095317_eogu_geumji.hwp`는 21쪽이다. 후보 10/11/13의 49쪽·20쪽 변화는
정상 원본 교체의 baseline 수정으로 덮을 수 없으므로 원인을 분리하는 중이다.


### 회복 후보 22 — 정상 원본 등록과 회귀 기준 확정

정상 HWP와 Downloads 원본의 SHA-256이 일치함을 재확인했다. 파일명을 바꾸거나
중복 원본을 추가하지 않고 기존 tracked 경로를 교체한다. 파생 HWPX 및 직접 한컴 PDF도
위 계보대로 교체하며, 임시 로그·JSON·TSV를 새 증적 파일로 추가하지 않는다.

후보 10에서 드러난 실제 회귀는 다음 소비 조건을 고쳤다.

- 저장된 빈 줄은 다음 저장 원점으로 실제 전진하고 동행 셀에도 같은 저장 밴드가 있을 때
  공간을 보존한다. 합성 줄·종료 spacer·한 셀만의 로컬 reset을 물리 쪽 경계로 승격하지 않는다.
  이 구분으로 거대 셀의 48→49쪽, 18095317의 21→20쪽 회귀를 기존 기대값으로 검사한다.
- 작은 자식 frame의 동일성은 컷이 실제 사용하는 **resolved padding**으로 판단한다.
  원시 여백만 더한 선언을 사용하면 최소 높이 셀에서 배치와 달라져 rowbreak 문서 p8의
  다음 문단과 0.94px 겹쳤다. 해당 겹침 검사는 기대값 변경 없이 통과했다.
- 전체 행 높이 합을 검증할 때 순수 텍스트 행은 paint 측정 높이, 중첩 내용 컷을 소비하는
  행은 컷 높이를 사용한다. 모든 행을 MeasuredTable 높이로 합친 가정 때문에 생긴
  76076 p81의 첫 줄 소유 회귀를 해결했다. 기존 p81/p82 oracle은 변경하지 않았다.

**PDF 근거로 잘못된 기존 좌표 핀을 정정한 한 건:** 76076 p4의 기존 1040~1052px
범위는 기준 `samples/issue1891/76076_regulatory_analysis-2024.pdf`의 실제 하단 좌표를
포함하지 않았다. PDF drawing의 세로 테두리 끝은 776.630pt = **1035.507px(96dpi)**다.
현재 Native는 1034.90px로 차이 0.61px이며 compare/overlay에서 하단 일치를 직접 확인했다.
종전 핀의 범위를 넓히지 않고 독립 PDF 실측 ±2px로 바꾸었다. 이 정정은 정상 86712 교체를
이유로 다른 문서의 실제 회귀를 허용한 것이 아니다. 원 코드의 약 1048.76px은 새 핀에서 실패한다.

검증 소스는 `fc5df351b` 위의 이 회차 네 제품 파일이며 주 checkout/검증 checkout의
바이트 일치를 확인했다(검증 checkout의 detached HEAD 자체를 검증 source SHA로 쓰지 않는다).

| 파일 | SHA-256 |
| --- | --- |
| `src/renderer/composer.rs` | `0a0a838941ed652140a6ec0ac2f8f3bcd7db6949aa92a26805359f9680070566` |
| `src/renderer/layout/table_layout.rs` | `264b11b0c281cbbc995f68bcec9224b6e294bf8f75f35dbe96e6eca89aceb51f` |
| `src/renderer/layout/table_partial.rs` | `edae68d9ae60f46074edb6ea0899cb9fc4b7b7140193e7c78c91df7c7e01f6c3` |
| `src/renderer/typeset.rs` | `8332fa28b8d2452c25836e580688b4d4ba3dcad6b17e50cbfb5dde901a8f9b70` |

Native binary SHA-256: `17785f4c61d97b6526663fcefee0a86c6776fd61f94c5b5026120d76112a3123`.
Fresh optimized WASM SHA-256: `7197788bcec3da2e2abefe6fa3a2299a47fa8d36922dc87425c074d7e6e86520`
(빌드 성공, 4분 12초). Native/fresh WASM 정상 HWP 15쪽, HWPX 3쪽 및 기존 입력 대조군을
각 입력에 대응하는 PDF로 다시 캡처했다. 다음 최종 결과 표에 완료된 검사만 추가한다.


### 후보 22의 실패와 최종 후보 26의 직접 회귀 보정

후보 22의 전체 release-test는 **10012 passed / 1 failed / 50 skipped**(442.338초,
exit 100)였다. 실패는 `body_overflow_baseline::body_overflow_does_not_grow_partition_11`의
파생 HWPX 실제 **26쪽 16.63px 넘침**이다. 진단 JSON의 `page=25`는 0-based다.
이 결과를 통과로 세지 않았고 baseline의 0건 기준도 완화하지 않았다.

이 과정에서 기존 `issue_2279_saved_equation_split_keeps_two_lines_on_each_page`가
문구 존재만 확인해 3+1/4+0줄도 통과시킬 수 있음을 확인했다. HWP/HWPX 양쪽의
실제 2035년 산식 셀(row 26, col 2)을 검사하도록 강화했다. 앞쪽은 `2032년 기본형건축비)`까지,
다음 쪽은 `주택면적`·`이자율`을 소유해야 한다. 후보 22에서 세 번째 줄이 앞쪽에 남아 **FAIL**,
후보 26에서 **PASS**다. 비교표 본문 수용 검사에도 두 포맷의 26쪽을 추가했다.

저장 원본은 다음 독립적인 물리 경계를 갖는다:

```text
앞 23행 23 × 1746 + 이어지는 3행 3 × 6426 = 59436 HU
현재 행의 두 번째 줄 끝 1560 + 1300 + 상하 여백 223 + 223 = 3306 HU
59436 + 3306 = 62742 HU = 표의 선언된 첫 조각 높이
```

`stored_row_reset_closes_declared_frame`은 control-free·유효 저장 줄·이전 reset 없음 조건에서
이 같은 원시 행 상자 합을 확인한다. `cell_units`의 frame 표지, ordinary RowCut scanner,
그리고 typeset의 **whole-row fast path**가 같은 판정을 소비한다. HWPX는 3.7px 작은 앞쪽
소비 높이 때문에 행 전체가 fit하는 빠른 분기로 빠졌으므로, scanner만 고치면 남던 회귀였다.
선언 경계가 행 내부에 있을 때는 충분한 공간이 있어도 해당 RowCut을 먼저 선택한다.
파일명·쪽수·문서별 상수로 분기하지 않고, 편집으로 reflow된 표의 whole-row 경로는 제외한다.

최종 후보의 전 페이지 CLI 진단은 두 입력 각각 **64쪽, off-canvas 0 / text-overlap 0 /
본문 하단 넘침(기존 2px 공차) 0**이다. `layout-anomaly`의 전체 `overflowCount`는 각각 **6**이며,
이는 1·2·30·31·49·50쪽 표의 우측 13.52px 초과다. 아래쪽 전용 회귀 게이트의 0건을
전체 방향 overflow 0건으로 표현하지 않는다. 새 HWPX 26쪽 넘침은 사라졌다.

정상 HWP의 26·27쪽 Native compare에서 산식의 실제 2+2줄 소유를 직접 확인했다.
25→26 이어받기, 표 제목/하단, 28→29 정성 편익 내용도 새 입력에 대응시켰다.
26쪽 첫 표 높이와 후속 표 시작에는 여전히 약 8px의 차이가 있고, 글꼴별 glyph/행 원점 차이도
남는다. 이 파일의 완전한 PDF 화소 일치 또는 통합 PR 전체 승인을 뜻하지 않는다.

최종 후보 26 제품 파일 SHA-256(아래 검증은 이 바이트에 적용):

| 파일 | SHA-256 |
| --- | --- |
| `src/renderer/composer.rs` | `0a0a838941ed652140a6ec0ac2f8f3bcd7db6949aa92a26805359f9680070566` |
| `src/renderer/layout/table_layout.rs` | `c90a3cd85634127ba76dbeec23f006ee294d3b832965251446c0404921026c5a` |
| `src/renderer/layout/table_partial.rs` | `edae68d9ae60f46074edb6ea0899cb9fc4b7b7140193e7c78c91df7c7e01f6c3` |
| `src/renderer/typeset.rs` | `212c37242a81f3e8ee813117d98352066d1fac9087e2f9eb4bf02ddd88de9ad4` |


### 최종 후보 26 — 재현 환경과 시각 증거

- Native SHA-256: `4467723c7a604d708a4aefb6e7cefdaacae2ca5391c46555a32ce6a1c78a0ac0`.
- fresh optimized WASM SHA-256: `8dc9187e1a4b884f127d11e0c4c496cacbcbec57ebeac521980247a095be9b3b`.
  `scripts/wasm-pack-locked.sh --target web --out-dir <scratch>/wasm-recovery-26`가
  wasm-opt까지 3분 19초에 성공했다.
- `DEVELOPER_DIR=/Library/Developer/CommandLineTools`,
  `CARGO_TARGET_DIR=/Users/tsjang/rhwp/target/pr7239-7240-review-20260917`.
  별도 검증 checkout과 제출 checkout의 변경 제품·테스트·baseline·fixture를 바이트 대조했다.
- Visual Sweep: 각 입력에 대응하는 PDF, `--dpi 96`,
  `--rhwp-bin <scratch>/rhwp-isolation-final26`, WASM은
  `--wasm-pkg <scratch>/wasm-recovery-26`를 명시했다.
  Native/fresh WASM 각각 34쪽(총 68쪽), compare·standalone overlay·review **204 PNG**를 갱신했다.
  정상 HWP 15쪽, 파생 HWPX 6쪽, 76076 5쪽, kps-ai 2쪽, 기존/새 긴·짧은 표 각 3쪽이다.
- 34쌍 전부 SVG의 text/tspan/rect/line/path 등 그리기 노드의 속성·텍스트가 동일하다.
  raster 완전 동일은 **18/34쌍**이다. Visual Sweep의 WASM 쪽은 문서 전체 font-face를
  보충하며 Native는 페이지별 font-face를 내보내므로 법령 비교표의 굵은 글꼴 등 raster 차이가
  남는다. 4·33쪽의 text 속성/좌표는 같고 CSS의 휴먼명조 bold 공급이 다른 것을 직접 대조했다.
  이를 새 조판 좌표 회귀나 두 backend의 전체 화소 일치로 표현하지 않는다.
- 26·27쪽 실제 산식 2+2줄, 25→26 중첩 표 이어받기, 28→29 정성 편익, 본문 내 표 바닥을
  Native와 WASM compare/overlay로 직접 판독했다. 76076·kps-ai·기존/새 긴·짧은 표도 대조했다.
  정상 86712 p26 첫 표와 다음 표는 PDF 대비 약 8px 차이가 남으며 글꼴 차이도 남는다.

Visual Sweep 자동 flag는 모두 0이지만, 글자 겹침 일치율은 아래처럼 낮다. 흰 배경을
포함한 pixel match나 flag 0을 PDF 조판의 완전 일치로 해석하지 않는다.

| 정상 입력 | Native 평균 pixel / ink match | fresh WASM 평균 pixel / ink match |
| --- | --- | --- |
| HWP 15쪽 | 90.632% / 9.913% | 90.701% / 9.923% |
| 파생 HWPX 6쪽 | 89.395% / 9.019% | 89.419% / 9.037% |

아래는 후보 26의 페이지별 증거다. 임시 로그·JSON·TSV와 font 포함 SVG는 커밋하지 않는다.

| 입력·쪽 | Native | fresh WASM |
| --- | --- | --- |
| 정상 HWP p4 | [compare](../assets/pr7243_review/native_corrected_86712_compare_004.png) · [overlay](../assets/pr7243_review/native_corrected_86712_overlay_004.png) · [review](../assets/pr7243_review/native_corrected_86712_review_004.png) | [compare](../assets/pr7243_review/wasm_corrected_86712_compare_004.png) · [overlay](../assets/pr7243_review/wasm_corrected_86712_overlay_004.png) · [review](../assets/pr7243_review/wasm_corrected_86712_review_004.png) |
| 정상 HWP p5 | [compare](../assets/pr7243_review/native_corrected_86712_compare_005.png) · [overlay](../assets/pr7243_review/native_corrected_86712_overlay_005.png) · [review](../assets/pr7243_review/native_corrected_86712_review_005.png) | [compare](../assets/pr7243_review/wasm_corrected_86712_compare_005.png) · [overlay](../assets/pr7243_review/wasm_corrected_86712_overlay_005.png) · [review](../assets/pr7243_review/wasm_corrected_86712_review_005.png) |
| 정상 HWP p6 | [compare](../assets/pr7243_review/native_corrected_86712_compare_006.png) · [overlay](../assets/pr7243_review/native_corrected_86712_overlay_006.png) · [review](../assets/pr7243_review/native_corrected_86712_review_006.png) | [compare](../assets/pr7243_review/wasm_corrected_86712_compare_006.png) · [overlay](../assets/pr7243_review/wasm_corrected_86712_overlay_006.png) · [review](../assets/pr7243_review/wasm_corrected_86712_review_006.png) |
| 정상 HWP p7 | [compare](../assets/pr7243_review/native_corrected_86712_compare_007.png) · [overlay](../assets/pr7243_review/native_corrected_86712_overlay_007.png) · [review](../assets/pr7243_review/native_corrected_86712_review_007.png) | [compare](../assets/pr7243_review/wasm_corrected_86712_compare_007.png) · [overlay](../assets/pr7243_review/wasm_corrected_86712_overlay_007.png) · [review](../assets/pr7243_review/wasm_corrected_86712_review_007.png) |
| 정상 HWP p8 | [compare](../assets/pr7243_review/native_corrected_86712_compare_008.png) · [overlay](../assets/pr7243_review/native_corrected_86712_overlay_008.png) · [review](../assets/pr7243_review/native_corrected_86712_review_008.png) | [compare](../assets/pr7243_review/wasm_corrected_86712_compare_008.png) · [overlay](../assets/pr7243_review/wasm_corrected_86712_overlay_008.png) · [review](../assets/pr7243_review/wasm_corrected_86712_review_008.png) |
| 정상 HWP p10 | [compare](../assets/pr7243_review/native_corrected_86712_compare_010.png) · [overlay](../assets/pr7243_review/native_corrected_86712_overlay_010.png) · [review](../assets/pr7243_review/native_corrected_86712_review_010.png) | [compare](../assets/pr7243_review/wasm_corrected_86712_compare_010.png) · [overlay](../assets/pr7243_review/wasm_corrected_86712_overlay_010.png) · [review](../assets/pr7243_review/wasm_corrected_86712_review_010.png) |
| 정상 HWP p25 | [compare](../assets/pr7243_review/native_corrected_86712_compare_025.png) · [overlay](../assets/pr7243_review/native_corrected_86712_overlay_025.png) · [review](../assets/pr7243_review/native_corrected_86712_review_025.png) | [compare](../assets/pr7243_review/wasm_corrected_86712_compare_025.png) · [overlay](../assets/pr7243_review/wasm_corrected_86712_overlay_025.png) · [review](../assets/pr7243_review/wasm_corrected_86712_review_025.png) |
| 정상 HWP p26 | [compare](../assets/pr7243_review/native_corrected_86712_compare_026.png) · [overlay](../assets/pr7243_review/native_corrected_86712_overlay_026.png) · [review](../assets/pr7243_review/native_corrected_86712_review_026.png) | [compare](../assets/pr7243_review/wasm_corrected_86712_compare_026.png) · [overlay](../assets/pr7243_review/wasm_corrected_86712_overlay_026.png) · [review](../assets/pr7243_review/wasm_corrected_86712_review_026.png) |
| 정상 HWP p27 | [compare](../assets/pr7243_review/native_corrected_86712_compare_027.png) · [overlay](../assets/pr7243_review/native_corrected_86712_overlay_027.png) · [review](../assets/pr7243_review/native_corrected_86712_review_027.png) | [compare](../assets/pr7243_review/wasm_corrected_86712_compare_027.png) · [overlay](../assets/pr7243_review/wasm_corrected_86712_overlay_027.png) · [review](../assets/pr7243_review/wasm_corrected_86712_review_027.png) |
| 정상 HWP p28 | [compare](../assets/pr7243_review/native_corrected_86712_compare_028.png) · [overlay](../assets/pr7243_review/native_corrected_86712_overlay_028.png) · [review](../assets/pr7243_review/native_corrected_86712_review_028.png) | [compare](../assets/pr7243_review/wasm_corrected_86712_compare_028.png) · [overlay](../assets/pr7243_review/wasm_corrected_86712_overlay_028.png) · [review](../assets/pr7243_review/wasm_corrected_86712_review_028.png) |
| 정상 HWP p29 | [compare](../assets/pr7243_review/native_corrected_86712_compare_029.png) · [overlay](../assets/pr7243_review/native_corrected_86712_overlay_029.png) · [review](../assets/pr7243_review/native_corrected_86712_review_029.png) | [compare](../assets/pr7243_review/wasm_corrected_86712_compare_029.png) · [overlay](../assets/pr7243_review/wasm_corrected_86712_overlay_029.png) · [review](../assets/pr7243_review/wasm_corrected_86712_review_029.png) |
| 정상 HWP p32 | [compare](../assets/pr7243_review/native_corrected_86712_compare_032.png) · [overlay](../assets/pr7243_review/native_corrected_86712_overlay_032.png) · [review](../assets/pr7243_review/native_corrected_86712_review_032.png) | [compare](../assets/pr7243_review/wasm_corrected_86712_compare_032.png) · [overlay](../assets/pr7243_review/wasm_corrected_86712_overlay_032.png) · [review](../assets/pr7243_review/wasm_corrected_86712_review_032.png) |
| 정상 HWP p33 | [compare](../assets/pr7243_review/native_corrected_86712_compare_033.png) · [overlay](../assets/pr7243_review/native_corrected_86712_overlay_033.png) · [review](../assets/pr7243_review/native_corrected_86712_review_033.png) | [compare](../assets/pr7243_review/wasm_corrected_86712_compare_033.png) · [overlay](../assets/pr7243_review/wasm_corrected_86712_overlay_033.png) · [review](../assets/pr7243_review/wasm_corrected_86712_review_033.png) |
| 정상 HWP p34 | [compare](../assets/pr7243_review/native_corrected_86712_compare_034.png) · [overlay](../assets/pr7243_review/native_corrected_86712_overlay_034.png) · [review](../assets/pr7243_review/native_corrected_86712_review_034.png) | [compare](../assets/pr7243_review/wasm_corrected_86712_compare_034.png) · [overlay](../assets/pr7243_review/wasm_corrected_86712_overlay_034.png) · [review](../assets/pr7243_review/wasm_corrected_86712_review_034.png) |
| 정상 HWP p64 | [compare](../assets/pr7243_review/native_corrected_86712_compare_064.png) · [overlay](../assets/pr7243_review/native_corrected_86712_overlay_064.png) · [review](../assets/pr7243_review/native_corrected_86712_review_064.png) | [compare](../assets/pr7243_review/wasm_corrected_86712_compare_064.png) · [overlay](../assets/pr7243_review/wasm_corrected_86712_overlay_064.png) · [review](../assets/pr7243_review/wasm_corrected_86712_review_064.png) |
| 정상 파생 HWPX p10 | [compare](../assets/pr7243_review/native_corrected_hwpx_compare_010.png) · [overlay](../assets/pr7243_review/native_corrected_hwpx_overlay_010.png) · [review](../assets/pr7243_review/native_corrected_hwpx_review_010.png) | [compare](../assets/pr7243_review/wasm_corrected_hwpx_compare_010.png) · [overlay](../assets/pr7243_review/wasm_corrected_hwpx_overlay_010.png) · [review](../assets/pr7243_review/wasm_corrected_hwpx_review_010.png) |
| 정상 파생 HWPX p25 | [compare](../assets/pr7243_review/native_corrected_hwpx_compare_025.png) · [overlay](../assets/pr7243_review/native_corrected_hwpx_overlay_025.png) · [review](../assets/pr7243_review/native_corrected_hwpx_review_025.png) | [compare](../assets/pr7243_review/wasm_corrected_hwpx_compare_025.png) · [overlay](../assets/pr7243_review/wasm_corrected_hwpx_overlay_025.png) · [review](../assets/pr7243_review/wasm_corrected_hwpx_review_025.png) |
| 정상 파생 HWPX p26 | [compare](../assets/pr7243_review/native_corrected_hwpx_compare_026.png) · [overlay](../assets/pr7243_review/native_corrected_hwpx_overlay_026.png) · [review](../assets/pr7243_review/native_corrected_hwpx_review_026.png) | [compare](../assets/pr7243_review/wasm_corrected_hwpx_compare_026.png) · [overlay](../assets/pr7243_review/wasm_corrected_hwpx_overlay_026.png) · [review](../assets/pr7243_review/wasm_corrected_hwpx_review_026.png) |
| 정상 파생 HWPX p27 | [compare](../assets/pr7243_review/native_corrected_hwpx_compare_027.png) · [overlay](../assets/pr7243_review/native_corrected_hwpx_overlay_027.png) · [review](../assets/pr7243_review/native_corrected_hwpx_review_027.png) | [compare](../assets/pr7243_review/wasm_corrected_hwpx_compare_027.png) · [overlay](../assets/pr7243_review/wasm_corrected_hwpx_overlay_027.png) · [review](../assets/pr7243_review/wasm_corrected_hwpx_review_027.png) |
| 정상 파생 HWPX p33 | [compare](../assets/pr7243_review/native_corrected_hwpx_compare_033.png) · [overlay](../assets/pr7243_review/native_corrected_hwpx_overlay_033.png) · [review](../assets/pr7243_review/native_corrected_hwpx_review_033.png) | [compare](../assets/pr7243_review/wasm_corrected_hwpx_compare_033.png) · [overlay](../assets/pr7243_review/wasm_corrected_hwpx_overlay_033.png) · [review](../assets/pr7243_review/wasm_corrected_hwpx_review_033.png) |
| 정상 파생 HWPX p34 | [compare](../assets/pr7243_review/native_corrected_hwpx_compare_034.png) · [overlay](../assets/pr7243_review/native_corrected_hwpx_overlay_034.png) · [review](../assets/pr7243_review/native_corrected_hwpx_review_034.png) | [compare](../assets/pr7243_review/wasm_corrected_hwpx_compare_034.png) · [overlay](../assets/pr7243_review/wasm_corrected_hwpx_overlay_034.png) · [review](../assets/pr7243_review/wasm_corrected_hwpx_review_034.png) |
| 76076 p4 | [compare](../assets/pr7243_review/native_original_76076_compare_004.png) · [overlay](../assets/pr7243_review/native_original_76076_overlay_004.png) · [review](../assets/pr7243_review/native_original_76076_review_004.png) | [compare](../assets/pr7243_review/wasm_original_76076_compare_004.png) · [overlay](../assets/pr7243_review/wasm_original_76076_overlay_004.png) · [review](../assets/pr7243_review/wasm_original_76076_review_004.png) |
| 76076 p33 | [compare](../assets/pr7243_review/native_original_76076_compare_033.png) · [overlay](../assets/pr7243_review/native_original_76076_overlay_033.png) · [review](../assets/pr7243_review/native_original_76076_review_033.png) | [compare](../assets/pr7243_review/wasm_original_76076_compare_033.png) · [overlay](../assets/pr7243_review/wasm_original_76076_overlay_033.png) · [review](../assets/pr7243_review/wasm_original_76076_review_033.png) |
| 76076 p34 | [compare](../assets/pr7243_review/native_original_76076_compare_034.png) · [overlay](../assets/pr7243_review/native_original_76076_overlay_034.png) · [review](../assets/pr7243_review/native_original_76076_review_034.png) | [compare](../assets/pr7243_review/wasm_original_76076_compare_034.png) · [overlay](../assets/pr7243_review/wasm_original_76076_overlay_034.png) · [review](../assets/pr7243_review/wasm_original_76076_review_034.png) |
| 76076 p81 | [compare](../assets/pr7243_review/native_original_76076_compare_081.png) · [overlay](../assets/pr7243_review/native_original_76076_overlay_081.png) · [review](../assets/pr7243_review/native_original_76076_review_081.png) | [compare](../assets/pr7243_review/wasm_original_76076_compare_081.png) · [overlay](../assets/pr7243_review/wasm_original_76076_overlay_081.png) · [review](../assets/pr7243_review/wasm_original_76076_review_081.png) |
| 76076 p82 | [compare](../assets/pr7243_review/native_original_76076_compare_082.png) · [overlay](../assets/pr7243_review/native_original_76076_overlay_082.png) · [review](../assets/pr7243_review/native_original_76076_review_082.png) | [compare](../assets/pr7243_review/wasm_original_76076_compare_082.png) · [overlay](../assets/pr7243_review/wasm_original_76076_overlay_082.png) · [review](../assets/pr7243_review/wasm_original_76076_review_082.png) |
| kps-ai p37 | [compare](../assets/pr7243_review/native_kps_ai_compare_037.png) · [overlay](../assets/pr7243_review/native_kps_ai_overlay_037.png) · [review](../assets/pr7243_review/native_kps_ai_review_037.png) | [compare](../assets/pr7243_review/wasm_kps_ai_compare_037.png) · [overlay](../assets/pr7243_review/wasm_kps_ai_overlay_037.png) · [review](../assets/pr7243_review/wasm_kps_ai_review_037.png) |
| kps-ai p38 | [compare](../assets/pr7243_review/native_kps_ai_compare_038.png) · [overlay](../assets/pr7243_review/native_kps_ai_overlay_038.png) · [review](../assets/pr7243_review/native_kps_ai_review_038.png) | [compare](../assets/pr7243_review/wasm_kps_ai_compare_038.png) · [overlay](../assets/pr7243_review/wasm_kps_ai_overlay_038.png) · [review](../assets/pr7243_review/wasm_kps_ai_review_038.png) |
| 기존 긴 표 p1 | [compare](../assets/pr7243_review/native_old_tall_compare_001.png) · [overlay](../assets/pr7243_review/native_old_tall_overlay_001.png) · [review](../assets/pr7243_review/native_old_tall_review_001.png) | [compare](../assets/pr7243_review/wasm_old_tall_compare_001.png) · [overlay](../assets/pr7243_review/wasm_old_tall_overlay_001.png) · [review](../assets/pr7243_review/wasm_old_tall_review_001.png) |
| 기존 긴 표 p2 | [compare](../assets/pr7243_review/native_old_tall_compare_002.png) · [overlay](../assets/pr7243_review/native_old_tall_overlay_002.png) · [review](../assets/pr7243_review/native_old_tall_review_002.png) | [compare](../assets/pr7243_review/wasm_old_tall_compare_002.png) · [overlay](../assets/pr7243_review/wasm_old_tall_overlay_002.png) · [review](../assets/pr7243_review/wasm_old_tall_review_002.png) |
| 기존 짧은 표 p1 | [compare](../assets/pr7243_review/native_old_short_compare_001.png) · [overlay](../assets/pr7243_review/native_old_short_overlay_001.png) · [review](../assets/pr7243_review/native_old_short_review_001.png) | [compare](../assets/pr7243_review/wasm_old_short_compare_001.png) · [overlay](../assets/pr7243_review/wasm_old_short_overlay_001.png) · [review](../assets/pr7243_review/wasm_old_short_review_001.png) |
| 새 긴 표 p1 | [compare](../assets/pr7243_review/native_tall_compare_001.png) · [overlay](../assets/pr7243_review/native_tall_overlay_001.png) · [review](../assets/pr7243_review/native_tall_review_001.png) | [compare](../assets/pr7243_review/wasm_tall_compare_001.png) · [overlay](../assets/pr7243_review/wasm_tall_overlay_001.png) · [review](../assets/pr7243_review/wasm_tall_review_001.png) |
| 새 긴 표 p2 | [compare](../assets/pr7243_review/native_tall_compare_002.png) · [overlay](../assets/pr7243_review/native_tall_overlay_002.png) · [review](../assets/pr7243_review/native_tall_review_002.png) | [compare](../assets/pr7243_review/wasm_tall_compare_002.png) · [overlay](../assets/pr7243_review/wasm_tall_overlay_002.png) · [review](../assets/pr7243_review/wasm_tall_review_002.png) |
| 새 짧은 표 p1 | [compare](../assets/pr7243_review/native_short_compare_001.png) · [overlay](../assets/pr7243_review/native_short_overlay_001.png) · [review](../assets/pr7243_review/native_short_review_001.png) | [compare](../assets/pr7243_review/wasm_short_compare_001.png) · [overlay](../assets/pr7243_review/wasm_short_overlay_001.png) · [review](../assets/pr7243_review/wasm_short_review_001.png) |


### 최종 후보 26 — 완료된 검증과 회차 결과

| 검사 | 실제 결과 |
| --- | --- |
| focused 13개 모듈 | 73 passed / 0 failed |
| fmt, Native Clippy, WASM32 lib Clippy, workspace build, workspace all-target Clippy | 모두 exit 0, Clippy 세 단계 `-D warnings` |
| suite manifest base 비교 | `236a601da803b53429e9090eef652c661dd3bfe2` 대비 exit 0 |
| 전체 release-test nextest | 10013 tests run: 10013 passed (3 slow), 50 skipped / exit 0 |
| Native Skia lib | rhwp 3930 passed / 0 failed / 13 ignored, workspace 182 passed / exit 0 |
| Native Skia 누락 이미지 | 2 tests run: 2 passed, 202 skipped / exit 0 |
| Native Skia 직접 PDF | 4 tests run: 4 passed, 201 skipped / exit 0 |
| fresh optimized WASM / Visual Sweep | 빌드 exit 0, Native/WASM 각 34쪽 캡처 완료. PDF 차이는 위 지표·PNG로 공개 |

실행 명령은 다음과 같다. `run-rust-test`의 필터 밖 skipped는 focused 미선택이며,
전체 회귀의 50 skipped 및 Skia lib의 13 ignored를 실행 완료로 세지 않는다.

```sh
node scripts/run-rust-test.mjs <13개 모듈> -- --cargo-profile release-test --target-dir "$CARGO_TARGET_DIR" --no-fail-fast
cargo fmt --all -- --check
cargo clippy --locked -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown -- -D warnings
cargo build --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
node scripts/rust-test-suite-manifest.mjs --check --base-ref 236a601da803b53429e9090eef652c661dd3bfe2
cargo nextest run --locked --cargo-profile release-test --tests --test-threads 6 --no-fail-fast
cargo test --locked --profile release-test --features native-skia --lib -- --test-threads 6
node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --target-dir "$CARGO_TARGET_DIR" --features native-skia
node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --target-dir "$CARGO_TARGET_DIR" --features native-skia
```

focused 모듈: `issue_1891`, `issue_2279_layout_oracles`,
`issue_2308_render_normalized_derived_state`, `issue_3820_rowbreak_rowspan_band`,
`issue_rowbreak_chart_overlap`, `issue_1156_rowbreak_fragment_fit`,
`issue_2097_rowbreak_midpage_declared_fits`, `issue_3128_terminal_nested_table_geometry`,
`issue_7234_scaffold_table_cell_height`, `issue_5804_dash_leader_glyphs`,
`issue_5830_dash_leader_last_line`, `issue_7080_area_dot_fullwidth`,
`issue_3500_char_shapes_roundtrip`.

전체 nextest 실행은 414.653초(컴파일 포함 명령 639.45초)에 완료했다.
`RHWP_SECURITY_SWEEP_SAMPLES_JSON`에는 정상 HWP와 재생성 HWPX를 명시했다.
제품 내부 `#[cfg(test)]` 변경은 없어 unit-tier 증가 비교는 비해당이다.

**회차 결과:** 정상 HWP를 기존 tracked 경로에 등록하고 파생 HWPX·직접 한컴 PDF·활성 회귀
기준을 함께 정정했다. 잘못된 파일의 65쪽/NO_LS 기대값과 허용 baseline을 제거하고,
저장 줄 소실·2+2줄 소유·본문 하단 수용을 검사한다. 기존 입력의 실제 회귀를 기대값 변경으로
숨기지 않고 저장 frame·빈 줄·전체 행 높이를 소비하는 경로를 보정했다.
76076 p4의 예외적인 핀 정정은 독립 PDF 좌표 근거와 수정 전 실패를 위에 명시했다.

PDF의 글꼴·일부 위치 차이와 #7242 원 합성 입력 보류는 남는다. 이번 회차를 통합 PR
전체 승인으로 보고하지 않는다. 원격 push·GitHub comment·merge는 이 회차에서 수행하지 않았다.
분석·수정·실제 검증 결과보고를 마친 후 이 회차 파일을 함께 커밋한다.
