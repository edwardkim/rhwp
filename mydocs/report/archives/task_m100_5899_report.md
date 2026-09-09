# task_m100 #5899 처리 결과 — 양쪽정렬 꼬리말 쪽번호가 종이 밖으로 밀려나던 결함

- 이슈: [#5899](https://github.com/edwardkim/rhwp/issues/5899)
- 대상 문서: `samples/hwp3-sample11-hwpx.hwpx`
- 정답지: `pdf/hwp3-sample11-2020.pdf` (한글 2020, 151쪽)
- 기준 커밋: `72674c565` (origin/devel)
- 변경 파일: `src/renderer/layout/paragraph_layout.rs` (1곳), `tests/cases/issue_5899_footer_page_number_justify.rs` (신규)

## 1. 증상

`hwp3-sample11-hwpx.hwpx` 를 렌더하면 **151쪽 중 150쪽에서 꼬리말 쪽번호가 보이지 않는다.**
글자가 지워진 것이 아니라 종이 밖 `x ≈ 20,163~20,657 px` (종이 폭 793.71 px)에 그려진다.
같은 줄의 `DCT Technology Inc.` 도 본문 폭 전체로 흩어진다.

전수 스윕(151쪽 `export-svg` 후 `<text x>` 최댓값과 종이 폭 대조):

| | 종이 밖 글자가 있는 쪽 |
|---|---|
| 수정 전 | **150 / 151** (1쪽은 표지라 꼬리말 없음) |
| 수정 후 | **0 / 151** |

## 2. 정답지 대조 (p116)

`PyMuPDF get_text("words")` (pt) vs rhwp SVG (px, 96dpi → ×0.75 로 pt 환산):

| 조각 | 한글 2020 | 수정 전 rhwp | 수정 후 rhwp |
|---|---|---|---|
| `DCT` | 57.00 pt | 56.70 pt | 56.70 pt |
| `Technology` | 81.00 pt | 270.23 pt | 78.15 pt |
| `Inc.` | 139.00 pt | 514.05 pt | 129.90 pt |
| **`115`** | **523.00 pt** | **15,122.10 pt** | **523.58 pt** |

꼬리말 본문 폭은 `<hp:subList textWidth="48192">` = 481.92 pt, 오른쪽 끝은 57.00 + 481.92 =
538.92 pt. 수정 후 쪽번호 시작점 523.58 pt 는 정본 523.00 pt 와 **0.58 pt(0.8 px)** 차이다.
`Technology`·`Inc.` 의 남은 3 pt 안팎 차이는 대체 폰트 advance 차이(이 이슈 범위 밖)다.

## 3. 원인

`Contents/section0.xml` 의 꼬리말 둘째 문단은
`DCT Technology Inc.` + 공백 74개 + `<hp:autoNum numType="PAGE">` 이고, 문단 정렬은
`paraPr id="3"` 의 `JUSTIFY` 다.

1. **필드 치환은 모델 텍스트를 보존한다.** `replace_composed_char_with_display()` 는 #3216
   규약대로 `run.text`(공백 placeholder)를 그대로 두고 `run.display_text` 만
   `"DCT Technology Inc." + 공백74 + "115"` 로 만든다.
2. **꼬리말 마지막 줄도 양쪽정렬 대상이다.** `needs_word_distribution()` 의
   `is_header_footer_para` 예외 때문이다.
3. **슬랙 분배가 공백을 모델 텍스트로 셌다.** `compute_line_extra_spacing()` 의 양쪽정렬
   분기가 `r.text` 로 `all_chars` 를 만들어, 줄이 *후행 공백 75개로 끝나는* 것으로 판정됐다
   → `interior_spaces = 2`.
4. **폭·글자수는 이미 표시 텍스트 기준이다.** `total_text_width`·`total_char_count` 는
   `effective_text_for_metrics()` 를 쓴다. 그래서 슬랙은 표시 폭으로 구하고 나누기는 공백
   2개로 해서 `extra_word_spacing` 이 공백당 **262.9 px** 로 부풀었다.
5. **렌더는 표시 텍스트의 모든 공백에 그 여분을 붙인다.** 공백 76개 × 262.9 px 가 쌓여
   쪽번호가 20,163 px 로 밀려났다.

즉 **분모(모델 텍스트의 내부 공백 2개)와 실제 적용 대상(표시 텍스트의 공백 76개)이 어긋난
것**이 근본 원인이다. `total_char_count` 는 #3216 때 표시 텍스트 축으로 옮겼는데 같은 함수의
공백 계수만 모델 텍스트 축에 남아 있었다.

## 4. 수정

`src/renderer/layout/paragraph_layout.rs` 의 양쪽정렬 분기에서 공백을 **그려지는 텍스트**로 센다.

```rust
let all_chars: Vec<char> = comp_line
    .runs
    .iter()
    .flat_map(|r| effective_text_for_metrics(r).chars())
    .collect();
```

`display_text` 가 없는 런은 `effective_text_for_metrics()` 가 `run.text` 를 그대로 돌려주므로
일반 본문 동작은 바뀌지 않는다. 이 줄에서는 후행 공백 0개·내부 공백 76개가 되어
`slack / 76` 이 공백당 여분이 되고, 줄 끝(=쪽번호 오른쪽 끝)이 본문 폭에 정확히 맞는다.

## 5. red → green

```
$ cargo test --profile release-test --test regression_suite_022 issue_5899   # 수정 전
test issue_5899_..._no_glyph_is_drawn_outside_the_paper ... FAILED
test issue_5899_..._footer_page_number_sits_at_the_right_margin ... FAILED
  p2: 글자 "1" 가 x=20656.8 로 종이 폭 793.7 밖에 그려졌다
  p2: 쪽번호 1 가 오른쪽 여백(0.8×793.7 ~ 793.7px) 안에 와야 한다 (실제 x=20656.8)
test result: FAILED. 0 passed; 2 failed

$ cargo test --profile release-test --test regression_suite_022 issue_5899   # 수정 후
test result: ok. 2 passed; 0 failed
```

테스트는 0-based 1·115·150쪽(=쪽번호 1·115·150)에서 (a) 어떤 글자도 종이 밖에 없을 것,
(b) 꼬리말 마지막 줄이 쪽번호로 끝나고 그 x 가 종이 폭의 0.8배~1.0배 사이일 것을 잠근다.
(b)는 슬랙을 통째로 없애 왼쪽에 붙여 버리는 반대 방향 회귀도 막는다.

## 6. 검증 게이트

| 게이트 | 결과 |
|---|---|
| `cargo test --profile release-test --lib -p rhwp` | **3893 passed / 0 failed** (13 ignored) |
| 신규 계약 테스트 `regression_suite_022 issue_5899` | 수정 전 **2 failed** → 수정 후 **2 passed** |
| `regression_suite_022` 전체 | 116 passed / 0 failed |
| 정렬·머리말/꼬리말 인접 스위트 004·005·013·021·028·032 (`--no-fail-fast`) | 767 passed / **4 failed(전부 사전 실패)** |
| 259문서 쪽수 게이트 (`tools/render_page_gate.py`) | 수정 전후 TSV **완전 동일** — 일치 245/259 (94.6%), **회귀 0** |
| 코퍼스 SVG self-diff (fixture 60문서 × 앞 2쪽 = 90 SVG) | **차이 0** — 이 변경은 필드 있는 머리말/꼬리말 밖에서는 무동작 |
| `cargo clippy --all-targets -- -D warnings` | exit 0 (경고 없음) |
| `node scripts/rust-unit-test-tiers.mjs --check --base-ref origin/devel` | 확인 완료: 4225 tests / 299 modules |
| rustfmt (`--check --edition 2021`, LF 사본 + `mod` 스텁) | exit 0 |
| src 유닛 테스트 총량 래칫 | `src/**` 의 `#[cfg(test)]` 증가 0 (새 테스트는 `tests/cases/`) |

사전 실패 4건은 origin/devel(`72674c565`) 무수정 상태에서도 **같은 이름으로 동일하게** 실패한다
(같은 바이너리로 재실행해 확인). 이번 변경과 무관하다:

| 스위트 | 사전 실패 |
|---|---|
| 005 | `wmf_emf_goldens::wmf_emf_goldens_lock_current_engine` (WMF/EMF 골든 9건 불일치) |
| 005 | `issue_4179_cursor_rect_text_host_para_pages::text_host_para_cursor_rect_builds_few_page_trees` (page tree 17회 > 기대 3회) |
| 013 | `issue_4129_mixed_nested_scan_budget::giant_cell_paginate_scan_budget_holds` (스캔 카운터 0회) |
| 013 | `issue_4126_cursor_rect_empty_para_pages::empty_host_para_cursor_rect_builds_few_page_trees` (page tree 20회 > 기대 8회) |

(013 무수정 1회차에서 `apply_endnote_shape_contract::dry_run_no_file` 이 한 번 실패했으나 같은
바이너리 재실행 2회에서 재현되지 않은 플레이크였다. 수정본에서는 나오지 않았다.)

## 7. 전/후 스크린샷

- 전체 쪽: `mydocs/report/edit_demo_5899/p116_full.png`
- 꼬리말 확대: `mydocs/report/edit_demo_5899/p116_footer_zoom.png`

각각 **수정 전 / 수정 후 / 한글 2020 정본** 3단이다.
