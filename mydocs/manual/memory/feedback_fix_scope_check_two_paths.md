---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-17
name: 정정 영역의 두 경로 영역 점검 영역의 패턴 영역
description: 한 본질 영역의 정정 영역이 다중 경로 영역에 적용 영역 필요 영역인 케이스 — layout 단계 영역 정정 영역만으로는 부족 영역하고 영역 자동 보정 / reflow / pre-process 단계 영역도 동일 본질 영역 정정 영역 필수 영역
type: feedback
originSessionId: 4ef64500-5782-4a52-a9a8-9330ea6c128b
---
본질 영역의 정정 영역이 한 경로 영역만 영역 적용 영역되면 영역 다른 경로 영역 (예: 자동 보정 / reflow / pre-process) 영역에서 회귀 영역 가능 영역. 시각 판정 영역에서 검출 영역.

**Why:** PR #673 (Task #671) 사례 영역에서 발견 — `recompose_for_cell_width` 영역의 layout 단계 영역 정정 영역만으로는 영역 자동보정 모드 영역에서 회귀 영역. 자동보정 영역의 `reflow_linesegs_on_demand` 영역에서 셀 paragraph 영역에 column 폭 영역으로 LINE_SEG 영역 채움 영역 → layout 영역의 가드 #1 (`line_segs.is_empty()`) 영역 거짓 영역 → 정정 영역 미적용 영역. 작업지시자 시각 판정 영역에서 발견: "그대로 보기 하면 2줄로 처리되지만 오히려 자동보정 선택하면 한줄로 겹쳐집니다." 메인테이너 후속 commit (A1) 영역으로 자동보정 영역 자체 영역의 셀 폭 영역 정정 영역 정합.

**How to apply:**

### review 영역 시
1. **본질 영역 점검**: PR 영역의 정정 영역이 어느 단계 영역 (parsing / preprocessing / reflow / layout / rendering) 영역에 적용 영역인지 점검
2. **다중 경로 영역 검색**: 동일 본질 영역의 다른 경로 영역 영역 점검:
   - `grep -rE "<관련 함수명>" src/` 영역 — 동일 함수 영역의 호출처 영역 영역 모두 영역 점검
   - `reflow_*` / `validate_*` / `compose_*` / `recompose_*` 영역 영역 함수 영역의 동일 본질 영역의 다른 인자 영역 영역
   - 자동보정 모드 영역 (`reflow_linesegs_on_demand`, `validate_linesegs`) 영역의 호출 영역 영역
3. **가드 영역 점검**: PR 영역의 가드 영역이 다른 경로 영역의 결과 영역 영역에서 거짓 영역으로 영역 우회 영역 가능 영역인지 점검 영역
4. **시각 판정 시 모드 영역 점검**: rhwp-studio 영역의 "그대로 보기" / "자동 보정" 영역 영역 모두 영역 점검 영역 — 두 모드 영역의 결과 영역 영역 정합 영역인지 점검 영역

### 정정 시
- 다중 경로 영역의 동일 본질 영역 정정 영역 — layout 단계 영역만 영역으로 부족 영역 하면 영역 reflow 단계 영역 / preprocessing 단계 영역도 동일 본질 영역 정정 영역
- 메인테이너 후속 commit 영역의 패턴 영역 정합 (`feedback_pr_supersede_chain` 영역의 확장 영역)

**관련 룰**:
- `feedback_visual_judgment_authority` — 결정적 검증 영역 통과 영역에도 시각 판정 영역에서만 회귀 검출 영역
- `feedback_pr_supersede_chain` — PR + 메인테이너 후속 정정 영역의 통합 머지 패턴 영역
- `feedback_image_renderer_paths_separate` — 동일 본질 영역의 다른 경로 영역 (svg/web_canvas/paint/json) 영역 영역의 사전 sweep 영역 패턴 영역의 정합 영역
- `project_hancom_lineseg_behavior` — 한컴 LINE_SEG 비표준 영역의 본질 영역
