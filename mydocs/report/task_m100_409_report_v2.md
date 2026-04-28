# 최종 결과 보고서 v2 — Task #409

## 1. 타스크 요약

- **이슈**: [#409](https://github.com/edwardkim/rhwp/issues/409)
- **마일스톤**: M100 (v1.0.0)
- **브랜치**: `local/task409` (v1) + `local/task409_v2` (잔여 페이지네이션 수정)
- **단계**: v1 Stage 1~3 (이미 devel 머지) + v2 Stage 4~5 (본 보고서)

## 2. 통합 변경 요약

### v1 — Layout 측 vpos 보정 가드 확장
`src/renderer/layout.rs:1366-1390` `prev_has_overlay_shape` 가드를 `Control::Picture` (non-TAC) + `TopAndBottom + vert=Para` 케이스로 확장. 21페이지 2x1 표가 차트 바로 아래로 정상 위치 복원.

### v2 — Pagination 측 chart 높이 누적
`src/renderer/typeset.rs:622-672` controls 루프에서 비-TAC + TopAndBottom + vert=Para 인 Picture/Shape 의 `common.height + margin.bottom` 을 `current_height` 에 누적. 22페이지에 누락되었던 pi=191 헤딩 + pi=192 (10x5 표) 정상 표시.

## 3. 수정 전/후 비교

### 21페이지 / 22페이지 SVG (PDF 대조)

| 항목 | v0 (수정 전) | v1 후 | **v2 후** |
|------|------------|-------|----------|
| 21페이지 차트 위치 | y=94.5 | y=94.5 ✓ | y=94.5 ✓ |
| 21페이지 2x1 표 | y=937 (페이지 하단, 잘림) ❌ | y=532 (차트 직하) ✓ | y=532 ✓ |
| 22페이지 (4) 헤딩 | 누락 ❌ | 누락 ❌ | **표시** ✓ |
| 22페이지 10x5 표 | 누락 ❌ | 누락 ❌ | **표시** ✓ |
| 22페이지 연령대별 차트 | 차트만 표시 | 차트만 표시 | 표 + 차트 표시 ✓ |

### LAYOUT_OVERFLOW (대상 샘플 전체)

| 단계 | 건수 | 잔여 |
|------|------|------|
| v0 | 22 | page=2/20/27 다수 |
| v1 | 4 | page=2 449.2 / page=20 247.9 / page=27 15.0+111.9 |
| **v2** | **1** | page=2 449.2 (본 작업과 무관한 기존 결함) |

→ chart 관련 모든 overflow 해소.

## 4. 회귀 검증

### 전체 테스트 (cargo test --release)

11개 테스트 스위트 100% 통과:

| Suite | 결과 |
|-------|------|
| `lib` | **1023 passed**, 0 failed, 1 ignored |
| `svg_snapshot` | **6 passed**, 0 failed |
| `composition_alpha` | 14 passed |
| `find_replace_engine` | 25 passed |
| 기타 7 suites | 0~25 passed each |
| **합계 실패** | **0** |

### 6개 다른 샘플 LAYOUT_OVERFLOW 무회귀

| 샘플 | v0 | v1 | v2 |
|------|----|----|----|
| `biz_plan.hwp` | 0 | 0 | 0 |
| `exam_kor.hwp` | 7 | 7 | 7 |
| `exam_math.hwp` | 0 | 0 | 0 |
| `aift.hwp` | 1 | 1 | 1 |
| `k-water-rfp.hwp` | 0 | 0 | 0 |
| `kps-ai.hwp` | 4 | 4 | 4 |
| `2025년 기부·답례품_양식.hwpx` | 22 | 4 | **1** |

## 5. 변경 파일 (통합)

- `src/renderer/layout.rs` — `prev_has_overlay_shape` 가드 확장 (v1)
- `src/renderer/typeset.rs` — controls 루프 chart 높이 누적 (v2)

## 6. 산출물

- v1: `task_m100_409.md`, `task_m100_409_impl.md`, `task_m100_409_stage1~3.md`, `task_m100_409_report.md`
- v2: `task_m100_409_v2_impl.md`, `task_m100_409_stage4~5.md`, `task_m100_409_report_v2.md` (이 파일)
- PR 초안: `task_m100_409_pr.md` (v1+v2 통합)

## 7. 결론

- **사용자 지적 해결**: 21페이지 2x1 표 위치 + 22페이지 (4) 헤딩/10x5 표 누락 모두 PDF 일치하게 복원
- **회귀 무**: 1023 lib + 6 svg_snapshot + 9개 통합 테스트 100% 통과, 6개 다른 샘플 무회귀
- **개선**: 타겟 샘플 LAYOUT_OVERFLOW 22 → 1 (-21)
- **잔여**: page=2 PartialParagraph 449.2px (본 작업과 무관, 기존 결함, 별도 이슈 권장)

이슈 클로즈 + PR 생성 승인 요청.
