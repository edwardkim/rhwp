# Task #331 구현계획서 — 문단 trailing line_spacing advance 보정

- **수행계획서**: `mydocs/plans/task_m100_331.md`
- **이슈**: [#331](https://github.com/edwardkim/rhwp/issues/331)
- **브랜치**: `local/task331`

---

## 변경 대상

`src/renderer/typeset.rs` 단일 파일.

### 핵심 원칙

1. `FormattedParagraph::total_height` 자체는 변경하지 않음 (다른 호출부 영향 회피).
2. `current_height += ...` 누적 시점에서만 trailing_ls 를 빼서 HWP `vpos_h` 와 일치.
3. **마지막 partial 인 경우에만** trailing_ls 보정 적용. 분할되어 다음 페이지로 이어지는 partial 은 변경하지 않음 (기존 동작 유지).

---

## Stage 1 — FullParagraph advance 보정

**위치**: `src/renderer/typeset.rs:612, 622`

```diff
-        if st.current_height + fmt.height_for_fit <= available {
+        if st.current_height + fmt.height_for_fit <= available {
             st.current_items.push(PageItem::FullParagraph { para_index: para_idx });
-            st.current_height += fmt.total_height;
+            st.current_height += fmt.height_for_fit;
             return;
         }
 ...
         if line_count == 0 {
             st.current_items.push(PageItem::FullParagraph { para_index: para_idx });
-            st.current_height += fmt.total_height;
+            st.current_height += fmt.height_for_fit;
             return;
         }
```

**근거**: `height_for_fit = total_height - trailing_ls`. fit 검사가 이미 이 값을 사용 중이므로 advance 도 동일하게 맞추면 일관성 확보 + HWP `vpos_h` 일치.

### 검증

```bash
cargo build --release
RHWP_TYPESET_DRIFT=1 ./target/release/rhwp export-svg samples/21_언어_기출_편집가능본.hwp -p 0 2>&1 | grep -E "LAYOUT_OVERFLOW|^문서"
```

기대: `LAYOUT_OVERFLOW` 사라지거나 크게 감소.

---

## Stage 2 — PartialParagraph 마지막 partial trailing_ls 보정

**위치**: `src/renderer/typeset.rs:671~700`

현재:
```rust
let part_line_height = fmt.line_advances_sum(cursor_line..end_line);
let part_sp_after = if end_line >= line_count { fmt.spacing_after } else { 0.0 };
let part_height = sp_b + part_line_height + part_sp_after;
...
st.current_height += part_height;
```

`line_advances_sum` 은 범위 내 모든 줄의 `lh + ls` 를 더함. 마지막 partial(`end_line >= line_count`) 일 때 마지막 줄의 `ls` 가 trailing_ls 에 해당하므로 빼야 함.

```diff
         let part_line_height = fmt.line_advances_sum(cursor_line..end_line);
         let part_sp_after = if end_line >= line_count { fmt.spacing_after } else { 0.0 };
-        let part_height = sp_b + part_line_height + part_sp_after;
+        let trailing_ls_correction = if end_line >= line_count {
+            fmt.line_spacings.get(end_line - 1).copied().unwrap_or(0.0)
+        } else {
+            0.0
+        };
+        let part_height = sp_b + part_line_height + part_sp_after - trailing_ls_correction;
```

**왜 마지막 partial 만 보정?**
중간 partial 의 마지막 줄은 "문단의 진짜 마지막 줄" 이 아니므로 line_spacing 이 다음 줄과의 간격으로 유효함. 다음 페이지로 넘어가도 그 간격은 advance 에 포함되어야 함 (페이지 break 위치 결정과 무관).

### 검증

기존 분할 동작 회귀 없는지 `cargo test --lib` 로 확인.

---

## Stage 3 — 검증

### 3-1. 21_언어 샘플 (이슈 본문 케이스)

```bash
./target/release/rhwp export-svg samples/21_언어_기출_편집가능본.hwp -p 0 -o output/svg/task331/
./target/release/rhwp export-svg samples/21_언어_기출_편집가능본.hwp -p 1 -o output/svg/task331/
./target/release/rhwp export-svg samples/21_언어_기출_편집가능본.hwp --debug-overlay -p 0 -o output/debug/task331/
```

확인 항목:
- page 1 col 1 우하단에 pi=26 + 보기 ①②③ 가 fit (PDF 일치)
- `LAYOUT_OVERFLOW` 로그 사라짐
- `RHWP_TYPESET_DRIFT` 로그에서 `cur_h` 가 `vpos_h` 합과 일치하는 추세

### 3-2. lib 테스트

```bash
cargo test --lib 2>&1 | tail -20
```

기대: 992 passed 유지.

### 3-3. Golden SVG 6 개

```bash
cargo test --test golden_svg 2>&1 | tail -30
```

회귀 발생 시 → Stage 4 진행.

### 3-4. 기타 샘플 페이지 수 비교 (스팟 체크)

```bash
for f in form-002 multi-table-002 tac-case-002; do
  ./target/release/rhwp export-svg samples/${f}.hwp -o output/svg/task331/${f}/ 2>&1 | grep "내보내기 완료"
done
```

기대: 페이지 수 변동 없음 또는 의도된 감소(±1).

---

## Stage 4 (조건부) — Golden baseline 갱신

Stage 3-3 회귀 시:
1. 각 golden 의 `.actual.svg` 와 baseline 시각 비교 (browser/diff)
2. 변경이 trailing_ls 보정에 의한 의도된 결과인지 확인
3. 의도된 변경이면 baseline 갱신 + 변경 내역을 `mydocs/working/task_m100_331_stage4.md` 에 기록
4. 의도되지 않은 변경이면 Stage 1·2 재검토

---

## 단계별 보고서 / 커밋 계획

| Stage | 보고서 | 커밋 메시지 |
|-------|--------|------------|
| 1 | `mydocs/working/task_m100_331_stage1.md` | `Task #331 Stage 1: FullParagraph advance trailing_ls 보정` |
| 2 | `mydocs/working/task_m100_331_stage2.md` | `Task #331 Stage 2: PartialParagraph 마지막 partial trailing_ls 보정` |
| 3 | `mydocs/working/task_m100_331_stage3.md` | `Task #331 Stage 3: 21_언어 샘플 + lib/golden 검증` |
| 4 (조건부) | `mydocs/working/task_m100_331_stage4.md` | `Task #331 Stage 4: golden baseline 갱신` |
| 최종 | `mydocs/report/task_m100_331_report.md` | `Task #331: 문단 trailing line_spacing 누적 drift 해결` |

---

## 위험 요소 및 대응

| 위험 | 영향 | 대응 |
|------|------|------|
| 표/footnote advance 경로(line 1051, 1059, 1082, 1090, 1165, 1391, 1404, 1490)와 충돌 | 표/각주 위치 변동 | 해당 경로는 fmt.total_height 미사용 또는 별도 보정(line 1090) → 이번 변경 영향 없음 확인 |
| Golden 6 개 모두 회귀 | baseline 6 개 갱신 | Stage 4 에서 시각 비교 후 일괄 갱신 |
| 누적 차이로 페이지 수 감소 → 다른 샘플의 의도된 페이지 수 깨짐 | 다른 테스트 실패 가능 | Stage 3-4 에서 스팟 체크, 필요 시 baseline 갱신 |
| `paragraph_layout` 의 줄 y 좌표가 advance 와 어긋남 | 시각적 겹침/공백 | paragraph_layout 은 자체 spacing_before + per-line lh+ls 누적 → 동일 문단 내 일관, 본 변경은 문단 간 advance 만 영향 (안전) |

---

## 승인 요청

위 구현계획에 대한 승인 후 Stage 1 부터 진행하겠습니다.
