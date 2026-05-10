# Task #768 Stage 1 (RED) 완료 보고서

**Issue**: [#768](https://github.com/edwardkim/rhwp/issues/768)
**Stage**: 1 — TDD RED
**작성일**: 2026-05-10
**브랜치**: `local/task768` (stream/devel 베이스)

---

## 산출물

- **신규 회귀 테스트**: `tests/issue_768.rs`
- **단언**: pi=94 ("<편집 화면 분할에서>") 가 페이지 인덱스 2 (3쪽) 에 등장 — PDF 권위 (한글 2022) 정합

```rust
assert_eq!(page_idx, 2, "pi=94 가 PDF 권위(2)와 불일치");
```

## 테스트 실행 결과 (RED — 의도된 FAIL)

```
$ cargo test --test issue_768 -- --nocapture

[issue_768] pi=94 등장 페이지 인덱스 = 3 (page_count=8), PDF 권위 = 2

panicked at tests/issue_768.rs:57:5:
assertion `left == right` failed: pi=94 가 page_index=3 에 등장.
PDF 권위(한글 2022) 정합 = 2 (3쪽).
column-break 가 다단 영역 마지막 단에서 발생할 때 wrap-around 안 되고
페이지 break 강제하는 결함.
  left: 3
 right: 2

test issue_768_pi94_appears_on_page3_not_page4 ... FAILED
```

→ 결함 정확 검출:
- pi=94 가 페이지 인덱스 3 (4쪽) 에 등장 — 결함
- PDF 권위 자료 (`pdf/basic/shortcut-2022.pdf`) 페이지 3 끝에 "<편집 화면 분할에서>" + "화면 이동 Ctrl+W,N" 1행 — 정합
- 차이: column-break 가 마지막 단에서 발생 시 페이지 break 강제

## 부수 관찰

테스트 실행 중 stderr 에 다른 결함 메시지 출현 (본 task 와 무관, Task #716 / #332 / #452 영역):

```
LAYOUT_OVERFLOW: page=3, col=0, para=152, ... overflow=20.0px
LAYOUT_OVERFLOW_DRAW: section=0 pi=153 line=0 ... overflow=13.3px
LAYOUT_OVERFLOW: page=3, col=1, para=153, ... overflow=20.0px
```

→ shortcut.hwp 4쪽 (page_index=3) 에서 별개 LAYOUT_OVERFLOW. 본 task 비범위.

## 베이스라인 환경

- 브랜치: `local/task768` (stream/devel 베이스, Task #716 미적용)
- page_count = 8
- 결함 페이지 인덱스: 3 (= 페이지 4)
- 정합 페이지 인덱스: 2 (= 페이지 3, PDF 권위)

## 다음 단계 (Stage 2 — 분석)

1. `RHWP_TASK768_DEBUG=1` instrument 추가:
   - `typeset.rs:417` (column-break 분기) — pi/column_type/has_diff_col_def/col_count/current_column/current_height
   - `typeset.rs:261-273` (advance_column_or_new_page) — action(advance/push_new_page)
2. `dump-pages -p 2` 또는 `export-svg -p 2` 로 trace 수집
3. pi=94 column-break 진입 확인 → push_new_page 호출 추적
4. H1 wrap-around 적용 시 동작 시뮬레이션
5. Stage 3 GREEN 진입 승인 요청

## 승인 요청

Stage 1 RED 완료. Stage 2 (분석) 진행.
