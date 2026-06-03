# Task #445 Stage 2 — 구현

**브랜치**: `local/task445`
**작성일**: 2026-04-29

---

## 1. 변경 내용

### `src/renderer/layout.rs` (build_single_column merge 결과 후)

paragraph border 의 merge 그룹을 col_area 바닥/꼭대기로 클램프. y_end ≤ col_top 인 그룹은 제거.

```rust
// Task #445: paragraph border 가 col_area 바닥을 넘지 않도록 클램프.
// vpos-reset 미지원으로 paragraph 가 col_bottom 너머에 layout 될 수 있는데,
// border 까지 따라가면 페이지/꼬리말 영역까지 침범 (예: exam_kor p8 의 1671px).
// 텍스트 자체의 overflow 처리는 별도 이슈.
let col_top = col_area.y;
let col_bot = col_area.y + col_area.height;
for g in groups.iter_mut() {
    if g.2 < col_top { g.2 = col_top; }
    if g.4 > col_bot { g.4 = col_bot; }
}
groups.retain(|g| g.4 > g.2);
```

### 진단 코드 제거

Stage 1 에서 삽입한 `eprintln!` 모두 제거 (paragraph_layout.rs:2535 부근, layout.rs merge 부근).

## 2. 빌드/테스트 결과

```
$ cargo build --release
   Finished `release` profile [optimized] target(s) in 1m 05s

$ cargo test --release
test result: ok. 1066 passed; 0 failed; 1 ignored
test result: ok. 14 passed; 0 failed
test result: ok. 25 passed; 0 failed
test result: ok. 6 passed; 0 failed
... (총 1117 통과, 0 실패)
```

### 스냅샷 갱신: `issue_267_ktx_toc_page`

snapshot diff 1건 발생:
```
< <rect ... height="861.0666666666671" fill="none"/>
> <rect ... height="855.7333333333333" fill="none"/>
```

`fill="none"` + 무 stroke 의 invisible 구조 rect — paragraph border range 중 stroke 가 없는 케이스. 5.34px 가 col_bottom 너머로 빠지던 영역이 클램프됨. 가시적 변화 없음.

`UPDATE_GOLDEN=1 cargo test --release --test svg_snapshot issue_267_ktx_toc_page` 로 갱신.

## 3. 페이지별 결과 (exam_kor.hwp)

| 페이지 | 수정 전 y2 | 수정 후 y2 | 비고 |
|--------|-----------|-----------|------|
| 2 | 1452.7 | 1422.9 | col_bottom 일치 |
| 5 | 1506.5 | 1424.9 | col_bottom 일치 |
| 8 | **1671.3** | **1424.9** | 페이지 바깥 → 페이지 내부, **246px 단축** |
| 15 | 1595.8 | 1422.9 | col_bottom 일치 |

## 4. 회귀 검증 (다른 시험 샘플)

`/tmp` 에 export 후 y2>1500 길이 라인 검색 → 0건.

- `samples/exam_eng.hwp` (8페이지) ✓
- `samples/exam_math.hwp` (20페이지) ✓
- `samples/exam_science.hwp` (4페이지) ✓
- `samples/k-water-rfp.hwp` (28페이지) ✓

## 5. 산출물

- 코드 변경: `src/renderer/layout.rs` (merge 후 클램프 5줄 추가)
- golden 갱신: `tests/golden_svg/issue-267/ktx-toc-page.svg`

다음 단계: Stage 3 — 종합 검증 + PDF 비교.
