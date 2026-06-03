# Stage 4 보고서 — Task #1079: 회귀 가드 + 최종 검증

- 브랜치: `local/task1079`
- 신규: `tests/issue_1079_picture_pushdown_vpos.rs`

## 회귀 가드 (pr-149.hwp, 공개 샘플) — 2/2 통과
- `pr149_single_page`: page_count == 1 (그림 pushdown 이중 계상 2페이지 분리 회귀 차단).
- `pr149_content_within_page_and_complete`: page0 의 `<text>`/`<image>` 하단이 페이지 높이 이내
  + 본문 마지막 "입니다." 존재(누락 없음).

## 최종 검증
- pr-149: 1페이지, overflow 0, 라벨→그림→라벨→입니다 순서·본문 내 배치(PDF p1 정합).
- 전수 sweep: baseline 3057 lines / 382815px / 97파일 → **3056 / 382705 / 96파일**
  (pr-149 소멸=해소, **회귀 0**, #409 계열 보존).
- 골든 SVG **8/8**, `cargo test --release` 전체 0 failed(lib 1324 + 통합, 신규 2 포함),
  clippy/fmt clean.

## 결론
비-TAC TopAndBottom 그림의 pushdown ↔ 파일 vpos 이중 계상 정정 완료. pr-149 1페이지 수용.
최종 보고서 → PR.
