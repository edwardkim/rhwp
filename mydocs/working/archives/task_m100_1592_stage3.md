# Task #1592 — Stage 3 완료보고서 (통제 비교)

**단계**: 통제 비교 검증 (채택 게이트) · **브랜치**: `local/task1592`
**바이너리**: local/task1592 HEAD 위 빌드

## fidelity 전수 통제 비교 (fidelity13→14, 전체 경로 키)

| 항목 | 전(13) | 후(14) |
|------|------:|------:|
| 총 파일 | 10581 | 10831 |
| IR_DIFF | 4 | **3** |

공통 10581건:
- 개선 1 (36386761 백제학연구총서 위탁판매 의뢰 목록)
- **회귀 0**, 신규 IR_DIFF 0
- **순효과 +1**

→ 채택 게이트 충족(순효과>0·악화0). **빈 문단 광역 변경에도 회귀 0** — char_shapes=[] 조건이
좁아(대부분 빈 문단은 [(0,0)]) blast radius 최소.

## 회귀 가드
- 단위: `task1592_empty_paragraph_no_spurious_charshape` + `task1378_..._single_run_id_zero` 갱신.
- 통합: `36386761_백제학연구총서위탁판매의뢰목록` opengov 편입 + snapshot PASS.

## 결론
빈 문단 spurious (0,0) 해소(개선 1, 회귀 0). 채택. 잔여 IR_DIFF 3건(Class C 2 + C2 1)은 별건.
