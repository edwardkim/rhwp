# M-fid fatten 요약

- 클레임: `M-fid`
- 생성기: `tools/fidelity_compare/fatten_text_layer.py`
- 시각: `2026-08-18T11:01:01Z`
- 텍스트층 케이스: **244**
- --text-only 경로: **32**
- SVG 픽스처: **4**
- 소실/과잉/치환/일치: 26/92/36/90

## 하지 않은 것

- `scripts/visual_sweep.py` 미수정
- canvaskit_policy · serializer · layout-anomaly · render_backend · proptest 미수정
- gym 미수정

## 산출물

- 파일 수: **307**
- `fixtures/text_layer/cases/` — 쪽별 소실·과잉·치환·일치 픽스처
- `tables/{loss,excess,substitution,match}.tsv` — 분류표
- `fixtures/text_only_paths/` — `--text-only` parse·산출 계약
- `fixtures/svg/` — clip/PUA visible walker
- `WORKING.md` — 작업 기록
