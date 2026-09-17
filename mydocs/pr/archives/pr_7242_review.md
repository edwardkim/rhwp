---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-18
---

# PR #7242 검토

**검토 승인 — 공개 입력 정상화와 표 뒤 흐름 검증의 보류 사유 해소. 최종 통합 head CI는 별도 확인.** #7239·#7240 통합 후보에 누적 적용했다.

최신 판정은 아래 2026-09-18 회차를 따른다. 그 뒤의 기존 기록·실패 PNG는 수정 전 이력이며 삭제하지 않는다.

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

2026-09-18 표의 `fixed_` compare·overlay·review 12개를 최종 증거로 사용한다. 이전 실패 PNG는 보정 전 설명에만 연결한다. 통합 PR의 최종 head CI와 merge가 끝나면 실제 merge SHA·CI URL, 원 기여와 메인터너 보정, 검증 범위·남은 차이를 한국어로 설명하고 감사한다. [Visual Sweep 정본](../../../mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결한다. 최종 Native/fresh WASM compare·standalone overlay·review PNG를 merge SHA raw URL로 본문에 표시한다. 비공개 검증 자료를 공개 자료로 바꾸어 쓰지 않는다. UTF-8 본문 파일과 `--body-file`로 게시하고 원문과 이미지 URL을 다시 확인한다. 관련 공개 issue는 없어 임의 종료하지 않는다.

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
