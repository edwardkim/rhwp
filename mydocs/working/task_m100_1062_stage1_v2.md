# Stage 1 재진단 (v2) — Task #1062: 코드 정독으로 근본 원인 정정

- 브랜치: `local/task1062` (코드 정독·임시 진단 후 전량 revert, 소스 무변경)

## ⚠️ 이전 Stage 1/2/3 정정 — 근본 원인 오귀속

이전 분석은 overflow를 "본문 빈 문단 연속의 trailing_ls 누적"으로 보고 typeset 다단 누적을
vpos-delta로 바꿨으나(Stage 3) **대상 무효 + 비회귀 악화**로 실패했다. 코드 정독 결과 **인덱스
의미를 오해**했음이 드러났다.

## 결정적 사실 (계측 확정)

- IR 본문 문단 = **468개** (`dump`, `DIAG_PLEN: paragraphs.len()=468`).
- 그런데 페이지네이션 결과 PageItem `para_index` 최대 = **1181** (`DIAG_RESULT`), LAYOUT_OVERFLOW
  `para=` 도 468~1168.
- 표 0개. 즉 표 셀 평탄화도 아님.

**원인 코드 — `typeset.rs:1405`:**
```rust
let en_para_idx = paragraphs.len() + st.endnote_paragraphs.len();
```
→ **미주(Endnote) 문단**의 para_index 가 `본문수(468) + 미주문단순번` 으로 매겨진다.
즉 overflow 항목 `para=468~1181` 은 전부 **미주(해설) 문단**이며 본문이 아니다.

- 문서에 미주 컨트롤 **54개**, 미주 문단 약 **714개**(1181−468+1).
- overflow 미주 문단은 page 8~17 에 분포 — 이 시험지는 정답 해설을 미주로 다수 보유.

## 미주 레이아웃 메커니즘 (`typeset.rs:1395-1453`)

1. 미주 문단의 `line_segs.vertical_pos` 를 `endnote_start`(=직전까지 누적 vpos_offset)로 **누적
   offset**(1409). `vpos_offset` 은 미주마다 단조 증가(1430-1432).
2. 페이지네이션: `current_height + height_for_fit > available` 이면 `advance_column_or_new_page()`
   (1440-1443), 이후 `height_for_fit`(다단)/`total_height`(단단) 누적(1448).

→ typeset 은 미주에 페이지 분할을 넣지만, **렌더러(height_cursor)는 미주를 누적된 절대 vpos 로
배치**한다. `VPOS_CORR` 실측: runaway 미주 구간이 `path=lazy base=0` (lazy_base 미확립) 또는
큰 page_base 로, 미주의 고(高) vpos 가 페이지 하단 밖 y 로 매핑 → overflow (end_y 1103~1846 >
col_bottom 1092). **typeset 분할점과 렌더러 vpos 배치가 미주에서 발산.**

## 결론 — 수정 영역 정정

- 대상: **미주(Endnote) 레이아웃** — `typeset.rs` 미주 페이지네이션(1395-1453) + 렌더러
  `height_cursor.rs` 의 미주 vpos→y 기준(base) 처리. **본문 문단 누적이 아님.**
- 핵심 가설: 미주 페이지 분할 시 렌더러 vpos base 가 리셋되지 않아(또는 미주 vpos offset 모델이
  페이지 분할과 불일치) 누적 vpos 가 페이지 밖으로 매핑됨.
- 이전 노선(본문 trailing_ls / vpos-delta 누적)은 **폐기**.

## 다음 (revised Stage 2)

1. 미주 페이지 분할 시 렌더러 vpos base 동작 정밀 측정(VPOS_DEBUG, 미주 para_index 한정):
   분할 직후 base 리셋 여부, end_y 점프 지점.
2. 한컴 정답지(`pdf/3-09월_교육_통합_2022.pdf`)에서 미주 배치(쪽 하단/문서 끝/단 배치) 형태 확인.
3. 미주 vpos→y 모델 수정안 설계 → 페이퍼 검증 → Stage 3.

## 비회귀 확인 대상 (미주 보유 문서)

endnote-01, footnote-01, 그리고 미주/각주 포함 기존 골든 — 미주 레이아웃 변경의 회귀면.
