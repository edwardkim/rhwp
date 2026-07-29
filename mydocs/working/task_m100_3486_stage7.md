---
kind: investigation
status: active
canonical: mydocs/manual/codex/docs_and_git_workflow.md
last_verified: 2026-07-29
---

# Task #3486 Stage 7 — 암호 HWP3 전체 24쪽 PDF 1:1 기준선과 결함 분리

- 이슈: [#3486](https://github.com/edwardkim/rhwp/issues/3486)
- 브랜치: `task_m100_3486_hwp3_render_fidelity_v2`
- 선행: `77a14af73` (Stage 6: HWP3 음수 들여쓰기의 첫 줄 기준 정규화)
- 입력: `samples/HWP3-password-123456.hwp` (암호 HWP3, 24쪽)
- 기준 오라클: `pdf/HWP3-password-123456.pdf` (한컴 PDF, 24쪽)

## 비교 방법과 범위

2026-07-29에 암호 HWP3 원본을 재직렬화하지 않고, 일회성 `--password-stdin` wrapper로
`scripts/task1274_visual_sweep.py`에 직접 전달했다. PDF와 rhwp SVG raster를 같은 144 DPI,
pixel diff threshold 32로 24쪽 모두 1:1 대응했다. 기준 PDF의 legacy text layer 때문에
`pdftotext -bbox-layout`는 abort했지만, SVG export·PDF raster·compare·overlay·review 생성은
24쪽 모두 완료됐다. 따라서 이 단계의 판정 근거는 raster와 육안 대조이며 PDF text bbox 결과가 아니다.

전체 자료:

- [review contact sheet](/Users/tsjang/rhwp/tmp/pdfs/task3486/stage7-all-pages.PMNvQm/hwp3-password-123456-stage7/review_contact_sheet.png)
- [overlay contact sheet](/Users/tsjang/rhwp/tmp/pdfs/task3486/stage7-all-pages.PMNvQm/hwp3-password-123456-stage7/overlay_contact_sheet.png)
- [page metrics](/Users/tsjang/rhwp/tmp/pdfs/task3486/stage7-all-pages.PMNvQm/hwp3-password-123456-stage7/overlay/overlay_metrics.json)
- [structural analysis](/Users/tsjang/rhwp/tmp/pdfs/task3486/stage7-all-pages.PMNvQm/hwp3-password-123456-stage7/analysis/metrics.json)

`pixel_match_percent`는 넓은 백지 영역 때문에 실제 품질보다 높게 나올 수 있다. 아래의
`proxy`는 잉크 영역 일치율(`visual_accuracy_proxy_percent`)이며 자동 비교 우선순위용이다.
사람의 한컴 PDF 대조 판정이나 합격률로 사용하지 않는다.

| 쪽 | 전체 픽셀 일치율 | 내용 픽셀 proxy | 육안 1:1 판정 요약 |
| ---: | ---: | ---: | --- |
| 1 | 88.24856% | 28.34432% | 제목 조합 자모, 본문 폭, 삽화·TOC 배치 불일치 |
| 2 | 94.01220% | 35.80916% | 본문·여백 계열 차이 지속 |
| 3 | 93.44988% | 6.82176% | 표 글꼴/내용 및 하단 흐름 불일치; bottom drift 후보 |
| 4 | 92.17249% | 8.44613% | 본문 폭·행간 차이 |
| 5 | 91.46643% | 7.49087% | 본문·목록 조판 차이 |
| 6 | 92.40040% | 9.96615% | 본문 폭·행간 차이 |
| 7 | 94.15494% | 8.14705% | 본문·목록 조판 차이 |
| 8 | 91.67933% | 7.46612% | 본문 폭·행간 차이 |
| 9 | 93.89827% | 8.96045% | 본문·목록 조판 차이 |
| 10 | 92.39202% | 6.50127% | 목록·`한글` legacy 글리프와 행 폭 불일치 |
| 11 | 92.46506% | 10.10838% | 본문 폭·행간 차이 |
| 12 | 90.76860% | 7.76722% | 본문·목록 조판 차이 |
| 13 | 91.85025% | 10.69106% | 본문 폭·행간 차이 |
| 14 | 92.39137% | 11.27997% | 본문 폭·행간 차이 |
| 15 | 92.30382% | 11.41985% | 본문 폭·행간 차이 |
| 16 | 92.14252% | 11.10503% | 본문 폭·행간 차이 |
| 17 | 92.41441% | 12.52250% | 본문 폭·행간 차이 |
| 18 | 95.79696% | 7.06868% | 전 페이지 잔여 문단/목록 흐름이 PDF와 다름 |
| 19 | 94.27785% | 8.61779% | 본문 폭·행간 차이 |
| 20 | 91.43617% | 8.28559% | 본문·목록 조판 차이 |
| 21 | 92.21866% | 8.26985% | 본문·목록 조판 차이 |
| 22 | 92.94142% | 11.23025% | 본문 폭·행간 차이 |
| 23 | 94.66336% | 11.14413% | 본문 폭·행간 차이 |
| 24 | 95.29642% | 8.83464% | 본문·글머리표·하이퍼링크 조판 차이 |

24쪽 평균은 전체 픽셀 92.70172%, 내용 픽셀 proxy 11.09576%이고, 최저 proxy는 10쪽의
6.50127%다. 자동 structural heuristic은 3쪽의 content-bottom drift만 표시했지만, 이는
semantic/font 계열 차이를 포착하지 못하는 한계가 있다. 특히 18쪽은 배경 비율로 전체 픽셀 수치는
95.79696%지만 본문 흐름은 PDF와 명확히 다르므로, 전체 픽셀 일치율을 수용 근거로 삼으면 안 된다.

## 대표 페이지의 실제 증적

### page 1

- compare: `/Users/tsjang/rhwp/tmp/pdfs/task3486/stage7-all-pages.PMNvQm/hwp3-password-123456-stage7/compare/compare_001.png`
- overlay: `/Users/tsjang/rhwp/tmp/pdfs/task3486/stage7-all-pages.PMNvQm/hwp3-password-123456-stage7/overlay/overlay_001.png`
- review: `/Users/tsjang/rhwp/tmp/pdfs/task3486/stage7-all-pages.PMNvQm/hwp3-password-123456-stage7/review/review_001.png`
- visual_accuracy_proxy_percent: 28.34432

코멘트: 내용 픽셀 중심 자동 일치율 보조값 = 약 28.34%.
높을수록 좋음: 기준 PDF와 rhwp PNG가 더 비슷함
낮을수록 나쁨/검토 필요: 잉크 위치나 형태 차이가 큼
단, 사람 판정 정확도가 아니라 내용 픽셀 중심 자동 일치율 보조값입니다

PDF의 `한글 97 안내문`은 현대 한글로 보이지만 rhwp HWP3에는 `ᄒᆞᆫ` 조합 자모 계열이
분리/오조판된다. 본문 줄 폭, 삽화와 TOC의 크기·위치도 다르다.

### page 3

- compare: `/Users/tsjang/rhwp/tmp/pdfs/task3486/stage7-all-pages.PMNvQm/hwp3-password-123456-stage7/compare/compare_003.png`
- overlay: `/Users/tsjang/rhwp/tmp/pdfs/task3486/stage7-all-pages.PMNvQm/hwp3-password-123456-stage7/overlay/overlay_003.png`
- review: `/Users/tsjang/rhwp/tmp/pdfs/task3486/stage7-all-pages.PMNvQm/hwp3-password-123456-stage7/review/review_003.png`
- visual_accuracy_proxy_percent: 6.82176

코멘트: 내용 픽셀 중심 자동 일치율 보조값 = 약 6.82%.
높을수록 좋음: 기준 PDF와 rhwp PNG가 더 비슷함
낮을수록 나쁨/검토 필요: 잉크 위치나 형태 차이가 큼
단, 사람 판정 정확도가 아니라 내용 픽셀 중심 자동 일치율 보조값입니다

Stage 4의 셀 채움과 Stage 6의 표 뒤 첫 줄 x 좌표 보정은 반영됐지만, 표 내 legacy text와
아래 설명의 전체 흐름은 PDF와 아직 다르다. 구조 heuristic도 이 쪽의 하단 -39px drift를 보고했다.

### page 10

- compare: `/Users/tsjang/rhwp/tmp/pdfs/task3486/stage7-all-pages.PMNvQm/hwp3-password-123456-stage7/compare/compare_010.png`
- overlay: `/Users/tsjang/rhwp/tmp/pdfs/task3486/stage7-all-pages.PMNvQm/hwp3-password-123456-stage7/overlay/overlay_010.png`
- review: `/Users/tsjang/rhwp/tmp/pdfs/task3486/stage7-all-pages.PMNvQm/hwp3-password-123456-stage7/review/review_010.png`
- visual_accuracy_proxy_percent: 6.50127

코멘트: 내용 픽셀 중심 자동 일치율 보조값 = 약 6.50%.
높을수록 좋음: 기준 PDF와 rhwp PNG가 더 비슷함
낮을수록 나쁨/검토 필요: 잉크 위치나 형태 차이가 큼
단, 사람 판정 정확도가 아니라 내용 픽셀 중심 자동 일치율 보조값입니다

목록 기호와 legacy 한글 glyph, 본문 행 폭·기준선 차이가 함께 보인다. 현재 HWP3 PUA 문자 몇 개를
문서별로 치환하는 방식으로 이 쪽을 고치지 않는다.

### page 18

- compare: `/Users/tsjang/rhwp/tmp/pdfs/task3486/stage7-all-pages.PMNvQm/hwp3-password-123456-stage7/compare/compare_018.png`
- overlay: `/Users/tsjang/rhwp/tmp/pdfs/task3486/stage7-all-pages.PMNvQm/hwp3-password-123456-stage7/overlay/overlay_018.png`
- review: `/Users/tsjang/rhwp/tmp/pdfs/task3486/stage7-all-pages.PMNvQm/hwp3-password-123456-stage7/review/review_018.png`
- visual_accuracy_proxy_percent: 7.06868

코멘트: 내용 픽셀 중심 자동 일치율 보조값 = 약 7.07%.
높을수록 좋음: 기준 PDF와 rhwp PNG가 더 비슷함
낮을수록 나쁨/검토 필요: 잉크 위치나 형태 차이가 큼
단, 사람 판정 정확도가 아니라 내용 픽셀 중심 자동 일치율 보조값입니다

rhwp가 페이지 상단에 놓은 잔여 문단·목록 흐름과 PDF의 내용이 다르다. 배경 때문에 픽셀 일치율은
높지만 실제 1:1 정합은 아니다.

### page 24

- compare: `/Users/tsjang/rhwp/tmp/pdfs/task3486/stage7-all-pages.PMNvQm/hwp3-password-123456-stage7/compare/compare_024.png`
- overlay: `/Users/tsjang/rhwp/tmp/pdfs/task3486/stage7-all-pages.PMNvQm/hwp3-password-123456-stage7/overlay/overlay_024.png`
- review: `/Users/tsjang/rhwp/tmp/pdfs/task3486/stage7-all-pages.PMNvQm/hwp3-password-123456-stage7/review/review_024.png`
- visual_accuracy_proxy_percent: 8.83464

코멘트: 내용 픽셀 중심 자동 일치율 보조값 = 약 8.83%.
높을수록 좋음: 기준 PDF와 rhwp PNG가 더 비슷함
낮을수록 나쁨/검토 필요: 잉크 위치나 형태 차이가 큼
단, 사람 판정 정확도가 아니라 내용 픽셀 중심 자동 일치율 보조값입니다

본문 길이·글머리표·하이퍼링크 조판 차이가 마지막 쪽까지 이어진다. 따라서 Stage 6 보정만으로는
전체 문서 수용 기준을 충족하지 못한다.

## 원인 가설의 분리와 다음 계약

이번 전수 대조는 다음 세 계열을 분리한다. 하나의 문서 전용 PUA 치환이나 전역 font-size 조정으로
동시에 처리하지 않는다.

1. **legacy/옛한글 display** — HWP3 parser가 `ᄒᆞᆫ`으로 보존한 조합 자모가 PDF의 `한`과 다르게
   조판된다. HWPX에서도 같은 논리 문구가 보이므로 HWP3 decoder만의 문제가 아닐 수 있다. 다음
   구현 전에는 Studio CanvasKit의 old-Hangul typeface 선택과 SVG fallback의 실제 glyph/advance를
   동일 input에서 분리 확인한다.
2. **HWP3 legacy glyph·표·목록 변환** — 3·10·24쪽의 표 text, PUA/글머리표, cell 내용은 조합
   자모와 독립된 차이다. HWP3 원시 code point를 대응 HWP5/HWPX 변환본의 IR과 문단 단위로 대조해
   일반 규칙을 확보한 뒤에만 mapping을 추가한다.
3. **본문 font metric·page flow** — 18쪽을 포함한 전 페이지의 line width/vertical flow 차이는
   문자 mapping 하나로 설명할 수 없다. 다음 비교는 같은 논리 문서의 비암호 HWP5를 동일 PDF에
   sweep하여, HWP3 parser 고유 page partition/line-info 문제인지 공통 renderer font metric 문제인지
   먼저 수치와 render tree로 판별한다.

## HWP5 대조로 확정한 18쪽 line-segment 결함

비암호 `samples/HWP5-nopassword-123456.hwp`도 같은 PDF에 24쪽 전수 sweep했다. 이 결과의 평균
내용 픽셀 proxy는 12.86633%이며 HWP3의 11.09576%보다 소폭 높다. 특히 18쪽은 HWP5 10.70215%,
HWP3 7.06868%로, HWP5 render tree가 PDF와 동일하게 `다운로드 메뉴에서…Active X control`부터
시작한다. HWP3는 바로 앞 17쪽에 그 줄을 잘못 남기고 18쪽을 `을, 넷스케이프…`부터 시작한다.

HWP3 원문 문단 258의 IR은 다음 저장 좌표를 보존한다.

| line | `text_start` | `vertical_pos` | PDF/HWP5와의 의미 |
| ---: | ---: | ---: | --- |
| 0 | 0 | 63488 | 17쪽 마지막 `웹에서 … 홈페이지(…)의` |
| 1 | 57 | 0 | **18쪽 첫 줄** `다운로드 메뉴에서…Active X control` |
| 2 | 112 | 1600 | 18쪽 이어지는 `을, 넷스케이프…` |

`dump-pages`는 이 지점을 `[vpos-reset@line1]`로 표시하지만, 기본과
`--respect-vpos-reset` 양쪽이 같은 `PartialParagraph pi=258 lines=0..2`를 17쪽에 배치했다.
처음에는 `HwpDocument::set_respect_vpos_reset()`의 wrapper→core 전달 누락을 가설로 세웠다.
그러나 실제 fixture focused test가 이를 반증했다. `HwpDocument`는 `DocumentCore`를 `Deref`하므로
setter의 `self.respect_vpos_reset`은 이미 `PaginationOpts`가 읽는 core 필드를 바꾸며, 중복 대입을
추가해도 page item은 변하지 않았다. 해당 임시 코드·회귀는 즉시 제거했다.

`RHWP_USE_PAGINATOR=1` probe는 이 reset을 **논리적으로는** 올바르게 해석했다. 즉 17쪽은
`pi=258 lines=0..1`, 18쪽은 `lines=1..3`이 되었다. 하지만 이를 기본 엔진으로 전환한 24쪽 sweep은
내용 픽셀 proxy가 11.09576%에서 9.56949%로 내려갔고, structural flag가 1쪽에서 12쪽으로 늘었으며
frame overflow 후보도 0쪽에서 7쪽으로 증가했다. 18쪽 raster도 첫 두 줄을 제대로 paint하지 못했다.
따라서 `PaginationEngine`으로 기본 경로를 바꾸는 해법은 배제한다.

기본 `TypesetEngine`에도 같은 문단 내 reset 감지 함수가 이미 있다. 다만 HWP3 native 경로의 조건이
`para.controls.is_empty()`여서 문단 258의 인라인 `Control::Hyperlink` 하나를 flow-affecting 개체처럼
취급하고 감지를 중단했다. 실제 원문은 표·그림·각주가 아니라 본문 중간의 hyperlink 마커뿐이다.
즉 이번 수정 후보는 **원본 HWP3 + 보이는 텍스트 + hyperlink만 포함한 문단**에 한정하여 internal
vpos rewind를 인식시키는 것이다. 표·그림·각주·수식·form·알 수 없는 컨트롤은 계속 제외한다.

다음 구현 계약은 다음과 같다.

1. 조건을 일반 `controls.is_empty()`에서 hyperlink-only 허용으로만 넓힌다.
2. 암호 HWP3 fixture의 258번 문단은 default `TypesetEngine`에서 17쪽 `0..1`, 18쪽 `1..3`으로
   배치되어야 한다.
3. 구현 전용 unit 회귀와 실제 fixture 회귀를 추가하고, 수정 후 24쪽 전체 PDF sweep을 다시 실행한다.
4. 이 Stage 문서·코드·회귀·재대조 결과를 **같은 커밋**으로 남긴다. 이 문서만 독립 커밋하지 않는다.

## 구현과 focused 회귀

`src/renderer/typeset.rs`의 `internal_vpos_page_break_line()`은 원본 HWP3에만 적용되는
되감김 경로다. 이번 변경은 `hwp3_text_rewind_controls_are_inline_hyperlinks()`를 두어
`Control::Hyperlink`만 인라인 메타데이터로 허용했다. 기존에 `controls.is_empty()`가 보호하던
표·그림·각주·미주·수식·form·field·ruby·알 수 없는 컨트롤은 전부 계속 차단한다.

`tests/hwp3_password_fixture.rs`에는 실제 암호 HWP3 fixture로 다음을 고정했다.

- 17쪽(global index 16): `PartialParagraph pi=258 lines=0..1`
- 18쪽(global index 17): `PartialParagraph pi=258 lines=1..3`

검증 명령과 결과:

```bash
CARGO_TARGET_DIR=target/task_3486_render_v2 CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --test hwp3_password_fixture
# 6 passed; 0 failed
```

따라서 `--respect-vpos-reset` 또는 대체 페이지네이터를 기본으로 전환하지 않고도, 기본
`TypesetEngine`이 이 HWP3 저장 계약을 직접 따른다.

## 구현 후 24쪽 PDF 1:1 재대조

동일 입력과 PDF를 144 DPI / threshold 32로 다시 전수 대조했다. 아래의 stable asset은 링크만
남긴 것이 아니라 실제 PNG를 저장소에 포함한다.

- [24쪽 review contact sheet](../report/assets/task_m100_3486/stage7/hwp3_password_pdf_all_pages_review.png)
- [24쪽 overlay contact sheet](../report/assets/task_m100_3486/stage7/hwp3_password_pdf_all_pages_overlay.png)
- [17쪽 review](../report/assets/task_m100_3486/stage7/hwp3_password_pdf_p017_review.png)
- [18쪽 review](../report/assets/task_m100_3486/stage7/hwp3_password_pdf_p018_review.png)
- 전체 원시 산출물: `/tmp/rhwp-stage7-hyperlink-sweep.Bi3Wrc/hwp3-password-123456-stage7-hyperlink-reset/`

| 쪽 | 변경 전 proxy | 변경 후 proxy | 판단 |
| ---: | ---: | ---: | --- |
| 1 | 28.34432% | 16.04026% | 병행 중인 renderer 변경으로 달라짐; 이번 코드와 분리 |
| 17 | 12.52250% | 12.70514% | reset 직전 한 줄만 남겨 PDF 흐름으로 보정 |
| 18 | 7.06868% | 10.95853% | PDF와 같이 `다운로드 메뉴…Active X control`부터 시작 |
| 2–16, 19–24 | 동일 | 동일 | 21쪽 raster byte-identical |

18쪽 구조 분석은 `content_bottom_delta_px=-2.0`, line-band 평균 절대 차이 1.6px,
structural flag 없음이었다. compare/review에서 PDF의 16개 line band와 rhwp 16개가 대응하는 것을
확인했다. 이 범위에서는 수용한다.

이번 전수 재실행에서 1쪽도 달라져 전체 proxy 평균은 11.09576%에서 10.75278%로 변했다. 그러나
이번 변경이 page item을 바꾼 것은 17·18쪽뿐이고, 2–16 및 19–24쪽(21쪽)은 직전 sweep의 rhwp PNG와
byte-identical이었다. 1쪽의 변화는 동시에 워킹트리에 있던 `src/parser/hwp3/mod.rs`, SVG/Skia/
Canvas 렌더러 변경의 영향이며 이 Stage의 page-boundary 보정 결과로 귀속하지 않는다. 그 변경을
되돌리거나 이 커밋에 섞지 않는다.

전체 24쪽은 아직 1:1 수용 상태가 아니다. 1쪽의 title·본문/TOC geometry, 3쪽 표, 10쪽 legacy
glyph, 24쪽 목록·하이퍼링크가 남아 있다. 이번 커밋의 완료 범위는 암호 HWP3의 hyperlink 포함
internal vpos reset으로 인한 17/18쪽 content loss 복구다. 다음 Stage는 이 커밋 뒤 새 분석 문서에서
font/glyph 및 표·목록 차이를 별도 원인으로 다룬다.
