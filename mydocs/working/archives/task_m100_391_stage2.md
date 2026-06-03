# Task #391 단계 2 — 가드 → 누적 공식 분기 (Green)

- **이슈**: [#391](https://github.com/edwardkim/scope/rhwp/issues/391)
- **단계**: 2/4

## 단계 1 → 2 사이의 분석 정정

구현계획서 단계 2 의 "초안" 은 `next_will_vpos_reset` 가드에 `is_last_column` 조건 추가였으나, **시도 결과 11p → 12p 로 악화** (single-item 단 더 발생). 가드는 다단에서도 안전마진 끄기로 보조 효과가 있었음.

**진짜 원인은 #359 의 누적 공식 변경 (`current_height += height_for_fit → total_height`)**. 다단에서는 layout 이 `vpos` 기반 stacking 을 하므로 typeset 누적의 `trailing_ls` 인플레이션이 단을 조기 종료시킴.

격리 검증 (worktree 로 pre-#359 vs current devel vs my fix 비교):

| 샘플 | pre-#359 | current devel | 가드만 비활성 | **누적 분기** |
|---|---|---|---|---|
| exam_eng | 8p, 0 | 11p, 0 | 12p, 0 | **8p, 0** ✓ |
| exam_kor | 24p, 30 | 30p, 0 | - | 24p, 30 (pre-#359 동일) |
| k-water-rfp | 26p, 73 | 27p, 0 | - | **27p, 0** ✓ |
| kps-ai (1단) | 81p, 60 | 79p, 5 | - | 79p, 5 |
| aift (1단) | 74p, 30 | 77p, 3 | - | 77p, 3 |

## 채택한 수정 (다단/단단 분기)

`src/renderer/typeset.rs:805` (FullParagraph 배치 후 누적):

```rust
// [Task #391] 다단/단단 분기:
//   - 단단 (col_count == 1): total_height (k-water-rfp p3 311px drift 차단, #359)
//   - 다단 (col_count > 1): height_for_fit (exam_eng 8p 정상 단 채움 복원)
// 다단에서는 layout 이 vpos 기반으로 항목을 단별로 stacking 하므로
// typeset 누적 시 trailing_ls 인플레이션이 단을 조기 종료시킴.
st.current_height += if st.col_count > 1 { fmt.height_for_fit } else { fmt.total_height };
```

라인 815 (line_count == 0 폴백 경로) 도 동일 변경. 라인 1294 의 `total_height - height_for_fit` (= trailing_ls 만큼 보정) 는 partial split 경로로 변경 없음.

`next_will_vpos_reset` 가드는 **그대로 둔다** (단단 케이스의 #359 효과 보존 + 다단에서 safety_margin 끄기 보조 효과).

## 회귀 검증

```
cargo test --lib --release            # 1014 passed (was 1008 + 6 new from other tasks)
cargo test --test svg_snapshot        # 6/6
cargo test --test issue_301           # 1/1
cargo test --test exam_eng_multicolumn # 1/1 (Red → Green)
cargo clippy --lib --release -- -D warnings  # 통과
```

## 11 샘플 회귀

| 샘플 | 단수 | pages | overflow | 변동 |
|---|---|---|---|---|
| **exam_eng** | 2단 | **11 → 8** | 0 → 0 | ✓ 본 task 핵심 |
| exam_kor | 2단 | 30 → 24 | 0 → 30 | pre-#359 상태 복원 (다단 layout 미해결 잔존) |
| **k-water-rfp** | 1단 | 27 | 0 | ✓ #359 보존 |
| kps-ai | 1단 | 79 | 5 | 무변화 |
| aift | 1단 | 77 | 3 | 무변화 |
| form-01 | 1단 | 1 | 0 | 무변화 |
| KTX | 1단 | 27 | 1 | 무변화 |
| hwp-multi-001 | 1단 | 10 | 0 | 무변화 |
| exam_math | - | 20 | 0 | 무변화 |
| biz_plan | - | 6 | 0 | 무변화 |
| 21_언어 | - | 15 | - | 무변화 |

## exam_kor 의 잔존 overflow

exam_kor 는 **#359 이전부터 24p, 30 overflow 였음** (worktree 로 직접 확인). #359 가 우연히 30p, 0 으로 만들었지만 페이지 수 자체가 늘어난 것이지 콘텐츠 정합성 보장이 아님. 다단 layout 의 `vpos` 기반 stacking 은 별개의 작업으로 추후 새 이슈에서 다룰 영역.

본 task 의 책임 범위는 "exam_eng 회귀 복원" 이며, exam_kor 잔존 overflow 는 사전부터 존재한 별개 문제이므로 본 task 에서 해결하지 않는다.

## 산출물

- `src/renderer/typeset.rs` 누적 공식 다단/단단 분기 (~3 줄 추가, 2 곳)
- `mydocs/working/task_m100_391_stage2.md` (본 보고서)

## 단계 3 진행 승인 요청

본 단계 2 보고서 + 커밋 승인 후 단계 3 (회귀 검증 정리 + 시각 판정 요청) 진행.
