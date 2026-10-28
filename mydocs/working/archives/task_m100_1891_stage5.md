# Task m100 #1891 Stage 5

## 목표

Stage 4에서 확인한 HWP5 위장 샘플의 페이지 압축 원인을 코드에 반영할 수 있는지
검증한다.

대상은 `line_segs` 가 없고 표시 텍스트도 없지만 `char_count > 0` 및
`char_shapes` 를 가진 빈 문단이다. 현재 이 문단이 0 높이로 측정되어 PDF 기준보다
본문이 위로 압축된다.

## 수정 원칙

- 특정 샘플명, 페이지 번호, PR/issue 번호, 임의 계수로 분기하지 않는다.
- 문서 속성인 `Paragraph.char_count`, `Paragraph.char_shapes`,
  `CharShape.base_size`, `ParaShape.line_spacing` 에 근거한다.
- `Typesetter::format_paragraph`, `HeightMeasurer::measure_paragraph`, 렌더링 advance
  경로가 같은 높이를 사용하도록 맞춘다.

## 확인 결과: `char_shapes` 기반 일괄 보정은 실패

`line_segs` 가 없고 표시 텍스트가 없지만 `char_count > 0`, `char_shapes` 를 가진
빈 문단을 모두 `CharShape.base_size` 및 `ParaShape.line_spacing` 으로 측정하는
실험을 했다. 빌드는 통과했지만 결과가 기준과 맞지 않았다.

| sample | pdf_pages | hwp_pages | hwpx-name hwp5 pages |
| --- | ---: | ---: | ---: |
| 76076 | 82 | 85 | 86 |
| 80168 | 157 | 159 | 159 |
| 80250 | 17 | 17 | 17 |
| 86712 | 65 | 64 | 65 |

`80250` 은 쪽수만 맞았지만 7쪽 경계가 잘못됐다. PDF 7쪽은 `② 이해관계자
의견수렴` 표와 `3. 규제목표` 제목까지 포함해야 하는데, 실험 결과는 7쪽에
`pi=36` 표가 `rows=0..1` 로 조기 분할됐다. 즉 빈 문단 전체를 일반 줄 높이로
살리는 방식은 표/본문 경계를 과하게 밀어 merge 후보가 아니다.

따라서 해당 코드 변경은 버렸다.

## 다음 구현 후보

빈 문단 자체의 `char_shapes` 만으로 높이를 결정하지 않는다. HWP5 원본에 저장된
주변 `LineSeg.vertical_pos`, `line_height`, `line_spacing`, 문단/표 흐름 관계를 함께
읽어 실제 저장 flow 에서 빈 문단이 공간을 차지했는지 추정한다.

특히 다음을 분리해 확인한다.

- HWP5 원본 로드 쪽수 부족: 주변 `LineSeg.vertical_pos` 로 본문 흐름 간격을 복원할
  수 있는지 확인한다.
- HWP5 → HWPX export → 재파스 쪽수 팽창: #1913 이후에도 HWP5 원본의 `line_segs`
  부재 의미를 HWPX 산출물에서 보존해야 하는지 확인한다.

## 검증 계획

- `80250_regulatory_analysis.hwp` 와 `samples/issue1891/80250_regulatory_analysis.hwpx`
  페이지 수가 PDF 기준 17쪽에 맞는지 확인한다.
- 4개 샘플에 대해 PDF/HWP/HWPX 페이지 수가 모두 같은지 확인한다.
- 회귀 테스트는 관련 테스트부터 실행하고, 통과 후 PR 준비 전 전체 검증으로 확대한다.
