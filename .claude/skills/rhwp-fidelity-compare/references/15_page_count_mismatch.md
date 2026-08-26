# 15 — 예외: 쪽수 불일치 (F11)

`page-count-ledger.tsv` 는 기준 PDF, `--export-all-svg` 의 전체 SVG,
`--layout-ledger` 의 전체 render tree 쪽수를 분리해 기록한다.

정본: "페이지 수 차이는 전역 page-break 보정의 근거가 아니라
individual owner 조사 후보를 여는 신호다."

## 열

```
source	pages	delta_vs_reference	scope	note
reference_pdf	35	0	full PDF	comparison baseline
rhwp_svg	37	2	full export	page-count difference is a candidate, not a global-break fix
rhwp_render_tree	37	2	full render tree	page-count difference is a candidate, not a global-break fix
```

`--export-all-svg` 없이 돌리면 SVG 전체 쪽수는 `not counted` 다.
선택 쪽 캐시 수를 전체인 척하지 않는다.

## 요청 범위 overflow

```
요청 끝 쪽 214가 기준 PDF 마지막 index 34를 넘습니다.
```

종료 코드 2. 이건 ledger 가 아니라 argv 오류다. 끝 쪽을
`len(pdf)-1` 로 줄인다.

## 어떻게 읽나

1. PDF 쪽이 편집기 PageCount 와 같은가. 다르면 오라클이
   맞춰찍기/배율 산출일 수 있다 (hangul_pdf_baseline). 비교를
   참고 등급으로 내린다 (F17).
2. PDF 는 맞고 SVG/tree 가 더 많으면 rhwp 가 쪽을 일찍 나눈
   **owner 후보** 다. 전역 "page-break 한 줄 패치" 를 열지 않는다.
3. SVG 가 더 적으면 뒤 쪽이 합쳐졌거나 export 가 잘린 것. run-state
   missing 과 교차한다 (F12).
4. SVG 와 tree 가 서로 다르면 도구 불일치 후보다. 문서 결함보다
   먼저 적는다.

## 하지 말 것

- `typeset` 의 전역 페이지 높이를 숫자 맞추려고 줄이기
- "2쪽 차이 = +2 page-break 버그" 로 이슈 제목을 달기
- ledger 없이 dump-pages 숫자만으로 한컴과 비교했다고 하기
  (`dump-pages` 는 rhwp 내부 쪽. 한컴 PDF 쪽이 아니다)

## 레시피

```bash
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 34 \
  --text-only --export-all-svg --layout-ledger \
  --out-dir /tmp/rhwp-fidelity-plan
cat /tmp/rhwp-fidelity-plan/page-count-ledger.tsv
# delta 가 있으면 text-owner-shift 와 boundary ledger 를 같은 폴더에서 연다
```

owner 가 p18→p19 로 보이면 그 두 쪽만 시트로 올린다. 35쪽 레이아웃
정책을 한꺼번에 바꾸지 않는다.

## 맞춰찍기 오라클

편집기 25쪽, PDF 13쪽이면 ledger 의 reference_pdf=13 이다. rhwp 가
25를 내는 것은 "rhwp 가 많다"가 아니라 **오라클이 축소본** 일 수
있다. provenance 의 exportPath 를 먼저 고친다.

## 에이전트 한 줄

"PDF 35 · SVG 37 · tree 37. 차이는 후보이며 전역 page-break 패치
근거가 아닙니다. p18–p19 owner 원장을 유지자에게 넘깁니다."
