# Task #1588 — Stage 3 완료보고서

**단계**: 통제 비교 검증 (채택 게이트)
**브랜치**: `local/task1588`
**바이너리**: `local/task1588` HEAD (f3f2dc0a 위 빌드)

## 1. fidelity 전수 통제 비교

| 항목 | shapeComment 전 (fidelity11) | 후 (fidelity12) |
|------|------:|------:|
| 총 파일 | 10062 | 10261 (수집 진행) |
| IR_DIFF | 7 | **4** |

**공통 9952건 per-file 통제 비교**:

| 분류 | 건수 |
|------|----:|
| 개선 (IR_DIFF→PASS) | **3** (36389418·36391302·36392900 = 선 도형 shapeComment 3건) |
| **회귀 (PASS→IR_DIFF)** | **0** |
| 신규 파일(198) 중 IR_DIFF | 0 |
| **순효과** | **+3** |

→ 채택 게이트 충족: **순효과 +3 > 0, 악화 0**.

잔존 4건 = 36384689·36385445·36388711(Class C, para0 char_shape 시프트) +
36386761(Class D, spurious 0,0). Ruby·shapeComment 클래스 완전 해소.

## 2. 회귀 가드 영속화

- 단위: `task1588_line_shape_comment_emitted` / `task1588_line_shape_no_comment_when_empty`.
- 통합: `36392900` opengov 고정 말뭉치 편입 + snapshot PASS.

> Hangul 오라클: shapeComment 3건 모두 한글 COM 열기 ERR(COLLAPSE 아님) — 해당 파일군의
> 도구측 열기 이슈. shapeComment 는 **비시각 메타데이터**이고 IR diff=0 검증 완료라 채택 무영향.

## 3. 결론

선 도형 shapeComment 드롭 해소(개선 3, 회귀 0). 채택. 잔존 IR_DIFF 4건은 Class C/D 별건.
