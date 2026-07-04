# Task #1598 Stage 4 — 통제 비교 (한글 오라클)

## End-to-end 통제 (실제 rhwp 직렬화 경로)

새 바이너리로 36385226 roundtrip → 한글 PageCount:

| 파일 | 한글 PageCount | 비고 |
|------|---------------|------|
| orig | 3 | 정답 |
| rt (수정 전, 지오메트리 드롭) | 2 | 붕괴 |
| **new-rt (#1598 파서+직렬화)** | **3** | **해소** |

지오메트리 값 정확 일치(orig==rt): center(460,460)/ax1(460,0)/ax2(920,460)/start1·end1·
start2·end2(0,0).

## 잔여 long-tail 현황

- 36385226 (ellipse×9): **해소** (3→3).
- 36389684: 현재 바이너리에서 orig=rt=2 (붕괴 없음) — generic-shape 미보유(컨테이너/picture).

## 채택 판정: **채택**

- 통제 비교 악화 0 (개선 1: 36385226 붕괴 해소).
- baseline/lib/clippy/fmt 회귀 0.
- IR diff=0 유지.

## 보류 (별 타스크 후보)

- ellipse/arc 태그 전용 속성(intervalDirty/hasArcPr/arcType) — 붕괴 무관, 모델 미보유.
- arc 의 start/end 점(모델 ArcShape 는 center/축만 보유) — 실문서 출현 시 확장.
