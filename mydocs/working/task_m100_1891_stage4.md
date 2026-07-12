# Task m100 #1891 Stage 4

## 목표

PDF 기준보다 HWP5 원본 로드 페이지 수가 부족하거나, HWP5 -> HWPX 재파스 페이지 수가
팽창하는 원인을 문서 속성 기준으로 좁힌다.

## Stage 3 기준선

| sample | pdf_pages | hwp_src | hwp_rt | hwpxname_src | hwpxname_rt |
| --- | ---: | ---: | ---: | ---: | ---: |
| 76076 | 82 | 81 | 87 | 82 | 88 |
| 80168 | 157 | 150 | 172 | 149 | 173 |
| 80250 | 17 | 16 | 17 | 16 | 19 |
| 86712 | 65 | 64 | 71 | 65 | 72 |

## 확인할 축

- `respect_vpos_reset` 계열 옵션이 원본 페이지 수를 PDF 기준으로 이동시키는지 확인한다.
- 옵션성 동작이면 일반 렌더 경로에 반영 가능한 문서 속성인지 확인한다.
- 옵션으로 해결되지 않으면 표/셀 split, lineSeg 부재 문단 높이, ParaShape spacing 계열로 분리한다.

## 확인 결과 1: respect_vpos_reset

`output/poc/task1891-stage4/vpos_page_counts.tsv`

| sample | plain_pages | respect_vpos_pages |
| --- | ---: | ---: |
| 76076.hwp | 81 | 81 |
| 80168.hwp | 150 | 150 |
| 80250.hwp | 16 | 16 |
| 86712.hwp | 64 | 64 |
| 76076.hwpx-name HWP5 | 82 | 82 |
| 80168.hwpx-name HWP5 | 149 | 149 |
| 80250.hwpx-name HWP5 | 16 | 16 |
| 86712.hwpx-name HWP5 | 65 | 65 |

`respect_vpos_reset` 는 이 군집의 페이지 수 차이에 영향을 주지 않는다.

## 확인 결과 2: 80250 첫 페이지 경계 이탈

가장 작은 샘플인 `80250_regulatory_analysis` 를 PDF 텍스트와 rhwp `export-text`
결과로 먼저 비교했다.

- 기준 PDF: 17쪽
- HWP 원본 로드: 16쪽
- `.hwpx` 확장자 HWP5 원본 로드: 16쪽

첫 차이는 PDF 7~8쪽 경계에서 발생한다.

- PDF 7쪽은 `3. 규제목표` 제목까지 포함하고 끝난다.
- PDF 8쪽은 `ᄋ 성범죄ㆍ강력범죄 등을 저지른 자 등을 특별교통수단 운전자 결격...`
  본문으로 시작한다.
- rhwp HWP 7쪽은 `3. 규제목표` 아래 1x1 표 본문까지 포함하고, 8쪽이
  `Ⅱ. 규제의 적정성` 으로 시작한다.

`dump-pages samples/80250_regulatory_analysis.hwp -p 6` 요약:

- body_area 높이: `971.3px`
- 현재 7쪽 사용 높이: `942.9px`
- `pi=38` 제목 `3. 규제목표`: `h=30.0`
- `pi=40` 1x1 표: `638.2x83.8px`
- `pi=37`, `pi=39` 빈 문단: 모두 `h=0.0`

반면 빈 문단은 원본 속성을 가지고 있다.

- `pi=37`: `cc=1`, `CS base=1000`, `PS line=150/Percent`
- `pi=39`: `cc=1`, `CS base=400`, `PS line=150/Percent`

이 두 빈 문단을 글자 모양/문단 모양 기반 기본 줄 높이로 계산하면 각각 약
20px, 8px 수준이다. 합계 약 28px 는 현재 7쪽 하단 잔여 여유와 거의 같아서,
`pi=40` 표가 PDF처럼 8쪽으로 넘어가는 설명과 맞다.

## 다음 단계 후보

코드 수정은 Stage 5에서 분리한다. 수정 후보는 특정 샘플/쪽번호가 아니라
`line_segs` 는 없지만 `char_count > 0` 이고 `char_shapes` 를 가진 빈 문단을
문서 속성(`CharShape.base_size`, `ParaShape.line_spacing`) 기준으로 측정하는 것이다.

우선 다음 경로가 같은 규칙으로 빈 문단을 0 높이로 만드는지 확인한다.

- `Typesetter::format_paragraph`
- `HeightMeasurer::measure_paragraph`
- 렌더링 `paragraph_layout` 의 빈 문단 advance
