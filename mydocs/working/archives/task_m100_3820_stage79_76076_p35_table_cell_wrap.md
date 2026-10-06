---
kind: analysis
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 79 — 76076 p35 표 셀 line-wrap PDF 정합성

## 입력과 분리 범위

대상은 `samples/76076_regulatory_analysis.hwp` p35(0-based 34)와 한컴 기준 PDF
`samples/issue1891/76076_regulatory_analysis-2024.pdf`다. Stage 78 (`192d7ff87`)은
RowBreak 마지막 physical tail의 높이와 가운데 정렬을 보정했다. 이 단계는 그 fragment
height 계약을 다시 바꾸지 않고, p35 표 본문 셀의 **가용 폭·glyph advance·wrap point**만
다룬다.

## 관찰된 결함

같은 `주요내용` 열의 본문 첫 줄에서 기준 PDF는
`…사업장에서 밀가루 등이 반죽된`으로 끝난 뒤 다음 줄을 `용기를 …`로 시작한다. 현재
RHWP는 첫 줄에 `용`을 추가로 넣고 둘째 줄을 `기를 …`로 시작한다. outer table x/column
bbox와 row boundary가 1--2px 범위인 상태에서도 발생하므로, 단순 outer-table 폭이나
Stage 78 tail height의 문제가 아니다.

## 분석 질문과 완료 조건

1. 해당 셀의 source char style, HWP `LINE_SEG`, renderer가 쓰는 content box/padding,
   SVG cluster advance를 각각 추출해 어느 계층에서 PDF와 달라지는지 확정한다.
2. PDF text span/glyph advance와 RHWP SVG를 같은 줄 단위로 비교해, 실제 가용 폭이 다른지
   또는 같은 폭에서 layout font metric이 과소/과대인지 구분한다.
3. 원인이 확정되기 전에는 전역 한양계 font metric·table padding·SVG `textLength`를
   조정하지 않는다. 다른 표와 일반 문단에 미치는 범위가 너무 넓다.
4. 수정이 필요하면 다음 implementation Stage에서 이 fixture와 같은 source contract를 가진
   focused regression, direct PDF p35/p36 evidence, overflow-cell baseline을 함께 남긴다.

## 방법

- 격리 release-test binary의 p35 render tree/SVG와 `rhwp dump --section 0 --para 347`을
  대조한다.
- 기준 PDF는 text extraction과 page raster를 함께 사용한다. raster pixel diff는 후보
  선별용이며, 줄 끝 문자·cell clip·available width의 직접 판정이 우선이다.
- Stage 75가 실제 한양계 TTF 설치 뒤에도 결과가 변하지 않았음을 전제로 한다. 따라서
  "폰트가 없어서"라고 가정하지 않고 source line segment와 renderer advance를 수치로
  확인한다.

## 확정 분석

대상 셀 `(outer row=4, col=2)`의 첫 paragraph는 107자와 연속 `char_offsets`를 가지지만
`LINE_SEG`는 **0개**다. 따라서 PDF의 줄 끝을 직접 보존해 재생할 source boundary는 없고,
`recompose_for_cell_width()`가 font metric으로 새로 reflow하는 경로가 맞다.

기준 PDF p35를 pypdfium2 character box로 읽어 SVG 좌표계(4/3배)로 환산했다. 한글의
advance는 약 `15.04px`로 RHWP의 `14.96px`와 사실상 같다. 반면 단어 사이 gap은 PDF
`9.92px`, RHWP `7.75px`로 약 `2.17px` 부족하다. 이 gap이 첫 줄의 8개 공백에 누적되어
RHWP가 `용` 한 글자를 더 넣는다.

원인은 `한양중고딕`의 layout space 처리다. source face는 metric resolver에서
`HanyangJungGothic`을 사용하며, 자동 생성 `HANYANGJUNGGOTHIC_LATIN_0`에는 source TTF의
space advance `518/1024 em`가 보존돼 있다. 그러나 현재 `measure_char_width_embedded()`는
모든 face의 U+0020을 한컴 반각 관례라며 이 테이블을 읽지 않고 `em/2` (`512/1024 em`)로
강제한다. 글자 간 추가 spacing을 더한 현재 output은 위 `7.75px`와 일치한다. 한컴 PDF의
`9.92px`에는 약 `670/1024 em` space advance가 필요하다. 한글 glyph width, cell inner width,
RowBreak fragment height는 이 차이의 원인이 아니다.

macOS에서 `한양중고딕`이라는 family 이름은 직접 등록되어 있지 않고 fallback chain의
`HY중고딕`(HYGothic-Medium, `H2GTRM.TTF`)가 SVG paint를 담당한다. CoreText로 읽은
HYGothic-Medium의 한글 advance는 `14.6667px`, space는 `4.8841px`였다. 이 값은 SVG
paint font 확인용이며, PDF가 담은 Type3 `T6/T18`의 line decision을 결정하는 layout
metric 자체는 아니다. Stage 80은 source face **한양중고딕의 space metric만** PDF 실측값으로
보정하고, HYGothic-Medium과 다른 HY 계열에는 손대지 않는다.

## Stage 80 이관

자동 생성 font metric 테이블은 직접 수정하지 않는다. `measure_char_width_embedded()`에
한양중고딕 한정 PDF-calibrated U+0020 advance (`670/1024 em`)를 명시한 뒤, p35 첫 줄이
`…반죽된`에서 끊기는 focused regression과 p35/p36 PDF direct pair를 검증한다.
다른 문서의 무저장-lineSeg 한양중고딕 셀에도 영향이 있으므로 overflow-cell baseline,
전체 release-test, clippy를 함께 실행한다.
