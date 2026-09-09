# Task #1753 Stage 2 완료보고서 — 선행 채움(prefill) 구현 + 테스트

## 수행 내용 (`src/renderer/typeset.rs`)
- `TypesetState.prefilled_paras: HashSet<usize>` 추가 + 메인 루프 진입부 스킵.
- `typeset_block_table` / `typeset_table_paragraph` 에 전체 슬라이스(paragraphs/composed) 플럼빙
  (호출부: 메인 루프, flush_deferred_table_controls).
- `prefill_before_deferred_table` 신설 — 이월 분기(multirow_clean_defer 등 advance 직전) 호출:
  - 가드: 단일 단 + current_items 비어있지 않음 + 텍스트 anchor + `is_para_topbottom_float`
    (자리차지·vert=Para) + v_off≥0 + RowBreak.
  - 후보: controls 없음 + 첫 실줄 저장 vpos ∈ (host vpos, 본문높이HU] (같은 쪽 연속 인코딩,
    누적좌표 문서 자연 배제) + 누적높이 fit(안전마진 4px). 최대 8개, 첫 실패 중단.
  - 배치: FullParagraph + flow 높이 소비(trim_spacing_before 는 기존 규칙 재사용).

## 구현계획서 대비 조정
- prefill 후보 판정 "단위테스트"는 판정이 상태(fit/레이아웃) 의존이라 순수 함수 분리가
  인위적 — 통합테스트(`tests/issue_1753_deferred_table_fill_ahead.rs`)로 대체.

## 검증
- dump-pages: pi52/53 → **9쪽** (한글 정합), 표 fragment 10~11쪽, pi54 표 뒤 11쪽 유지,
  총 21쪽 불변.
- SVG 9쪽: pi53 글리프 y≈1050.7 (본문 끝 1065.8 내) — 하단 선행 채움 렌더 확인.
- 통합테스트 통과, cargo check/fmt 통과.

## 상태
완료. Stage 3 (회귀 검증 + 최종보고) 진행.
