# samples/task2156 — Issue #2156 검증 샘플

## width_ladder.hwpx (합성)

`tools/make_width_ladder.py` 로 생성한 문자폭 사다리 (마지막 생성분: digit/
middot 정밀 k=800/1600 판). 문자 클래스 × 반복수 사다리 문단을 선언 극소
높이(288HU) 1열 표에 배치 — 한글이 행을 콘텐츠로 키우므로 행높이 → #2150
공식(ls=100%: n×em+pad) 역산 → 정확한 줄수 → 유효 문자폭 구간.

측정: `tools/probe_width_ladder.py` (한글 COM 행높이 직독 + 구간 교집합).

## 확정 결과 (#2156)

한글은 함초롬바탕(HCR Batang) 문서의 **비한글 문자(라틴·숫자·구두점·U+00B7)를
Haansoft Batang(한컴바탕, HBATANG.TTF) 메트릭으로 렌더**한다 (음절만 HCR
Batang hmtx, 공백은 useFontSpace=0 고정 0.5em):

| 문자 | HCR hmtx(종전 rhwp) | 한글 실측 = Haansoft |
|------|--------------------:|---------------------:|
| ( )  | 0.320em | **0.500em** |
| , .  | 0.320em | 0.291em |
| 0-9  | 0.550em | 0.583em |
| A    | 0.706em | 0.750em |
| a    | 0.569em | 0.500em |
| ·    | 0.320em | 0.333em |

수정: `text_measurement::haansoft_latin_override` (+ `HAANSOFT_BATANG_ASCII`
테이블, 추출 도구 `tools/extract_haansoft_table.py`). 회귀 테스트:
`issue_2156_hcr_batang_latin_uses_haansoft_metrics` (text_measurement.rs 유닛).
자가검증: `rhwp measure-width --size 10 --repeat 100 "(" → 6.667px`.
