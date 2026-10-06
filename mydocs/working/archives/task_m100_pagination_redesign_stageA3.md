# Stage A3 완료 보고 — 1×1 중첩셀 콘텐츠 페이지 분할 (#2007 실동작 수정)

- 브랜치: `task/2019-repursue-impl` / 앵커: #2007 42065

## 결과

**42065: 6p → 15p (한글 17p), 콘텐츠 정상 분할·판독 가능. 무회귀 0.**

- pi-page 오라클: PAGE_DELTA 15 vs 17, **n_mismatch=5**(수정 전 대량 크램 대비 대폭 개선).
- 시각(export-png): 이전 크램(판독 불가) → p6/p7이 콘텐츠를 이어받아 정상 분할·판독 가능(중복·크램 없음).

## 수정 (기존 메커니즘 재사용 — 모델 확장 불필요)

Stage A2에서 "CellUnit 모델 확장 필요"로 우려했으나, **기존 `nested_table_mixed_fragment_heights`
(단일 행 중첩 표를 페이지 분할 fragment 로 분해) + mixed_nested_fragment 렌더 지원이 이미 존재**함을
발견. 텍스트+중첩표 문단(4991 블록)엔 쓰이나 **빈-텍스트+1×1 중첩표(4924 블록)엔 미적용**이었을 뿐.

- `table_layout.rs:4933` else-if 분기 추가: `nested_tables[0].row_count == 1` 인 빈-텍스트 문단도
  `nested_table_mixed_fragment_heights` 로 fragment 유닛 산출 → `advance_row_cut` 가 페이지 경계 분할.
- **게이트**: `frags.len() > 1 && total_frag_h > 1000px` — 콘텐츠가 명백히 한 페이지 초과할 때만.
  한 페이지에 맞는 소형 1×1 중첩 표(서식 등)는 기존 atomic 유지(fragment 렌더 미세차로 form-002
  스냅샷 회귀 → 게이트로 차단).

수정: `src/renderer/layout/table_layout.rs` (+62줄, 1파일).

## 검증 (무회귀 0)

- `cargo test --lib` **2145/0**. `hwpx_roundtrip_baseline` 4/4.
- 표 분할 통합: issue_1488_rowbreak·issue_1749·issue_2015 전건 통과.
- `svg_snapshot` **8/0**(form-002 게이트로 해소), `opengov_corpus_snapshot` 2/0.
- **랜덤 코퍼스 250문서 페이지수 변동 0** → 대형 1×1 중첩 표에만 국소 작용.
- 회귀테스트 `tests/issue_2007_nested_cell_pagination.rs` + 픽스처
  `samples/basic/issue2007_nested_cell_pagination_42065.hwp`(roundtrip PASS).

## 잔여 (15 vs 17)

pi3-7이 +1 시프트(fragment 경계가 한글과 미세 불일치). 콘텐츠는 전부 정상 배치·판독 가능하며,
2쪽 차이는 fragment 높이/경계 튜닝(refinement)이지 크램 붕괴가 아님. 후속 정밀화 여지.

## 의의

**deep-class(intra-cell) 첫 실동작 수정.** A1(통일 근본원인)·A2(측정 정확·갭 특정)의 정밀 분석이
"기존 메커니즘 재사용"이라는 저위험 해법으로 이어짐(우려한 모델 확장 불필요). #1995(1x1 단일셀 표
콘텐츠 미분할)·#2006 일부와 동일 클래스에 적용 가능성 — 후속 확장 스코프.
