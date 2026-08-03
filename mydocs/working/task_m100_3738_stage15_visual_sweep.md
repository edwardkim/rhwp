---
kind: verification
status: completed
canonical: mydocs/manual/verification/visual_sweep_guide.md
last_verified: 2026-08-02
---

# Task #3738 Stage 15 visual sweep — HWP 표 24 reset tail과 그림 51

## 기준과 범위

- 입력: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- 기준: `pdf/pr3740/`에 보관한 한컴 2020 변환 PDF (215쪽)
- 대상: native HWP renderer, 144 DPI, rhwp 출력 221쪽
- 선택 페이지: rhwp p76–p79. PDF p76–p79의 표 24, 그림 51, `3. EU`의 semantic owner와 대조했다.

## 실행 상태

p76–p79 연속 raster/overlay run은 p76–p78을 완료한 뒤 환경의 단일 실행 한도에 도달해 p79 summary 전
종료됐다. 완료되지 않은 연속 run을 전체 범위 완료로 취급하지 않았다. p79는 같은 revision에서 단일 page
sweep을 별도 실행해 `requested_pages=[79]`, `completed_pages=[79]`, `run_state=complete`를 확인했다.

## 판정

| rhwp 페이지 | 확인 항목 | 결과 |
| --- | --- | --- |
| 76 | 표 24 row 4 reset 전 세 줄, 각주 영역 | 기준 PDF와 같은 tail 분할, structural flag 없음 |
| 77 | row 4 reset 후 tail, 그림 51과 caption, 각주 103·104 | 그림과 caption이 각주 위에 함께 존재, structural flag 없음 |
| 78 | 다음 절 owner | `3. EU`로 시작하며 그림 51 없음, structural flag 없음 |
| 79 | 뒤따르는 표 본문 | 표 내용 존재, 단독 그림 51/빈 표 페이지 없음, structural flag 없음 |

연속 run(p76–p78)의 overlay summary는 pixel match 평균 `92.78160%`, 최저 `92.32685%`, ink match 평균
`16.37868%`, 최저 `9.54044%`였다. p79 단일 run은 pixel match `91.39947%`, ink match `7.95157%`였다.
텍스트·서체 raster 차이가 큰 오래된 HWP 원본이므로 수치만으로 합격을 정하지 않고, table row fragment와
그림/caption/각주의 page owner를 review PNG로 확인했다.

## 증적

- [p76 review after](../pr/assets/pr_3740_issue3738_stage15/hwp_p076_review_after.png)
- [p77 before](../pr/assets/pr_3740_issue3738_stage15/hwp_p077_review_before.png)
- [p77 after](../pr/assets/pr_3740_issue3738_stage15/hwp_p077_review_after.png)
- [p78 before](../pr/assets/pr_3740_issue3738_stage15/hwp_p078_review_before.png)
- [p78 after](../pr/assets/pr_3740_issue3738_stage15/hwp_p078_review_after.png)
- [p79 after](../pr/assets/pr_3740_issue3738_stage15/hwp_p079_review_after.png)

이 결과는 표 24/그림 51의 선택 흐름을 확인한 것이다. HWP 전체 출력은 아직 PDF보다 6쪽 많으므로 전체
pagination 정합 또는 전체 visual sweep 완료 판정에는 사용하지 않는다.
