# Stage 5 보고서 — Task #1073: 회귀 가드 + 최종 검증

- 브랜치: `local/task1073`
- 신규: `tests/issue_1073_nested_table_split.rs`

## 회귀 가드 (kps-ai.hwp, 공개 샘플) — 3/3 통과
- `kps_ai_nested_table_first_chunk_no_overflow`: page 65(첫 조각) text max_y ≤ 페이지 높이
  (758px overflow 회귀 차단).
- `kps_ai_nested_table_continuation_no_overflow`: page 66(연속) 동일.
- `kps_ai_nested_table_split_no_title_duplication`: 첫 조각에 표 제목("소프트웨어사업") 존재 +
  연속 페이지엔 제목 미존재 → 분할 발생 + 전체 재렌더 중복/rowspan 라벨 누수 회귀 차단.

## 최종 검증
- 전수 sweep: baseline 3057 lines / 382815px → **3055 / 382054px** (회귀 0; kps-ai 758
  overflow 해소, hwpctl_ParameterSetID 보너스 개선).
- 골든 SVG **8/8**, `cargo test --release` 전체 0 failed(lib 1324 + 통합, 신규 3 포함),
  clippy clean, fmt clean.

## 잔여 (known limitation)
- 중첩 표 분할 break row 가 한컴(PDF) 대비 ~2 중첩행 늦음(available-height/overhead 측정
  정밀도). 콘텐츠/구조 정합·overflow 0 달성, break 정확 일치만 잔여 → 별도 정밀화 대상.
- 범위 외(atom 폴백 유지): 2단계+ 중첩, 텍스트 동거 문단의 중첩 표.

## 결론
중첩 표(셀 내부 표)가 페이지보다 클 때 중첩행 단위 페이지 분할 구현 완료 — kps-ai 758px
overflow 해소, 한컴 PDF 구조 정합, 비회귀 0. 최종 보고서 → PR.
