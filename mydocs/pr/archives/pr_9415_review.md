# PR #9415 검토 — KAF 부록 재저장 글상자와 크기 보호

## 최종 판정

**승인.** #9395의 한컴 재저장 글상자는 현재 요소 높이까지 인쇄하고, 크기 보호가 설정된
글상자는 선언 높이와 저장 줄 배치를 유지한다. 한컴 독립 출력으로 두 조건을 모두 확인했다.
문서 기록을 추가한 최종 head의 필수 CI와 fast-pass 확인 뒤 병합한다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#9415](https://github.com/edwardkim/rhwp/pull/9415) · jangster77 · devel |
| 코드 검증 head | `ddcc33907dac7d06766673db2ccf8edbce4cbc63` |
| 고정 devel | `0521a1df673e7f64d8bf7ef785a6a60001d3b9c3` |
| 관련 이슈 | [#9395](https://github.com/edwardkim/rhwp/issues/9395), 병합 시 닫기 |
| 검토 방식 | 공동 작업자 self-review |

## 저장 글상자와 인쇄 경계

원본 KAF 부록의 36쪽은 앞선 [통합 PR #9411](pr_9411_review.md)에서 고쳤다. 한컴으로 HWP를
다시 저장하면 선언 높이 7,855 HU와 별도로 현재 요소 높이 8,743 HU와 종축 배율이
저장된다. 위쪽 정렬 사각형에서 현재 요소 높이가 저장 글줄 및 안 여백과 일치할 때
인쇄 경계에 반영한다. 해당 글상자의 경계가 104.73px에서 116.57px로 늘어 셋째 줄과
아래 안 여백을 포함한다. 원본의 54쪽 출력은 Native와 WASM 각각 보정 전과 같았다.

크기 보호 상태에서는 한컴 재저장 후 선언 높이와 현재 요소 높이가 모두 7,855 HU이고
종축 배율은 1.0이었다. `such as`가 둘째 줄, `and Cadastre`가 셋째 줄에 배치되며
고정 크기에 따른 잘림도 한컴 인쇄본과 일치한다. 회귀 검사는 글줄 소속과 크기 보호
경계의 관계를 확인한다. [회귀 테스트](../../../tests/cases/issue_9379_kaf_appendix_textbox_wmf.rs)

## Visual Sweep 증적

54쪽짜리 [원본 HWP](../../../samples/issue9379/kaf_appendix.hwp), 한컴 HWP 재저장본,
크기 보호 대조본을 데스크톱 한컴 2020 `11.0.0.9172`의 Hancom PDF 한 쪽씩 인쇄 출력을
기준으로 비교했다. PDFium 96 DPI 및 compat 2022에서 raw 실루엣 2px 이웃 지표를 적용했다.
[입력·Print PDF 경로와 해시, MCP 영수증, 전쪽 TSV](../assets/issue_9395/verification.json)

| 36쪽 | Native | fresh WASM | Native 그림 | WASM 그림 |
| --- | ---: | ---: | --- | --- |
| 한컴 재저장본 | 95.39% → 100.00% | 100.00% | [review](../assets/issue_9395/resaved/native/review_036.png) · [overlay](../assets/issue_9395/resaved/native/overlay_036.png) | [review](../assets/issue_9395/resaved/wasm/review_036.png) · [overlay](../assets/issue_9395/resaved/wasm/overlay_036.png) |
| 크기 보호 대조본 | 94.89% → 99.96% | 94.94% → 99.96% | [review](../assets/issue_9395/protected/native/review_036.png) · [overlay](../assets/issue_9395/protected/native/overlay_036.png) | [review](../assets/issue_9395/protected/wasm/review_036.png) · [overlay](../assets/issue_9395/protected/wasm/overlay_036.png) |

재저장본 Native에서 보정 전보다 출력이 바뀐 쪽은 36쪽 한 곳이다. 기존 31쪽의 98% 미달은
새 한컴 Print에서도 재현된 PDF CID 글꼴 참조 오류에 해당한다. 이 쪽의 Native PNG는
보정 전과 동일하다. [31쪽 원인 조사](../assets/planet6897_20261125/kaf-print-repair/README.md#31쪽-기준-pdf-확인)

## 회귀·보호 검증

- macOS 최소 글꼴 환경에서 전체 nextest 14,124건 통과, 실패 0건, 45건 skip.
  Native Skia 4,225건 통과, 실패 0건, 12건 ignored. 집중 suite는 372건 통과.
- 포맷, Native·WASM32·workspace Clippy, 잠금 workspace 빌드, suite 계약 검사 통과.
  fresh WASM은 Mac에서 잠금 패키지 스크립트의 `--no-opt`로 빌드해 Chrome Canvas에서 검증했다.
- 보호 매니페스트에 검증한 허용 해시 11건을 추가했다. 보호 문서 23종의 Native 2,023쪽과
  fresh WASM 2,023쪽에서 공식 두 자리 표시값의 상승·하락·누락이 모두 0쪽이었다.
  편람 HWPX Native 383쪽을 포함한다. [정본 매니페스트](../../verification/protected-font-supply/manifest.json)
- 코드 head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/45654126759),
  [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/45654126585),
  [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/45654126799),
  [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/45654126988),
  [Proptest](https://github.com/edwardkim/rhwp/actions/runs/45654127673),
  [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/45654126227)가 모두 통과했다.

## 병합 뒤 기록

#9395에 병합 SHA와 검증 결과를 남긴다. 재저장본과 크기 보호 대조본 36쪽의 review·overlay를
병합 SHA에 고정해 표시하고, [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)과
최종 CI를 연결한다.
