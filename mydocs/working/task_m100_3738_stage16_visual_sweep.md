---
kind: verification
status: completed
canonical: mydocs/manual/verification/visual_sweep_guide.md
last_verified: 2026-08-02
---

# Task #3738 Stage 16 visual sweep — HWP p31 두 줄 각주 30

## 기준과 범위

- 입력: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- 기준: `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
- 대상: native HWP renderer, 144 DPI, rhwp 출력 221쪽과 기준 PDF 215쪽
- 선택 페이지: semantic owner가 같은 p31–p32. p31 각주 30 첫 줄과 p32 continuation tail을 대조했다.

## 실행 상태

`visual_sweep.py`에 `--pages 31-32`를 주어 실행했다. SVG와 render tree는 문서 전체 221쪽을 생성했으나,
raster/PDF/overlay/review 비교는 요청한 두 쪽만 수행했다. `requested_pages=[31,32]`, `completed_pages=[31,32]`,
`missing_pages=[]`, `run_state=complete`다.

## 판정

| rhwp 페이지 | PDF와의 semantic 계약 | 결과 |
| --- | --- | --- |
| 31 | 문단 421의 reset 전 네 줄, 각주 30 번호·separator·첫 줄 | body bottom `1006.0px`와 separator `1019.0px`가 분리됐다. `Aktuelle Entwicklungen … und`만 표시한다. |
| 32 | reset 뒤 본문과 각주 30 연속 tail | `„incentives” für Transplantationszentren`만 하단에 있고 번호·separator는 반복하지 않는다. |

overlay 결과는 pixel match 평균 `92.94880%`(최저 `91.31531%`), ink match 평균 `40.88410%`(최저 `35.50389%`)이며,
구조 검사 flagged page는 0건이다. 오래된 HWP 원본의 서체 raster 차이는 수치만으로 합격을 판정하지 않았고, review PNG와
render-tree 경계로 위 ownership과 비겹침을 직접 확인했다.

## 증적

- [p31 review after](../pr/assets/pr_3740_issue3738_stage16/hwp_p031_review_after.png)
- [p32 review after](../pr/assets/pr_3740_issue3738_stage16/hwp_p032_review_after.png)

이 결과는 p31–p32의 두 줄 각주 fragment에 한정한다. 전체 HWP 출력은 여전히 221쪽이고 기준 PDF는 215쪽이므로,
전체 visual sweep 또는 전체 pagination 완료 판정으로 사용하지 않는다.
