# Stage 2 완료보고서 — Task M100 #1218

**단계**: 단락 겹침 수정 (wrap=Square 호스트 본문 커서 전진)
**브랜치**: `local/task1218`

## 변경

`src/renderer/layout.rs` — 어울림(Square) 호스트 Table item 처리에서 `layout_wrap_around_paras` 호출 직후, 호스트 본문이 표보다 길면 커서를 본문 하단까지 전진:

```rust
let host_text_bottom = table_y_before + (text_h - last_ls).max(0.0);
if host_text_bottom > y_offset { y_offset = host_text_bottom; }
```

- `text_h` = 호스트 composed 줄들의 (line_height+line_spacing) 합, 마지막 trailing line_spacing 제외(height_for_fit 정합).
- 본문 ≤ 표(기존 다수 케이스): `host_text_bottom ≤ y_offset` → **동작 불변**.

## 검증 (계측 + 시각)

```
(수정 전) Table pi=258 y 822.4→895.9(dy=73.6), ① pi=259 y=895.9  → 겹침
(수정 후) Table pi=258 y 822.4→912.5(dy=90.1), ① pi=259 y=912.5  → 분리
```

- 4쪽 문26: **① 0.7262 가 문제 끝줄 아래 독립 줄로 렌더** (PDF 정합). ②~⑤ 도 각 줄 분리.
- `cargo test --release`: **1896 passed / 0 failed** (wrap_around `issue_546`, `svg_snapshot` 포함 — 회귀 0).
- rustfmt clean (layout.rs 19줄 추가, 단일 hunk).

## 남은 사항 (Stage 3 — 별도)

z-표 행 압축("1.0"/"1.1"→"1.01.")은 **표 셀 내부 줄높이**(셀[2] `lh=825 < 폰트 900`) 문제로, 본 단락-겹침 수정과 **독립**. 미해결 — 별도 처리 필요.
