---
kind: verification
status: completed
canonical: mydocs/manual/verification/visual_sweep_guide.md
last_verified: 2026-08-02
---

# Task #3738 Stage 17 visual sweep — HWP p78–p80 표 25 URL 각주

## 기준과 실행 범위

- HWP: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- 동일 문서 HWPX: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwpx`
- 한컴 2020 기준 PDF: `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
- 명령: `python3 scripts/visual_sweep.py --key issue3738-stage17-hwp-p078-080 --hwp <HWP> --pdf <PDF> --pages 78-80 --dpi 144 --rhwp-bin target/review-planet6897-20260802/release-test/rhwp --out /private/tmp/rhwp-stage17-p078-080-sweep.41vbIY`

`--pages 78-80`은 PDF viewer 기준 1-based 번호다. SVG와 render tree는 HWP 전체 220쪽을 내보냈지만, raster/PDF/compare/overlay/review는
78–80쪽만 수행했다. `requested_pages=completed_pages=[78,79,80]`, `missing_pages=[]`, `run_state=complete`다.

## semantic 판정

| 페이지 | 기준 계약 | 확인 결과 |
| --- | --- | --- |
| 78 | 표 25 첫 fragment와 기존 각주 105·106 | FootnoteArea에는 105·106만 있고 표 bottom `957.5px`가 footnote separator `987.6px`보다 위다. |
| 79 | 표 25 continuation와 URL 각주 107–111 | table bottom `838.5px`, FootnoteArea 시작 `851.9px`; 107–111만 존재한다. |
| 80 | 본문 p887–p889 뒤 URL 각주 112–124 | p889 마지막 p80 줄 bottom `469.8px`가 separator `501.6px`보다 위다. 112–124만 존재하며 p889 tail만 p81로 이어진다. |

pixel match는 p78 `90.71027%`, p79 `88.00550%`, p80 `90.41151%`이고 structure flag는 0건이다. HWP/PDF의 폰트 raster 차이는
overlay ink 비율을 크게 낮추므로 이 수치만으로 결론을 내리지 않았다. 세 review PNG의 표 조각·각주 번호·구분선과 render-tree 좌표를
사람이 함께 확인해 p80의 body/각주 overlap이 사라진 것을 판정했다.

## 장기 증적

- [p78 3-way review](../pr/assets/pr_3740_issue3738_stage17/hwp_p078_review_after.png)
- [p79 3-way review](../pr/assets/pr_3740_issue3738_stage17/hwp_p079_review_after.png)
- [p80 3-way review](../pr/assets/pr_3740_issue3738_stage17/hwp_p080_review_after.png)

원본 HWP/HWPX/PDF의 SHA-256과 저장 위치는 [Stage 17 작업 기록](task_m100_3738_stage17.md)에 기록했다. 이 증적은 p78–p80
표 25/각주 배분에만 적용하며, 전체 220/215쪽 pagination 정합 또는 다른 이월 페이지의 완료를 뜻하지 않는다.
