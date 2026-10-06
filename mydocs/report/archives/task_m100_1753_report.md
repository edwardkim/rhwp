# Task #1753 최종 보고서 — 지연 자리차지 표의 후속 텍스트 선행 채움 (부분)

## 요약
hwpdocs 페이지·PI 대조 S3 심층조사(한글 PDF 시각 대조)에서 확정한 실결함 수정.
visible-host 자리차지(TopAndBottom·vert=Para) RowBreak 표가 다음 쪽으로 이월될 때,
한글은 후속 텍스트를 현재 쪽 잔여 공간에 선행 채움(fill-before-deferred-float)하는데
rhwp 는 표 뒤로 밀어내던 문제. 대표 2814765: pi52/53 이 11쪽 → **9쪽**(한글 정합),
mismatch 3→1 (잔여는 캐럿-개체 분리 도구 한계).

## 원인
`typeset_block_table` 의 다행 RowBreak 이월(multirow_clean_defer → advance)이 표만
다음 쪽으로 보내고, 순차 모델이라 후속 문단은 fragment 뒤에서만 배치.

## 수정
`src/renderer/typeset.rs`:
- `prefill_before_deferred_table` — 이월 직전, 후속 control-free 문단들을 현재 쪽 잔여
  공간에 선행 배치(FullParagraph). 가드: 단일 단 + 텍스트 anchor + `is_para_topbottom_float`
  + v_off≥0 + RowBreak; 후보는 저장 첫 실줄 vpos ∈ (host vpos, 본문높이HU] (같은 쪽 연속
  인코딩 — 누적좌표 문서 자연 배제) + 누적높이 fit, 최대 8개.
- `TypesetState.prefilled_paras` + 메인 루프 스킵, `typeset_block_table` 슬라이스 플럼빙.

## 검증
| 항목 | 결과 |
|------|------|
| 재현·코퍼스 원본 (한글 OLE) | n_mismatch 3→**1** (pi52/53 해소, 21=21쪽) |
| cargo test --lib | 2051 passed / 0 failed |
| 통합 9크레이트 (신규 issue_1753 포함) | 102 passed / 0 failed |
| 페이지 게이트 (국제고속선기준 251 포함 8종) | 무회귀 |
| 코퍼스 mismatch / MATCH 표본 150 | 악화 0 / 150 유지 |
| rustfmt / clippy --lib | 통과 |

## 한계 / 후속
- **부분 수정**: pi51 host 제목 줄은 여전히 표 첫 fragment 쪽(10쪽) 렌더(한글은 9쪽).
  layout "분할 표 첫 부분 호스트 텍스트" 렌더 억제 신호(ColumnItemCtx 확장) 설계가 필요해
  후속 분리 — pi51 의 PI 판정도 캐럿-개체 분리로 잔존.
- 선행 채움은 control-free 문단 한정(표/그림 포함 후속은 순차 유지), 최대 8개.

## 산출물
- 소스: `src/renderer/typeset.rs` (+ `tests/issue_1753_deferred_table_fill_ahead.rs`)
- 재현: `samples/task1753/deferred_takeplace_fill_ahead.hwpx` + README
- 검증 TSV: `output/poc/hwpdocs_pipage/{bad39,match150}_recheck_1753.tsv`
- 시각 증거: `output/poc/hwpdocs_pipage/pdf/2814765_hangul_p{9,10}.png` (로컬)
