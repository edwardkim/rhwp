# 최종 결과보고서 — Task #1219

**제목**: 수식 포함 줄의 본문 한글 압축·겹침 — 거짓 오버플로우 해소
**이슈**: [#1219](https://github.com/edwardkim/rhwp/issues/1219) (M100 / v1.0.0)
**브랜치**: `local/task1219`
**작성일**: 2026-06-01

---

## 1. 문제

`samples/3-09월_교육_통합_2023.hwp` 6쪽 문26 「공차가 양수인 등차수열 {aₙ}과 등비수열
{bₙ}에 대하여」에서 본문 한글이 겹쳐 렌더링. 페이지 전체에서 **인라인 수식(treat-as-char)이
포함된 줄만** 한글이 압축됨(문24/26/27/30). 순수 텍스트 줄(문28 등)은 정상.

| | 한글 advance | |
|--|--|--|
| PDF (한글 2022, 정답지) | ≈ 12px (1.0em 전각) | 정상 |
| rhwp (수정 전) | 8.96px (0.746em) | 겹침 |

작업지시자 최초 표현("수식 폰트 크기")과 달리, **수식 글리프가 아니라 수식이 든 줄의 본문
한글**이 압축되는 문제였다.

## 2. 근본 원인

문단 IR 은 align=Left, 자간 0%, 장평 100% 로 압축 의도가 없으나, 렌더러의 비정렬 오버플로우
압축 분기(`paragraph_layout.rs`)가 `total_text_width > available_width` 로 오판해 음수 자간을
적용했다. 줄 폭(`total_text_width`)이 두 경로에서 과대계상된 것이 원인:

1. **줄 경계 수식 오포함** — `est_x` 측정 루프와 `total_tac_width_in_line` 이 전역
   `tac_offsets_px` 를 run 경계(`is_last_run_est_tac && pos == run_char_end_est`,
   `pos <= line_end`)로 재필터 → 줄 끝 위치(= 다음 줄 선두)의 수식을 현재 줄에 포함.
   문26 라인0 에 다음 줄 `a₁=b₁=1`(55px)이 합산(392px). 렌더 경로는 이 수식을 다음 줄에
   정상 배치 → **측정/렌더 불일치**.
2. **선두 미주 마커 이중계상** — 문단 선두 미주("문26)")가 `endnote_marker_x_advance` 로
   풀사이즈 선두 마커 렌더 + 폭(44px)을 `inline_offset` 에 반영(available 차감)되는데,
   `footnote_positions` 측정 루프가 **같은 미주**를 위첨자(fn_text, ~22px)로 `est_x` 에 다시
   가산. 렌더는 인라인 위첨자를 그리지 않음(문26 "공" x=78) → **측정/렌더 불일치**.

## 3. 수정

`src/renderer/layout/paragraph_layout.rs` — 측정 경로를 렌더 경로의 줄-경계 의미에 일치.

| # | 변경 | 효과 |
|---|------|------|
| 1 | `est_x` 측정 루프 TAC 소스: `tac_offsets_px` → `line_tac_offsets`(= `tac_offsets_for_line`) | 줄 끝 경계 수식 오포함 제거 |
| 2 | `total_tac_width_in_line`: `pos <= line_end` 필터 → `line_tac_offsets` 폭 합산 | 동일 결함 제거 |
| 3 | `footnote_positions` 측정 루프: `start_line==0` 의 `Control::Endnote`(선두 미주) 제외 | 미주 이중계상 제거 |

모두 "측정이 렌더와 동일한 줄-경계/마커 규칙을 공유"하도록 통일하는 변경.

## 4. 결과 (본문 한글 advance, PDF 목표 12px)

| 줄 | 원본 | 수정1·2 (TAC) | 수정3 (+미주) |
|----|------|--------------|--------------|
| 문26 공차가… | 8.96px | 11.08px | **11.93px** ✅ |
| 문27 함수… | 10.27px | 10.27px | **11.85px** ✅ |
| 문24 매개변수… | 8.77px | 10.25px | 11.30px |
| 문30 길이가… | 9.15px | 10.52px | 11.28px |

- 문26(지적 줄)·문27: ≈12px 정합, 시각 겹침 완전 해소(`output/poc/eq26_1b/cmp_q26_1b.png`).
- 문24/30: 8.77/9.15 → 11.3px 대폭 개선. 잔여 ~0.7px 미세 압축은 인라인 수식 폭 측정의
  별개 미세 오차로 추정(육안 겹침 없음) — 본 이슈(거짓 오버플로우·겹침) 범위 밖.

## 5. 검증

- `cargo test --release` → **1896 passed / 0 failed**.
- 골든 SVG 스냅샷 8건(table_text, issue_157/267/677, form_002, issue_147,
  **issue_617 exam_kor p5**, 결정성) 통과 → 인라인 TAC·각주·목차·우측탭 회귀 없음.
- 회귀 가드 추가: `tests/issue_1219_equation_line_hangul_advance.rs` — 문26 줄 한글 최소
  advance ≥ 11.0px 단언(수정 전 8.96px 면 실패).
- 경계 케이스(마지막 줄 끝 수식)는 `line_tac_offsets` 가 `composed_line_char_end` 의
  마지막-줄 분기로 렌더와 동일 처리 → 회귀 없음. 미주 수정은 선두 미주로 한정.

## 6. 커밋

| 커밋 | 내용 |
|------|------|
| `d55e3ae5` | Stage 1: 측정 루프 TAC 소스 `line_tac_offsets` 통일 |
| `db9bf95f` | Stage 1b+2: 선두 미주 측정 이중계상 제거 + 회귀 검증 |
| (Stage 3) | 회귀 테스트 + 최종 보고서 |

## 7. 비고

- `orders/` 갱신은 작업지시자 지시 없어 생략(메모리 룰 `feature_orders_no_update`).
- 문24/30 잔여 미세 압축을 PDF 와 픽셀 단위까지 맞추려면 인라인 수식 폭 측정 정밀화가
  필요하며, 이는 별도 이슈로 추적 권장(본 이슈의 겹침은 해소됨).
