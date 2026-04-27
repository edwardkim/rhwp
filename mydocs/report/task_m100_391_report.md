# Task #391 최종 결과 보고서 — 다단 섹션 #359 누적 공식 회귀 정정

- **이슈**: [#391](https://github.com/edwardkim/rhwp/issues/391)
- **브랜치**: `local/task391`
- **마일스톤**: v1.0.0 (M100)
- **작업일**: 2026-04-27

## 요약

`samples/exam_eng.hwp` (Section-level 2단 다단) 가 #359 merge 직후부터 **8 → 11 페이지로 회귀**하고 단(column) 채움이 비대칭으로 어그러져 단독 1-item 단이 다수 발생. 분석 결과 진짜 원인은 #359 의 **누적 공식 변경 (`current_height += total_height`)** 임이 확인되어, 다단/단단 분기로 정정.

| 결과 | exam_eng | k-water-rfp | exam_kor |
|---|---|---|---|
| pre-#359 | 8p, 0 | 26p, 73 | 24p, 30 |
| current devel | **11p, 0** ⚠️ | 27p, 0 | 30p, 0 |
| **#391 수정 후** | **8p, 0** ✓ | 27p, 0 ✓ | 24p, 30 (pre-#359 동등) |

## 증상

```
=== 페이지 3 (수정 전) ===
  단 0 (items=22, used=1191.0px, hwp_used≈1188.7px, diff=+2.3px)
  단 1 (items=1, used=19.9px, hwp_used≈1208.6px, diff=-1188.7px)  ← 단독 1-item
=== 페이지 5 ===
  단 0 (items=1, used=23.0px, hwp_used≈1208.6px, diff=-1185.6px)  ← 단독 1-item
=== 페이지 7 ===
  단 1 (items=1, used=27.5px, hwp_used≈1208.4px, diff=-1180.9px)  ← 단독 1-item
```

## 원인 분석

### 초기 가설 (오답)

`src/renderer/typeset.rs:421` 의 #359 선제 가드 `next_will_vpos_reset` 가 다단 단 전환을 단독 페이지 위험으로 오인. `is_last_column = current_column + 1 >= col_count` 조건 추가 시도.

**결과: 12p 로 더 악화** (single-item 단 더 발생). 가드는 다단에서도 안전마진 끄기로 보조 효과가 있었음.

### 실제 원인

`src/renderer/typeset.rs:805,815` 의 누적 공식 변경:

```rust
// #359 이전:  st.current_height += fmt.height_for_fit;
// #359 이후:  st.current_height += fmt.total_height;  ← 다단에서 trailing_ls 인플레이션
```

`RHWP_TYPESET_DRIFT=1` 진단:

| 샘플 | 항목 | trail_ls | vpos_h (실측 layout 높이) | fmt_total | diff |
|---|---|---|---|---|---|
| exam_eng | pi=122 | 4.6 | 15.3 | 19.9 | +4.6 |
| exam_kor | pi=1 | 9.2 | 235.9 | 245.1 | +9.2 |

**다단 layout 은 LINE_SEG 의 `vpos` 값으로 stacking** 하므로 typeset 누적이 `total_height` (= `vpos_h + trail_ls`) 를 더하면 N items × trail_ls 만큼 인플레이션 발생 → 단을 조기 종료시켜 다음 단/페이지로 항목들이 밀림.

단단 (k-water-rfp) 의 311px drift 는 layout 이 trailing_ls 를 stacking 에 포함시키는 별개 경로 — `total_height` 가 정합. 다단/단단 layout 의 stacking 차이가 본질.

## 변경 내용

### 코드 (`src/renderer/typeset.rs`, ~3 줄)

```rust
// [Task #391] 다단/단단 분기:
//   - 단단 (col_count == 1): total_height (k-water-rfp p3 311px drift 차단, #359)
//   - 다단 (col_count > 1): height_for_fit (exam_eng 8p 정상 단 채움 복원)
// 다단에서는 layout 이 vpos 기반으로 항목을 단별로 stacking 하므로
// typeset 누적 시 trailing_ls 인플레이션이 단을 조기 종료시킴.
st.current_height += if st.col_count > 1 { fmt.height_for_fit } else { fmt.total_height };
```

라인 805 (`fits → place 전체배치`) 와 815 (`line_count == 0` 폴백) 두 곳 동일 변경. 라인 1294 (partial split 보정) 는 변경 없음.

`next_will_vpos_reset` 가드는 **그대로 둔다** — 단단의 #359 효과 보존 + 다단에서 `safety_margin` 끄기 보조 효과.

### 테스트 (`tests/exam_eng_multicolumn.rs`, 신규)

```rust
#[test]
fn exam_eng_page_count_after_359_fix() {
    let doc = HwpDocument::from_bytes(&bytes).expect(...);
    assert_eq!(doc.page_count(), 8, "exam_eng.hwp 8 페이지 기대 ...");
}
```

## 검증

### 자동 검증

| 항목 | 결과 |
|---|---|
| `cargo test --lib --release` | **1014 passed**, 0 failed, 1 ignored |
| `cargo test --test exam_eng_multicolumn` | **1/1** (Red → Green) |
| `cargo test --test svg_snapshot` | 6/6 |
| `cargo test --test issue_301` | 1/1 |
| `cargo test --test hwpx_roundtrip_integration` | 14/14 |
| `cargo test --test hwpx_to_hwp_adapter` | 25/25 |
| `cargo clippy --lib --release -- -D warnings` | 통과 |

### WASM 빌드

```
docker compose --env-file .env.docker run --rm wasm
[INFO]: :-) Done in 1m 13s
[INFO]: :-) Your wasm pkg is ready to publish at /app/pkg.

pkg/rhwp_bg.wasm: 3.9 MB
pkg/rhwp.js:      222 KB
```

### 11 샘플 회귀 비교

| 샘플 | 단수 | pre-#359 | current devel | #391 수정 후 | 판정 |
|---|---|---|---|---|---|
| **exam_eng** | 2단 | 8p, 0 | 11p, 0 | **8p, 0** | ✓ 본 task 핵심 |
| exam_kor | 2단 | 24p, 30 | 30p, 0 | 24p, 30 | pre-#359 동등 |
| **k-water-rfp** | 1단 | 26p, 73 | 27p, 0 | **27p, 0** | ✓ #359 보존 |
| kps-ai | 1단 | 81p, 60 | 79p, 5 | 79p, 5 | 무변화 |
| aift | 1단 | 74p, 30 | 77p, 3 | 77p, 3 | 무변화 |
| form-01 | 1단 | - | - | 1p, 0 | 무변화 |
| KTX | 1단 | - | - | 27p, 1 | 무변화 |
| hwp-multi-001 | 1단 | - | - | 10p, 0 | 무변화 |
| exam_math | - | - | - | 20p, 0 | 무변화 |
| biz_plan | - | - | - | 6p, 0 | 무변화 |
| 21_언어 | - | - | - | 15p, - | 무변화 |

단단 모든 샘플 무변화 → kps-ai/aift 의 #359 개선 보존. exam_kor 잔존 30 overflow 는 #359 이전부터 존재 (worktree 직접 검증). 본 task 책임 범위 외.

### 시각 판정 자료

`output/svg/task391/{exam_eng,k-water-rfp,exam_kor}/` — 작업지시자 검토.

## 잔존 사항

- **exam_kor 다단 layout overflow** (24p, 30 overflow): #359 이전부터 존재. 다단 layout 의 vpos 기반 stacking 정합성을 다루는 별개 작업으로 추후 새 이슈에서 검토.

## 관련 이슈/커밋

- 본 task: 단계 1 (`14011e2`) → 단계 2 (`567475e`) → 단계 3 (`3f90774`) → 단계 4 (이 보고서)
- 원인 task: #359 (`e5d383f`)
- 별개 (이전 회귀): #345 (exam_eng 9 → 8 PR #343 후속, OPEN)

## 결론

`src/renderer/typeset.rs:805,815` 의 누적 공식을 `col_count` 로 분기하여 #359 의 다단 회귀를 정정. **exam_eng 8 페이지 복원 + 단독 1-item 단 해소** 달성하며 단단 케이스의 #359 개선은 그대로 유지. 1014 lib + 모든 통합 테스트 통과, WASM 빌드 정상.
