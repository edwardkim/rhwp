# 단계 2 보고서 — Task #356 인접 문단 vpos 리셋 감지 헬퍼

- **단계**: 2/5
- **브랜치**: `local/task356`

## 변경 사항

### 신규 함수

`src/renderer/pagination/engine.rs` 파일 끝에 free function 추가:

```rust
pub(super) fn detect_inter_paragraph_vpos_reset(
    prev: &Paragraph,
    cur: &Paragraph,
) -> bool
```

**감지 조건** (전부 만족 시 true):
1. 두 문단 모두 `line_segs` 비어있지 않음
2. `prev.last.column_start == cur.first.column_start` (단 변경과 구분)
3. `cur.first.vertical_pos < prev.last.vpos_end` (= vpos + lh + ls)

### 단위 테스트 (6개)

`mod inter_para_vpos_reset_tests`:

| 테스트 | 시나리오 | 기대 |
|--------|---------|------|
| `returns_false_when_either_paragraph_has_no_line_segs` | line_segs 비어있음 | false |
| `returns_false_for_normal_progression` | cur.vpos = prev.vpos_end (touching), 정상 진행 | false |
| `returns_true_for_clear_reset_to_new_page` | 본 샘플 pi=39(vpos_end=68681) → pi=40(vpos=500) | true |
| `returns_false_when_columns_differ` | column_start 다름 (다단 단 변경) | false |
| `returns_true_for_subtle_reset_just_below_vpos_end` | cur.vpos = prev.vpos_end - 1 | true |
| `uses_last_line_seg_of_prev_not_first` | prev 가 여러 줄일 때 마지막 줄 기준 검증 | (false / true) |

## 검증

```
cargo test --release inter_para_vpos_reset_tests
test result: ok. 6 passed; 0 failed
```

전체 회귀:
```
cargo test --release
test result: ok. 1014 passed; 0 failed; 1 ignored
+ 14 + 25 + 6 + 1 + 1 PASS, 0 FAIL (전체 1061)
```

베이스라인 1055 → 1061 (신규 6 추가, 기존 회귀 0).

## 다음 단계

**단계 3**: 페이지네이션 엔진 (`paginate_with_measured_opts` 메인 루프) 에 헬퍼 호출 통합. 문단 처리 진입부에서 prev 문단(있으면) 과 비교 후 트리거 시 `st.advance_column_or_new_page()` 강제 호출. 표/특수영역 제외 게이팅 적용.
