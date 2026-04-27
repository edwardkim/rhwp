# Task #391 단계 1 — 재현 정량 진단 + Red 테스트

- **이슈**: [#391](https://github.com/edwardkim/rhwp/issues/391)
- **브랜치**: `local/task391`
- **단계**: 1/4

## 목표

- 현재(devel) `exam_eng.hwp` baseline 측정 + 단계 2 검증용 Red 테스트 추가.

## 작업

### 1. baseline 측정

`./target/release/rhwp dump-pages samples/exam_eng.hwp > mydocs/working/task_m100_391_baseline.txt`

| 항목 | 값 |
|---|---|
| 페이지 수 | **11** (기대 8) |
| 단독 1-item 단 발생 | **3 곳** |

단독 1-item 단 위치:

| 페이지 | 단 | items | used | hwp_used | diff |
|---|---|---|---|---|---|
| p3 | 단 1 | 1 (pi=122) | 19.9px | 1208.6 | -1188.7 |
| p5 | 단 0 | 1 | 23.0px | 1208.6 | -1185.6 |
| p7 | 단 1 | 1 (pi=209) | 27.5px | 1208.4 | -1180.9 |

p1 col 0 -91.4px, p1 col 1 +217.1px, p2 col 0 -969.1px 의 비대칭은 baseline 파일 참조.

### 2. Red 테스트 추가

`tests/exam_eng_multicolumn.rs` 신규 (단일 테스트 함수):

```rust
#[test]
fn exam_eng_page_count_after_359_fix() {
    // ... HwpDocument::from_bytes ...
    assert_eq!(doc.page_count(), 8, "...");
}
```

페이지 수 8 만 검증하는 단일 케이스로 시작. 단 채움 비대칭 (col items 분포) 은 dump-pages 출력의 baseline 파일로 정성 비교.

### 3. Red 확인

```
$ cargo test --test exam_eng_multicolumn --release
test exam_eng_page_count_after_359_fix ... FAILED
assertion `left == right` failed: exam_eng.hwp 8 페이지 기대 (Task #391 / #359 회귀 복원). 실측 11p.
  left: 11
 right: 8
```

기대대로 Red. 단계 2 에서 가드 수정 후 Green 전환 예정.

## 산출물

- `tests/exam_eng_multicolumn.rs` (신규, 30 줄)
- `mydocs/working/task_m100_391_baseline.txt` (baseline 측정 출력)
- `mydocs/working/task_m100_391_stage1.md` (본 보고서)

## 단계 2 진행 승인 요청

본 단계 1 보고서 + 커밋 승인 후 단계 2 (가드 `is_last_column` 조건 추가) 진행.
