# PR #8477 리뷰 — 그림 칸의 저장 프레임·정렬과 분할 소유 보정

## 최종 판정

**메인터너 보정 후 수용 가능**

완료한 실측과 남은 통합 공통 검증을 구분합니다. 통합 PR #8622 생성과 code head CI는 완료했습니다. 원 PR 직접 승인·닫기와 통합 병합은 아직 수행하지 않았습니다.
WASM의 당시 base 대비 전쪽 대조와 최신 필수 검증은 완료했으며 아래에 실제 증거를 연결합니다. 준비 완료 후 PR 생성·CI 모니터링은 작업지시자가 승인했습니다.


최신 통합 code head `acbc0bfeb5660133b0732ce77eae16bd50b29f9f`의 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/41965861415)와 CodeQL·Render Diff·Adapter·Proptest가 모두 성공했습니다.
작업지시자가 CI 완료 뒤 문서 기록·병합·후속 처리를 승인했습니다. 원 head는 그대로 보존하고 보정된 [통합 PR #8622](https://github.com/edwardkim/rhwp/pull/8622)로 처리합니다.
이 문서는 code head CI 성공 뒤의 문서 전용 trailing 기록입니다. push 뒤 trailing head의 실제 fast-pass·필수 검사·최신 mergeability 확인을 병합 조건으로 둡니다.

## 이전 완료 제품과 재사용 증거의 출처

| 항목 | 값 |
| --- | --- |
| 원 PR | [PR #8477](https://github.com/edwardkim/rhwp/pull/8477) |
| 원 head | `d53da96d75daf912d3e53d9137eadc7770732bbf` |
| 현재 통합 적용 커밋 | `bc53d309003ba5eceb002a91566b083d99f55e23, 74e8d3bc44d92ccd50a223cbd468123d29ef40b5, 2bee09e883f7e153117d15c1de9413ad10e5b223, f1bf4da00f0668b1e852bb5422f85067cef81d00` |
| 이전 완료 제품 / 당시 base | `a7e5ce7112ea8b227c317a9cbfb9d6fed0b00973` / `22947f6a90c998e95fa51238d5b907a0ebd37162` |
| 전쪽 Native 측정 producer / 당시 base | `4b149b6cdfc514b9858ed644be2cf8120c731cd4` / `7fdd5441ff1e77c083818e920a48089f294de35e` |
| fresh WASM 실제 빌드 | `7bb9e66fd006109ca9f7ee4c4d3f752c82e60fae` (위 제품 코드와 동등한 구현) |
| reviewer | `jangster77` 요청. `2026-11-05 03:16:02 UTC` 원격 재조회에서 원 head 불변·Open·non-Draft 확인 |

기존 전쪽 TSV/PNG는 실제 측정 producer의 결과로 재사용합니다. 최신 소스의 재출력으로 표시하지 않습니다.
이전 a7 제품의 최소 글꼴 전체 Rust 12,561개 통과·Native Skia·Rust lint 및 fresh WASM 실제 실행과
새 base 기능 보존 집중 대조 14쪽을 완료했습니다. [제품 검증](../assets/planet6897_20261103/intermediate-evidence-summary.md)과
[최신 리베이스 대조](../assets/planet6897_20261103/rebase_scope_22947f6a90/README.md)를 연결합니다.

## 현재 전쪽 Native/fresh WASM 증적

| 원본 · 독립 Print | Native TSV · 전쪽 최저 | fresh WASM TSV · 전쪽 최저 |
| --- | --- | --- |
| [cell_picture_row_split.hwp](../../../samples/issue8472/cell_picture_row_split.hwp) · [issue8472_cell_picture_row_split-2020.pdf](../../../pdf/issue8472_cell_picture_row_split-2020.pdf) | [TSV](../assets/planet6897_20261103/pr_8477/current-4b149b6cdf/pr8477-cell_picture_row_split-f5c3c377-native.tsv) · 100.00000% (2쪽) | [TSV](../assets/planet6897_20261103/pr_8477/current-4b149b6cdf/pr8477-cell_picture_row_split-f5c3c377-wasm.tsv) · 99.99000% (2쪽) |

96 DPI, 색상 경계 보정 전 2px 내용 실루엣입니다. 입력·Print PDF는 기존 동일 파일을 재사용하고 전체 쪽수와 누락을 확인했습니다.

| 주장 쪽·범위 | Native review / overlay | fresh WASM review / overlay |
| --- | --- | --- |
| cell_picture_row_split.hwp 2쪽 · 그림 칸 앵커 | [review](../assets/planet6897_20261103/pr_8477/current-4b149b6cdf/pr8477-claim-p002-cell_picture_row_s-p002-native-review.png) / [overlay](../assets/planet6897_20261103/pr_8477/current-4b149b6cdf/pr8477-claim-p002-cell_picture_row_s-p002-native-overlay.png) | [review](../assets/planet6897_20261103/pr_8477/current-4b149b6cdf/pr8477-claim-p002-cell_picture_row_s-p002-wasm-review.png) / [overlay](../assets/planet6897_20261103/pr_8477/current-4b149b6cdf/pr8477-claim-p002-cell_picture_row_s-p002-wasm-overlay.png) |
| cell_picture_row_split.hwp 1쪽 · 앞 차트 표 높이와 1쪽 끝 라벨 조각 | [review](../assets/planet6897_20261103/pr_8477/current-4b149b6cdf/pr8477-claim-p001-cell_picture_row_s-p001-native-review.png) / [overlay](../assets/planet6897_20261103/pr_8477/current-4b149b6cdf/pr8477-claim-p001-cell_picture_row_s-p001-native-overlay.png) | [review](../assets/planet6897_20261103/pr_8477/current-4b149b6cdf/pr8477-claim-p001-cell_picture_row_s-p001-wasm-review.png) / [overlay](../assets/planet6897_20261103/pr_8477/current-4b149b6cdf/pr8477-claim-p001-cell_picture_row_s-p001-wasm-overlay.png) |

## 통합 검증 완료와 병합 전 조건

정확한 code head의 모든 필수 로컬 실행과 GitHub CI가 성공했습니다. 아래 최신 통합 기록과 최종 code CI 증거를 확인했습니다. 이번 후속 문서 head의 실제 fast-pass·required check·mergeability를 별도로 확인한 뒤 승인된 병합을 진행합니다.

아래 선행 분석은 당시 코드·증적의 기록입니다. 당시 남은 검증 문장은 현재 상태가 아니며 위 현재 기록과 구분합니다.

<details>
<summary>선행 구현 분석·독립 근거·당시 실행 기록</summary>

## 접수·검토 코드

| 항목 | 값 |
| --- | --- |
| 원 PR | [PR #8477](https://github.com/edwardkim/rhwp/pull/8477) · `planet6897` |
| 원 head | `d53da96d75daf912d3e53d9137eadc7770732bbf` |
| 리베이스 후 원 PR 적용 | `c3320fc829, cb04b95295, 3df53c930f, cff3d2c355` |
| 추가 제품 보정 | `9c5720da81`, `40778b67d6` |
| 제품 소스 | `40778b67d6f69ccd3ee86cde84818dc2deea2cff` |
| 기준 devel | `1924d371d6757392d063b8dc2e9bbe9a298be817` |
| 관련 이슈 | [이슈 #8472](https://github.com/edwardkim/rhwp/issues/8472) |
| reviewer 상태 | `jangster77` 요청 · 2026-11-04 원 head 불변·Open·non-Draft 재확인 |
| 기본 경로 | `maintainer_general` |
| 보조 경로 | `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `multi_pr_update_branch`, `post_merge` |

## 검증 입력과 독립 Print

입력은 [cell_picture_row_split.hwp](../../../samples/issue8472/cell_picture_row_split.hwp), SHA-256 `d0b00c7c54154b5aed971ba28b5dbc3389b0f972c33bc2c5fb2a8c1b2b469c40`입니다. 기존 저장소의 동일 원본을 사용했습니다.

기준은 [issue8472_cell_picture_row_split-2020.pdf](../../../pdf/issue8472_cell_picture_row_split-2020.pdf), SHA-256 `34da9c3790e26a66f61b6f2fbadd438c8a97fea266536e91a6a46742be070848`입니다. 파일명과 달리 선행 작성자가 한컴 2022(12.0.0.4547) Print로 기록한 기존 두 쪽 PDF이며 이번에 재산출하지 않았습니다. 원 보고서 전체를 검증했다고 확대하지 않고 원 PR의 정식 재현 입력 두 쪽을 판정합니다.

## 전쪽 측정과 주장별 증적

2026-11-04, 96 DPI, Print profile, compat 2022, 색상 경계 보정 전 2px raw입니다. 전체 쪽수는 원본·PDF·두 출력 경로 모두 2쪽으로 일치하고 누락·90% 미만 쪽은 없습니다.

| 페이지·주장 | Native raw·review / overlay | fresh WASM raw·review / overlay |
| --- | --- | --- |
| 1쪽 · 온전한 앞 차트 표와 뒤 표 첫 조각·라벨 정렬 | 100.00% · [review](../assets/planet6897_20261103/pr_8477/pr8477-claim-p001-cell_picture_row_s-native-review.png) / [overlay](../assets/planet6897_20261103/pr_8477/pr8477-claim-p001-cell_picture_row_s-native-overlay.png) | 99.99% · [review](../assets/planet6897_20261103/pr_8477/pr8477-claim-p001-cell_picture_row_s-wasm-review.png) / [overlay](../assets/planet6897_20261103/pr_8477/pr8477-claim-p001-cell_picture_row_s-wasm-overlay.png) |
| 2쪽 · 그림·빈 앵커를 함께 이어받고 각 셀의 높이로 가운데 정렬 | 100.00% · [review](../assets/planet6897_20261103/pr_8477/pr8477-claim-p002-cell_picture_row_s-native-review.png) / [overlay](../assets/planet6897_20261103/pr_8477/pr8477-claim-p002-cell_picture_row_s-native-overlay.png) | 100.00% · [review](../assets/planet6897_20261103/pr_8477/pr8477-claim-p002-cell_picture_row_s-wasm-review.png) / [overlay](../assets/planet6897_20261103/pr_8477/pr8477-claim-p002-cell_picture_row_s-wasm-overlay.png) |

전쪽 원본 TSV: [Native](../assets/planet6897_20261103/pr_8477/pr8477-cell_picture_row_split-f5c3c377-native.tsv) · [fresh WASM](../assets/planet6897_20261103/pr_8477/pr8477-cell_picture_row_split-f5c3c377-wasm.tsv). 전쪽 최저는 각각 **100.00% / 99.99%**입니다.

실제 시각 판독에서는 첫 쪽 표의 높이·라벨 세 줄씩·앞 그림 두 개의 정렬, 두 번째 쪽의 그림 두 개와 표 내부 포함·라벨 중복 없음·그림 위변 차이를 확인했습니다. 2px raw 100%에서도 남았던 약 1px 위변 차이는 별도로 보정했습니다.

## 조판 원칙과 공통 소비 경로

[분석 문서](../../working/task_m100_8472_declared_opening_frame.md)에 원본 저장값·독립 PDF·선행 PR #8059와 생산·소비 지점을 기록했습니다.

- 공통 첫 조각 생산자 `declared_first_frame.rs`가 원본 선언·실제 컷·이월된 그림 앵커를 인증합니다. scan의 예약 높이와 부분 표 paint가 같은 물리 프레임을 소비합니다.
- 가운데 정렬은 공통 `CellCutFrame::content_height`의 안여백 제외 내용을 소비합니다. 온전한 그림 셀도 기존 저장 그림 프레임 생산자의 셀별 내용을 사용합니다.
- 이어받기는 그림과 빈 앵커를 이미 함께 소유한 유닛의 내용 높이를 재사용합니다. 빈 앵커 줄을 다시 더하거나 출력 좌표를 clamp하지 않습니다.
- 편집·재조판, 중첩 표, 원본 vpos 리셋, 독립 최소를 가진 행의 기존 계약은 유지합니다. PR #7518·#7870·#8060 집중 대조군을 함께 검사했습니다.

## 회귀 검사와 독립 좌표

[정식 회귀 파일](../../../tests/cases/issue_8472_cell_picture_anchor.rs)은 원본 선언과 남은 본문, 라벨 중심·내용 보존, 셀별 저장 앵커와 내용 높이의 상대 차이, 그림·앵커의 같은 조각 소유와 셀 내부 포함을 검사합니다.

양쪽 raw가 100.00%인 2쪽만 독립 PDF 이미지 사각형의 네 경계를 ±1 CSSpx로 추가 보호합니다. 관계 검사는 유지하며 특정 설치 글꼴·OTF를 요구하지 않습니다. 실제 최소 글꼴 노출 목록과 같은 좌표를 확인했습니다. 독립 PDF 좌표는 PDF point에 96/72를 곱한 CSS px이며, 아래처럼 제품 출력과 별도로 추출했습니다.

| 2쪽 그림 | 독립 PDF 경계 `(left, top, right, bottom)` | 최소 글꼴 출력 `(x, y, width, height)` |
| --- | --- | --- |
| 왼쪽 | `(84.11466471354167, 91.58003743489583, 387.7919921875, 245.970703125)` | `(84.3, 91.7, 303.9, 154.6)` |
| 오른쪽 | `(401.5439860026042, 90.46134440104167, 705.2213541666666, 247.40934244791666)` | `(401.8, 90.4, 303.9, 157.1)` |

수정 전 새 관계 검사는 실제 첫 프레임 60.26667px / 원본 선언 198.62667px와 이어받기 그림 상대차 0 / 원본 상대차 −1.27333px로 실패했습니다. 제품 보정 후 최소 글꼴 집중 40건이 모두 통과했습니다. 최종 좌표 검사를 포함한 이 파일 6건도 최소 글꼴 환경에서 모두 통과했습니다. 이는 최종 전체 회귀의 대체가 아닙니다.

## 빌드·측정의 출처

Native 바이너리 SHA-256은 `6d556ea3b231bd1976303978848045d05141b4d3d4404c689764ea72fcc69b7d`, fresh WASM은 `09c68917e431c2baf5ab7cf9612a6802d4557cb6229f23911e74aff0c1f9f6da`입니다. JS는 `bb9be9d806756697eddee5d185e80e9256bd0d97947d78a94777222100f12393`입니다.

Native는 보정 소스를 미커밋 상태로 빌드한 뒤 같은 내용으로 커밋했습니다. 빌드 때 기록한 1,727개 Rust 파일을 제품 커밋 `40778b67d6f69ccd3ee86cde84818dc2deea2cff`와 대조해 다른 파일이 0개임을 확인했습니다. Native 측정 manifest가 기록한 호출 worktree SHA는 `9c5720da819ef01deaa69e3e9d43b6728c7d1d80`이며, 이를 제품 소스 SHA로 취급하지 않습니다. WASM 측정은 제품 커밋 SHA를 기록했습니다. 실제 WASM의 새 base paint-font API와 글상자 입력의 열기 성공도 확인했습니다.

PDF 래스터는 pypdfium2 5.13.0 / PDFium 153.0.7999.0의 `pdfium_smooth_rgb_exact_dpi_v2` 정책을 사용했습니다. Native는 Skia, WASM은 Canvas2D입니다. 설치된 시스템 글꼴 전체를 시각 측정에 제공했고, 이 입력의 Native 자동 웹폰트 추가 공급 목록은 비어 있습니다. 설치 글꼴은 측정에만 사용하고 글꼴 파일은 커밋하지 않습니다. 원시 manifest·분석 JSON·로그는 ignored `output/pr-review/planet6897-20261103/`에만 보관하고, 최종 입력·PDF·PNG·TSV와 이 문서의 출처·해시 기록을 영구 증거로 남깁니다.

재현은 저장소 루트에서 `venv/bin/python scripts/visual_sweep.py --silhouette-only --resume --hwp samples/issue8472/cell_picture_row_split.hwp --pdf pdf/issue8472_cell_picture_row_split-2020.pdf --key pr8477-owned-frame --rhwp-bin <검증한 Native 바이너리> --compat 2022 --dpi 96 --page-workers 1 --out <검증 폴더>`로 수행합니다. WASM은 같은 명령에 `--wasm-pkg <검증한 fresh pkg>`를 추가합니다. 입력·producer·환경·산출물 해시가 같고 완료된 쪽만 재사용합니다.

## 통합 전 남은 조건

현재 통합의 필수 실행 41개·편람 Native 383쪽 gate·PR #8389·#8397 원 TSV 표시값 무하락은 완료했습니다. 새 base Native/fresh WASM 대조도 모두 통과했습니다. GitHub CI·원격 게시·통합 승인 조건은 별도이며 준비 완료 후 승인된 PR 생성·CI 모니터링을 진행합니다.

## Merge 후 contributor PR comment 계획

통합 완료가 승인되고 실제 merge SHA가 확정되면 위 동일 입력의 1·2쪽 Native/fresh WASM review·overlay와 전체 TSV를 첫 comment부터 그 merge SHA의 raw URL로 표시합니다. 다른 PR의 대표 이미지로 대체하지 않습니다. 게시 전에 이미지 응답·형식·해시를 위 자산과 대조하고 게시 후 API 본문과 화면을 확인합니다. 원 PR 닫기는 통합 SHA의 devel 포함과 모든 후속 조건을 확인한 뒤 승인 범위에 따라 처리합니다.

## 최종 TSV·대표 PNG의 신원

| 파일 | 바이트 수 | SHA-256 |
| --- | ---: | --- |
| [pr8477-cell_picture_row_split-f5c3c377-native.tsv](../assets/planet6897_20261103/pr_8477/pr8477-cell_picture_row_split-f5c3c377-native.tsv) | 152 | `75a9bab7c2e3c98ae402a8caf9e39fecf6b6081c5a8aa222f99b01cf9da24690` |
| [pr8477-cell_picture_row_split-f5c3c377-wasm.tsv](../assets/planet6897_20261103/pr_8477/pr8477-cell_picture_row_split-f5c3c377-wasm.tsv) | 150 | `2d96a69585f096e23614ca2a655493c63d9587b4bc0daa604409a6609f8ddfa7` |
| [pr8477-claim-p001-cell_picture_row_s-native-overlay.png](../assets/planet6897_20261103/pr_8477/pr8477-claim-p001-cell_picture_row_s-native-overlay.png) | 204369 | `34076dab5632db68bde0ee390ecfc93379a99f453379d1418931ee5729820431` |
| [pr8477-claim-p001-cell_picture_row_s-native-review.png](../assets/planet6897_20261103/pr_8477/pr8477-claim-p001-cell_picture_row_s-native-review.png) | 631692 | `63056ae17360cb61ea705e883141f0f9a7562594728ebb206b2900ab85a76234` |
| [pr8477-claim-p001-cell_picture_row_s-wasm-overlay.png](../assets/planet6897_20261103/pr_8477/pr8477-claim-p001-cell_picture_row_s-wasm-overlay.png) | 207785 | `4cc7eef2a1774f808e7a0d080121c2aa40c92c9256d768a2f6735e56b41b0a57` |
| [pr8477-claim-p001-cell_picture_row_s-wasm-review.png](../assets/planet6897_20261103/pr_8477/pr8477-claim-p001-cell_picture_row_s-wasm-review.png) | 626596 | `c0219573819d291de02d333e60f90888654aedf985600b36d10fabdd6793299a` |
| [pr8477-claim-p002-cell_picture_row_s-native-overlay.png](../assets/planet6897_20261103/pr_8477/pr8477-claim-p002-cell_picture_row_s-native-overlay.png) | 66583 | `54ecb3e611a826217f3bf562d2060e80630ca2c9c6ee60b29b7365696dd992af` |
| [pr8477-claim-p002-cell_picture_row_s-native-review.png](../assets/planet6897_20261103/pr_8477/pr8477-claim-p002-cell_picture_row_s-native-review.png) | 135971 | `e965a30515fd3847da56a1e5f72c062b69b967b70ae1420971b2b2846076a587` |
| [pr8477-claim-p002-cell_picture_row_s-wasm-overlay.png](../assets/planet6897_20261103/pr_8477/pr8477-claim-p002-cell_picture_row_s-wasm-overlay.png) | 66397 | `de9012e9f89100d7212888398345a70a24dbd508a0bae8fda16403d3e6073fca` |
| [pr8477-claim-p002-cell_picture_row_s-wasm-review.png](../assets/planet6897_20261103/pr_8477/pr8477-claim-p002-cell_picture_row_s-wasm-review.png) | 135800 | `e360e513114134fdeeec585b8e4926b227924eb5413a78a92eb6565f11cebd97` |

</details>

## 최신 통합 검증 상태

현재 base는 `d4a72efd7c91d1e25092ed30b9d44e43cbebb9f0`입니다. 직전 문서 갱신 뒤 PR #8618의 이슈 라벨링 workflow·계약 검사·운영 문서 3개가 추가됐습니다.
184개 패치가 충돌 0건·range-diff 모두 `=`로 리베이스됐습니다. 제품·테스트·fixture·검증 도구의 내용은
리베이스 전과 같으므로 완료 증거를 재사용합니다. 실제 측정 source는
`ba2ebfd34a4ebf402bce8e002254b672a09ef764`이며 문서 commit을 새 렌더 제품으로 표시하지 않습니다.
[현재 적용 SHA 39개](../assets/planet6897_20261103/integration-commits.md)를 확인합니다.

- [최소 글꼴 환경의 필수 실행 41개](../assets/planet6897_20261103/rebase8609/rust-regression-validation.md)는 모두 exit 0입니다.
  전체 nextest 12,583 PASS·45 SKIP, Native Skia lib 4,040 PASS·12 IGNORE,
  CI Native 모듈 21개와 doctest 8 PASS·3 IGNORE를 포함합니다.
- [최신 Native 영향 10입력·43쪽](../assets/planet6897_20261103/rebase8609/native-own-validation.md)과
  [fresh WASM 같은 43쪽](../assets/planet6897_20261103/rebase8609/wasm-own-validation.md)은 전쪽 90% 이상,
  직전 후보 대비 정밀 raw 0.1%p 초과 하락·누락·쪽수 불일치 0건입니다.
- [최신 필수 편람 Native 383쪽](../assets/planet6897_20261103/rebase8609/handbook-native-validation.md)은
  최저 표시 raw 99.96%, 기준 표시값 하락·fallback 0건·exit 0입니다.
- [최신 실제 API·브라우저](../assets/planet6897_20261103/rebase8609/runtime-validation.md)는 실제
  271 assertion·17 Worker·12쪽 렌더/저장과 가져오기·편집 산출물 대조를 완료했습니다.
- 기존 c183의 [Native 80입력·406쪽](../assets/planet6897_20261103/current-c183a1ac61/native-validation.md)과
  [fresh WASM 전406쪽](../assets/planet6897_20261103/current-c183a1ac61/wasm-validation.md)은 양쪽 90% 이상입니다.
  충돌 없는 리베이스 뒤 전쪽 측정을 반복하지 않는 지시에 따라 원래 producer를 명시해 재사용합니다.
- 보호 PR #8389·#8397의 [c183 Native](../assets/planet6897_20261103/current-c183a1ac61/pr8389_8397/native-validation.md)와
  [c183 fresh WASM](../assets/planet6897_20261103/current-c183a1ac61/pr8389_8397/wasm-validation.md)은
  각 17문서·1,770쪽·원 TSV 2,153행에서 표시값 하락·누락 0행입니다.

원 PR별 주장에 대응하는 입력·쪽의 대표 review/overlay는 아래 원래 출처를 유지합니다.
최신 영향 입력의 그림은 최신 source로 별도 표시하며, 다른 문서의 대표 그림으로 대체하지 않습니다.
[직접 판독 원장](../assets/planet6897_20261103/direct-reading-current.md)과
[입력/PDF·compat·글꼴·명령](../assets/planet6897_20261103/visual-reproduction.md)을 연결합니다.
회귀 측정 주석 갱신은 실제 assertion·helper 본문과 구분했고 주석만 바뀐 전체 검사는 반복하지 않았습니다.

[새 base Native 대조](../assets/planet6897_20261103/rebase8609/native-base-preservation.md)와
[fresh WASM 대조](../assets/planet6897_20261103/rebase8609/wasm-base-preservation.md)는 각각 14입력·41쪽을 완료했습니다.
양쪽 모두 raw 하락 자체가 0쪽이며 90% 미만·누락·쪽수 차이·제품 해시 오류 0건입니다.
문서-only 리베이스의 [내용 동일성](../assets/planet6897_20261103/latest-rebase.md)을 확인해 완료 결과를 재사용했습니다.
추가 전체 회귀는 필요하지 않습니다. 제출 전 필수 검증은 완료했고 통합 PR #8622의 정확한 code head CI가 모두 성공했으며 이번 trailing head는 별도로 확인합니다.
원 PR 39개는 원격 재조회에서 head 불변·Open·non-Draft·reviewer 요청을 확인했습니다.
통합 PR 생성 전 본인 계정의 우선 처리할 열린 PR은 0건이었습니다. 원격 push·통합 PR 생성과 code head CI는 완료했습니다. 이 후속 문서 작성 시점에는 병합·원 PR 댓글·닫기를 아직 수행하지 않았습니다.
작업지시자는 준비 완료 후 PR 생성·CI 모니터링을 승인했습니다. 문서-only trailing commit의 병합·링크·기존 오늘할일 보존을 확인한 뒤 push하고 새 head CI를 확인합니다.

## Merge 후 contributor PR comment 계획

[통합 PR #8622](https://github.com/edwardkim/rhwp/pull/8622)의 실제 merge SHA와 devel 반영을 확인한 뒤 원 PR을 통합 완료로 정리합니다.
[이슈별 처리 범위](../assets/planet6897_20261103/source-followup-scope.md)에 따라 해결 이슈와 남은 별도 범위를 구분합니다.
[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결하고, 위 전쪽 TSV와 같은 원본·PDF·주장 쪽의 review 및 별도 overlay를 사용합니다.
실제 그림은 직접 판독 원장의 producer·페이지·해시를 유지하며 다른 문서의 대표 그림으로 대신하지 않습니다.
원 PR의 기여와 메인터너 보정 이유·검증 결과·남은 차이를 한국어 존댓말로 설명합니다.
이미지 URL은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/...` 형식으로 고정하고 게시 전 응답의 이미지 형식·SHA-256을 원본과 대조합니다.
기존 같은 검토 댓글이 있으면 수정하고 없을 때만 처음 게시합니다. 게시 뒤 API 본문과 이미지 표시를 재확인하며 contributor fork branch는 삭제하지 않습니다.

[최종 code CI와 오탐 처리](../assets/planet6897_20261103/code-head-ci-completion.md)를 함께 기록했습니다. 현재 기록 시점에는 merge·원 PR close/comment를 아직 실행하지 않았으며, 실제 원격 완료 사실은 원 PR 타임라인과 통합 PR에 남깁니다.

[후속 댓글의 실제 원본·쪽·Native/WASM PNG 대응표](../assets/planet6897_20261103/source-comment-evidence.md#pr-8477)를 고정했습니다. 이미지 수·출력 경로·해시를 게시할 댓글과 대조합니다.
