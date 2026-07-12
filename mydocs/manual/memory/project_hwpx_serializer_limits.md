---
name: HWPX serializer fidelity 한계 (#1315 확정)
description: HWPX 저장 시 문단 run 평탄화·셀/글상자 컨트롤 소실·합성 lineseg — 작업지시자가 부족 판정, 후속 이슈 대상
type: project
---

#1315 baseline 구축에서 확정된 HWPX serializer 한계 (2026-06-11 작업지시자 판정: "문단, 스타일, 표 안 이미지, 글상자 내 이미지 처리 부족"):

1. 문단 run 평탄화 — `char_shapes[0]`만 charPrIDRef 출력 (section.rs), 서식 분할 소실
2. 셀 subList 텍스트 전용 (table.rs write_sub_list) — 셀 내 그림/컨트롤 소실
3. 글상자 drawText 동일 패턴 (shape.rs write_draw_text_paragraph)
4. 합성 lineseg (vertsize=1000/baseline=850 고정) → 페이지 수 변화 (form-002 10→17쪽, 보도자료 9→13쪽)

**Why:** #1315는 "구조(뼈대) 보존" 게이트만 구축, full fidelity는 의도적 범위 외. baseline 통과 ≠ 시각 충실 (diff_documents는 카운트만 비교 — 컨트롤/lineseg 미비교).

**How to apply:** HWPX 직렬화 후속 작업 시 이 4개 한계가 출발점. 해소 시 `tests/hwpx_roundtrip_baseline.rs`의 xfail 승격 테스트와 `mydocs/manual/hwpx_roundtrip_baseline.md` 갱신 필요. 관련: exam_social.hwpx borderFillIDRef 31 xfail.
