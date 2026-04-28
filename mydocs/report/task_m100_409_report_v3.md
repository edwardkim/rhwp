# 최종 결과 보고서 v3 — Task #409

## 1. 타스크 요약

- **이슈**: [#409](https://github.com/edwardkim/rhwp/issues/409)
- **마일스톤**: M100 (v1.0.0)
- **단계**: v1 (Stage 1~3) + v2 (Stage 4~5) + v3 (Stage 6~7)

## 2. 통합 변경 (3 layer)

### v1 — Layout vpos 보정 가드 확장
`src/renderer/layout.rs` `prev_has_overlay_shape` 가드를 `Control::Picture` (non-TAC) + `TopAndBottom + vert=Para` 케이스로 확장. 21페이지 2x1 표 정상 위치.

### v2 — Pagination chart 높이 누적
`src/renderer/typeset.rs::typeset_section` controls 루프에 비-TAC + TopAndBottom + vert=Para Picture/Shape 의 `height + margin.bottom` 을 `current_height` 에 누적. 22페이지 (4) 헤딩 + 10x5 표 정상 표시.

### v3 — Atomic TAC top-fit 시멘틱
`src/renderer/typeset.rs::typeset_paragraph` 에 단일 라인 + TAC Picture/Shape 항목의 top-fit 분기 추가. 23페이지 차트 정상 배치.

## 3. 사용자 지적 모두 해결

| 페이지 | 결함 | v0 | v1 | v2 | **v3** |
|--------|------|----|----|----|--------|
| 21 | 2x1 표가 차트 직하 위치 | 잘림 ❌ | **정상** ✓ | 정상 ✓ | 정상 ✓ |
| 22 | (4) 헤딩 + 10x5 표 | 누락 ❌ | 누락 ❌ | **표시** ✓ | 표시 ✓ |
| 23 | 막대 차트 하단 배치 | 누락 ❌ | 누락 ❌ | 누락 ❌ | **표시** ✓ |
| 24 | 정상 후속 콘텐츠 시작 | 차트로 시작 ❌ | 차트로 시작 ❌ | 차트로 시작 ❌ | **2x1 표→(6) 헤딩→파이차트** ✓ |

## 4. 검증

### LAYOUT_OVERFLOW (대상 샘플 전체)

| 단계 | 건수 |
|------|------|
| v0 | 22 |
| v1 | 4 |
| v2 | 1 |
| **v3** | **1** (page=2 PartialParagraph 449.2px — 본 작업 무관 기존 결함) |

### 전체 테스트 (cargo test --release)

11개 스위트 100% 통과:
- `lib`: **1023 passed**
- `svg_snapshot`: **6 passed**
- 기타 9 suites: 모두 0 failed

### 6개 다른 샘플 LAYOUT_OVERFLOW 무회귀

| 샘플 | v0 | v1 | v2 | v3 |
|------|----|----|----|----|
| `biz_plan.hwp` | 0 | 0 | 0 | 0 |
| `exam_kor.hwp` | 7 | 7 | 7 | 7 |
| `exam_math.hwp` | 0 | 0 | 0 | 0 |
| `aift.hwp` | 1 | 1 | 1 | 1 |
| `k-water-rfp.hwp` | 0 | 0 | 0 | 0 |
| `kps-ai.hwp` | 4 | 4 | 4 | 4 |

## 5. 변경 파일 (통합)

- `src/renderer/layout.rs` — vpos 가드 확장 (v1)
- `src/renderer/typeset.rs` — chart 높이 누적 (v2) + atomic TAC top-fit (v3)

## 6. HWP 시멘틱 정리 (v1+v2+v3 일관성)

| 측면 | 시멘틱 |
|------|--------|
| Layout vpos 보정 | 그림 다음 문단의 vpos 에 그림 높이 반영 → 보정 시 이중 점프 방지 (v1) |
| Pagination 높이 산정 | 그림이 본문을 미는 만큼 current_height 에 누적 → layout y 와 일관화 (v2) |
| Pagination fit 판정 | 분할 불가 atomic TAC 항목은 시작점 fit 으로 판정 (하단 여백 흘림 허용) (v3) |

## 7. 결론

- 21~24 페이지 PDF 일치 복원 (사용자 지적 전건 해결)
- 11개 테스트 스위트 100% 통과, 6개 다른 샘플 무회귀
- HWP layout/pagination 시멘틱 3-layer 일관화

이슈 클로즈 + 머지 + push + PR 초안 v3 업데이트 진행 승인 요청.
