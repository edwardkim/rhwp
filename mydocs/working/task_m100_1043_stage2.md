# Stage 2 완료보고서 — Task M100 #1043

## 작업
중첩 표(1×1 wrapper) 외곽선 회귀 테스트 추가 (`tests/issue_nested_table_border.rs`).

## 픽스처 교체 경위 (v2)
초안 테스트는 `samples/2. 인공지능(AI) … 제안요청서.hwpx` 를 픽스처로 썼으나, 이 파일은
**비공개 문서(git 미추적, 커밋 금지)** 다. 테스트 코드만 커밋되고 픽스처는 추가 불가하므로
CI·타 컨트리뷰터·fresh clone 에서 `cargo test` 가 깨진다. 작업지시자 승인에 따라 **git-tracked
공개 샘플**로 교체했다.

교체 대상 선정: 버그 빌드(`get(id)`) vs 정정 빌드(`get(id-1)`)로 263개 tracked 비공개제외 샘플의
전 페이지 SVG 를 렌더·diff 하여, 정정으로 외곽선 렌더가 달라지는 샘플을 추출했다. 결과 2건:
- `samples/exam_social.hwp` p1 — 외곽선 stroke-width 만 변화(0.75→0.5), 폭 <500px (약신호).
- **`samples/k-water-rfp.hwp` p19(index 18)** — 외곽 실선 4변이 정정 빌드에만 존재(강신호) → 채택.

## 신규 테스트
**`nested_table_border_kwater_rfp_p19_outer_outline_present`**
- 대상: `samples/k-water-rfp.hwp` 19페이지(index 18). 외곽 1×1 wrapper 표 + 내부 표 자료 박스.
- 구조 관찰: 내부 표 외곽 격자는 **점선**(`stroke-dasharray`), wrapper 외곽 테두리는 같은 y 에
  겹치는 **실선**으로 그려진다. off-by-one 버그에서는 wrapper borderFill 을 한 칸 어긋나게
  읽어(NONE) 실선 외곽선이 통째로 누락되고 내부 점선만 남는다.
- 가드(좌표 hardcode 회피): 전폭(>500px) 수평선 중 **점선과 y 가 일치(±1px)하는 실선**이 ≥1
  존재하는지 확인. "외곽 박스 = 내부 표 외곽" 관계로 판정하므로 무관한 다른 표의 실선(겹치는
  점선 없음)·페이지네이션 시프트에 영향받지 않는다.

헬퍼 `parse_lines()` 를 `(x1,y1,x2,y2,dashed)` 반환으로 확장(점선 여부 포함).

## 양방향 검증 (가드 유효성)
| 코드 상태 | `kwater_rfp_p19` | `exam_social` |
|-----------|------------------|----------------|
| 정정 적용 | **통과** — 점선과 겹치는 전폭 실선 2건(상 y=583.3, 하 y=993.9, width 621) | 통과 |
| 버그 임시 복원 | **실패** — "겹치는 전폭 실선 0건" | 통과 |

→ 버그 재발 시 즉시 실패하는 유효한 회귀 가드 확인. 검증 후 정정 코드 복원(작업 트리 clean).

## 검증
- `cargo test --test issue_nested_table_border`: **2 passed**
  (exam_social HWP5 기존 + k-water-rfp HWP5 신규, 모두 tracked 픽스처).
- 작업 트리 추적 변경: `tests/issue_nested_table_border.rs` 1건뿐.

## 비고
- 비공개 픽스처 의존 테스트는 폐기. 신규 테스트는 전부 공개 tracked 샘플 기반.
