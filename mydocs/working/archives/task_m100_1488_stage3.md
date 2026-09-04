# Task M100 #1488 Stage 3 — 검증 및 ROOT B 스코핑

- 브랜치: `local/task_m100_1488`
- 작성일: 2026-06-25
- 단계: Stage 3 (검증 + 잔여 결함 판정)

## 1. 시각 검증 (한글 2024 PDF 18페이지 대조)

`samples/rowbreak-problem-pages.hwpx` 18페이지 전수를 정답지
`pdf/rowbreak-problem-pages-2024.pdf`(18페이지)와 컨택트시트로 대조.

- 페이지 수 18 일치, **내용 손실 없음** (모든 표/도식/본문 존재).
- 깨진 페이지·가득 빈 페이지·심각한 텍스트 겹침 **없음**.
- 이슈 결함 처리:
  - 17~22p 여분 빈 페이지 → **제거** (22→18p).
  - 2p 본문·도식 겹침 → **해소** (PDF 구조와 일치).
  - 16p 상단 컷오프(빈 페이지 인접) → **해소**.
  - pi=28 오버레이 셀(다이어그램) → 3페이지(15~17)로 정상 분할, 빈 페이지 없음.

## 2. ROOT B (표 하단 분할 overflow) — 별도 후속 권장

잔여 `LAYOUT_OVERFLOW`(하단 마진 스필):

| 위치(sec/page-in-sec/para) | type | overflow |
|------|------|----------|
| 0/2,0/3 para11 | PartialTable | 16.8 / 18.1px |
| 0/6 para21 | PartialTable | 2.7px |
| 1/1 para5 | PartialTable | 32.7px |
| 1/2 para11 | Table / PartialTable | 42.5 / 76.2px |
| 1/7 para28 | PartialTable | 11.7px (pi=28 분할 위치 이동분) |

판정:

- **모두 기존(clean devel) 문제** — 이슈 본문의 "clean devel" overflow 로그와 동일
  (단, Stage 2 가 sec1 p14 para28 13.5px overflow 를 해소).
- **내용 손실 없음** — 초과 fragment 의 나머지는 다음 페이지로 정상 연속
  (pi=5: 10→11p, pi=11 ci=1: 11→12p). 하단 마진으로 수십 px 스필하는 **표시 위치 문제**.
- 근본은 한컴의 "표를 다음 페이지로 밀지 vs 부분 배치" 페이지 하단 분할 패리티
  (`typeset.rs` Task #1025/#1086/#1486/#1105 계보)로, 단일 게이트 수정이 다수 샘플
  회귀를 유발할 수 있는 고위험 영역. [[tech_trailing_model_no_ssot]] 교훈상 본 이슈의
  핵심(여분 빈 페이지) 수정과 분리하여 별도 이슈로 다루는 것이 안전.

→ **권장**: ROOT B 를 별도 후속 이슈(표 하단 분할 overflow 패리티)로 등록. 본 타스크는
  이슈 #1488 의 헤드라인 결함(여분 연속 페이지 + 겹침)을 정답지 페이지 수 일치로 해소.

## 3. 회귀 테스트

- 단위: `test_advance_row_cut_empty_overlay_reset_no_hard_break` (mechanism 가드).
- 통합: `tests/issue_1488_rowbreak_empty_overlay_pages.rs` — 페이지 수 18 +
  pi=28 표 ≤4페이지 분할 가드. 통과.
- 전체 `cargo test` 1938(lib) + 통합 전부 통과, `hwpx_roundtrip_baseline` 4/4 통과.
