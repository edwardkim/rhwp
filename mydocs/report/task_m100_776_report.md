# Task #776 최종 보고서 — H1' + H3b 정정 (column top sb + zone 전환 ColumnDef.spacing)

**Issue**: [#776](https://github.com/edwardkim/rhwp/issues/776) Task #773 후속 정정
**브랜치**: `local/task776`
**기간**: 2026-05-10 (Stage 0-6)
**선행 RFC**: [Task #774](https://github.com/edwardkim/rhwp/issues/774)
**해소 결함**: [#773](https://github.com/edwardkim/rhwp/issues/773) (shortcut.hwp 페이지 1 본문 압축)

---

## 요약

[Task #774 RFC](https://github.com/edwardkim/rhwp/issues/774) 가 식별한 2개 independent paragraph spacing 결함을 정정:

1. **H1'**: `paragraph_layout.rs:744-749` 의 `is_column_top` 가드 → `cell_ctx.is_none()` 가드로 대체
2. **H3b**: `layout.rs:1237-1257` 의 zone 전환 시 ColumnDef.spacing / 2 가산

이슈 #773 (shortcut.hwp 페이지 1 본문 압축 ~52-64 px) 해소. 회귀 0 (cargo test 1217 passed).

## 정정 코드

### H1' (paragraph_layout.rs:744-749)

```diff
-        // 단/페이지의 맨 처음 문단은 spacing_before 적용하지 않음
-        let is_column_top = (y - col_area.y).abs() < 1.0;
-        if start_line == 0 && spacing_before > 0.0 && !is_column_top {
+        // [Task #776 H1'] 단/페이지의 맨 처음 문단도 sb 적용 (한컴 PDF 정합).
+        // 셀 안 paragraph 는 cell padding 과 sb 중복 방지를 위해 skip 유지.
+        // RFC #774 stage 3 분석: shortcut/sungeo/treatise 3 샘플 ±1 px 정합 검증.
+        if start_line == 0 && spacing_before > 0.0 && cell_ctx.is_none() {
             y += spacing_before;
         }
```

### H3b (layout.rs:1237-1257)

```diff
             let is_new_zone = (col_content.zone_y_offset - last_zone_y_offset).abs() > 0.1;
             if is_new_zone {
                 if col_content.zone_y_offset > 0.0 {
-                    current_zone_start_y = prev_zone_y_end;
+                    // [Task #776 H3b] zone 전환 시 진입 zone 의 ColumnDef.spacing / 2
+                    // 가산 (한컴 PDF 정합). RFC #774 stage 5 분석:
+                    // shortcut.hwp 의 zone 전환 측정 ~18.9 px = 10mm spacing / 2.
+                    let zone_top_extra = col_content.items.first()
+                        .map(|item| item.para_index())
+                        .and_then(|pi| paragraphs.get(pi))
+                        .and_then(|para| para.controls.iter().find_map(|c| {
+                            if let crate::model::control::Control::ColumnDef(cd) = c {
+                                Some(hwpunit_to_px(cd.spacing as i32, self.dpi) / 2.0)
+                            } else { None }
+                        }))
+                        .unwrap_or(0.0);
+                    current_zone_start_y = prev_zone_y_end + zone_top_extra;
                 } else {
                     current_zone_start_y = 0.0;
                 }
                 last_zone_y_offset = col_content.zone_y_offset;
             }
```

## 검증 결과

### issue_776 가드 (4건 GREEN)

| Test | Stage 1 (RED) | Stage 2 (H1') | Stage 3 (H3b) | PDF 기대 | 정합 |
|------|--------------|--------------|--------------|---------|------|
| shortcut.hwp pi=0 | 0.00 | 26.45 | 26.45 | 26.83 | ±0.38 ✓ |
| shortcut.hwp pi=2 | 73.76 | 100.21 | 138.01 | 137.87 | ±0.14 ✓ |
| sungeo.hwp pi=0 | 0.00 | 13.33 | 13.33 | 12.63 | ±0.70 ✓ |
| treatise sample.hwp pi=0 | 0.00 | 24.00 | 24.00 | 23.69 | ±0.31 ✓ |

### RFC #774 예측 검증

| 측정 | 변화량 | RFC 예측 | 정확도 |
|------|------|---------|------|
| shortcut.hwp pi=0 offset | +26.45 | sb=26.45 | 100% |
| shortcut.hwp pi=2 offset | +64.25 | H1'+H3b=64.25 | 100% |
| sungeo.hwp pi=0 offset | +13.33 | sb=13.33 | 100% |
| treatise pi=0 offset | +24.00 | sb=24.00 | 100% |

**RFC 예측 100% 정확** — 모든 변화량이 RFC 분석과 일치.

### 회귀 검증

```
$ cargo test --release
test result: ok. 1217 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
```

영역별:
- re_sample (재현 검증): 13 passed
- exam_math/eng/kor (다단 layout): 회귀 0
- column (다단): 27 passed
- 페이지 카운트 (shortcut/sungeo/treatise/exam_*): 회귀 0
- clippy (신규 코드 영역): 경고 0건

## 영향 범위

### H1' 영향 받는 케이스

- 단/페이지 첫 paragraph 의 ParaShape.spacing_before > 0
- 셀 밖 (cell_ctx.is_none())

10 샘플 분석 결과: 3/10 영향 (shortcut, sungeo, treatise sample). 7/10 변경 없음 (sb=0).

### H3b 영향 받는 케이스

- 새 zone 진입 (zone_y_offset > 0)
- 진입 zone 의 첫 PageItem 의 paragraph 가 ColumnDef control 포함
- ColumnDef.spacing > 0

shortcut.hwp 의 모든 페이지 (헤더 + TAC 표 + 본문 다단 패턴) 에 적용. 다른 단일 zone 샘플은 영향 없음.

## 단계별 산출물

| 단계 | 산출물 | 결과 |
|------|------|------|
| Stage 0 | 수행/구현 계획서 | 작성 완료 |
| Stage 1 | RED test 가드 (issue_776.rs) | 4 RED |
| Stage 2 | H1' 정정 | 3 GREEN + 1 RED (예상) |
| Stage 3 | H3b 정정 | 4 GREEN |
| Stage 4 | 회귀 검증 | 1217 passed |
| Stage 5 | 광범위 검증 + edge case | 회귀 0 |
| Stage 6 | 최종 보고서 (본 문서) | — |

## 이슈 close 권고

- **#773** (shortcut.hwp 페이지 1 본문 압축): 본 task 정정으로 해소 → close
- **#776** (Task #773 후속 정정): 본 task 완료 → close
- **#774** (RFC 분석): 본 task 가 RFC 권고 적용 → close (RFC analysis phase complete)

## 미해결 (RFC 영역)

- `/2` factor 의 이론적 근거 (HWPSPEC 명세 외)
- 셀 안 paragraph sb 적용 패턴 (cell padding 과의 정합)
- ParaShape.spacing_after (sa) 정합
- 다중 zone (3+ zone) 의 H3b 누적 동작 검증

→ 후속 RFC 또는 추가 분석 task 영역.

## PR 권고

본 task 정정을 stream/devel 에 반영하기 위한 PR 생성 권고:
- base: stream/devel
- head: origin/pr-task776 (cherry-pick from local/task776)
- 검토 항목:
  - paragraph_layout.rs / layout.rs 의 정정 코드
  - tests/issue_776.rs 의 가드
  - 회귀 검증 결과 (1217 passed)
  - 시각 정합 (PDF 기준)

작업지시자 승인 후 origin → stream/devel PR 생성.
