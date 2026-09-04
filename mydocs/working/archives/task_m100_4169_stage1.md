---
kind: working
status: completed
issue: 4169
last_verified: 2026-08-08
---

# Task #4169 Stage 1 - 단일줄 과밀 판정 memo

## 구현

- `Paragraph.single_line_overflow_memo: AtomicU64` — 0=미판정, `(f32 폭 bits << 1)|overflowed`
  패킹, Relaxed. `Cell`은 `RenderNormalizedSection.paragraphs: Arc<Vec<Paragraph>>` 경유
  `DocumentCore` Send 단언이 `Paragraph: Sync`를 요구해 컴파일 불가 — Atomic 선택 근거.
  Clone은 수동 impl(파생 캐시를 텍스트와 함께 원자적으로 복제 — undo 스냅샷 정합).
- 가드는 memo 히트 시 `estimate_composed_line_width` 측정만 생략하고, overflowed=true의
  fresh 재래핑은 매 빌드 그대로 수행한다.
- 무효화 배선(전수): 모델 프리미티브 8곳(insert_text_at/delete_text_at/split_at/merge_from/
  apply_char_shape_range/set_single_char_shape/replace_style_char_shape_preserving_overrides/
  shift_for_inline_control_insert), `reflow_line_segs` 수렴점, 직접 대입 우회 5곳 —
  table_ops 표 계산식 셀 기록, document.rs clear_initial_field_texts,
  object_ops/common.rs `reflow_paragraph_line_segs_after_control_delete` 서두(셀 인라인
  그림/도형/수식/각주 삭제 가족 일괄 — 적대 리뷰 CONFIRMED 결함 수정),
  field_query.rs `rebuild_char_offsets` 서두(빈 필드 제거 분기 — 감사 중 발견한 동류 누락),
  clipboard.rs `strip_structural_controls_for_text_clipboard`(clip 사본이 다중 문단 붙여넣기로
  문서에 스플라이스될 수 있음).

## 검증 결과

- `--lib composer`: 81 passed / 0 failed (신규 5: 메모 히트 측정 생략 증명, over=true 재래핑
  유지, 폭 키 불일치 재판정, fit 판정 메모, 뮤테이션 경로 무효화).
- `--lib issue4149`: 신규 `issue4149_cell_picture_delete_clears_single_line_overflow_memo`
  (실명령 경로: stale verdict 주입 → 그림 삭제 → 미판정 어서션, 결함 분기 branch-1 통과) 포함 pass.
- `--lib object_ops`: 59 passed / 0 failed. 기존 가드 회귀 핀 issue_2287(2)/2430(2)/2525(1)/
  2527(3)/4138(2) 전부 pass.
- 적대 리뷰: 무효화 누락 사냥에서 셀 그림 삭제 CONFIRMED 1건(위 수정 반영). undo/redo 스냅샷
  클론 정합·f32 폭 키 정밀도(ulp vs 최소 리사이즈 3자리 차)·단일 스레드 TOCTOU 불가·ls==1
  왕복 무해 — 반박 실패. 관찰: 향후 in-place CharShape 편집 API 도입 시 폭 단독 키가 구멍이
  된다(현재 사이트 없음 확인).
- 실측(거대 셀 문서, release-test): `build_page_tree` 17.6ms → 13.7ms (−22%), 캐럿 질의
  ≈17.2 → ≈14.3ms.
