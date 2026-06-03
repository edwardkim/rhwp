# Task #463 Stage 2 완료보고서

## 변경 요약

`paragraph_layout.rs` 의 `para_border_ranges.push` 호출에 **`cell_ctx.is_none()` 게이팅** 1줄 추가.

```rust
// 변경 전
if para_border_fill_id > 0 {

// 변경 후
if para_border_fill_id > 0 && cell_ctx.is_none() {
```

## 변경 파일

- `src/renderer/layout/paragraph_layout.rs:2516` (게이팅 + 주석 4줄)

## 효과

표 셀 안의 단락은 본문 외곽선 큐(`para_border_ranges`) 에 더 이상 들어가지 않는다. 셀 외곽선 자체는 표 셀 렌더 경로(`table_layout`/`border_rendering`)에서 이미 처리되므로 시각적 손실은 없다.

## 검증

### 단위 테스트

```
cargo test --release --lib
test result: ok. 1069 passed; 0 failed; 1 ignored; 0 measured
```

전체 1069개 테스트 통과.

### exam_kor 14p 직접 확인

- 변경 전: 좌측 단에 본문 외곽선 박스 4개 (4~5분리)
- 변경 후: 좌측 단에 본문 외곽선 박스 1개 (단일 큰 박스)

stroke 가 있는 본문 외곽선 rect:

| 위치 | 변경 전 | 변경 후 |
|------|---------|---------|
| 좌측 단 본문 | 4개 | **1개** |
| 우측 단 본문 | 다수 | **1개** |
| [38~42] 헤더 | (없음, bf=5 idx=None borders) | (없음, 동일) |

### 회귀 (다른 샘플)

`./target/release/rhwp export-svg <file>` 정상 완료:

- `2010-01-06.hwp` (6p)
- `2022년 국립국어원 업무계획.hwp` (40p)
- `biz_plan.hwp` (6p)
- `21_언어_기출_편집가능본.hwp` (15p)

`samples/exam_kor.hwp` 전체 20p 의 본문 외곽선 stroked rect 개수는 page 별로 0~1 개 범위로 정상.

## 추가 발견 (참고)

분석 과정에서 별도 디버그 출력으로 확인한 사항:

1. PDF 의 [38~42] 안내 영역에는 외곽선이 **없다** (plain text). 이는 ParaShape[15] 가 참조하는 `border_fill[5]` 의 4면이 모두 `line_type=None width=0` 이기 때문. rhwp 렌더러는 이미 이 정보를 반영하여 strokeless rect 만 emit 하고 있었음 (수정 불필요).
2. cell context 게이팅을 추가한 후, bf=6 (사실은 idx=5/None borders) 와 bf=7 (idx=6/Solid borders) 그룹은 stroke signature 가 다르므로 (None vs Some) merge 로직에서 자동으로 분리된다. 별도의 threshold 조정은 불필요.

## 다음 단계

Stage 3: 시각 검증 + 회귀 테스트 (현재 완료) → Stage 4: 최종 보고서 + 머지.
