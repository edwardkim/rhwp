# Task #317 구현 계획서

상위: 수행계획서 `task_m100_317.md`
브랜치: `task317`

## 단계 구성

4단계 분할.

### 1단계 — paragraph-by-paragraph current_height 추적 (조사)

목표: 페이지 3 단 0 의 direct vs reloaded current_height 누적이 어디서 갈라지는지 정확히 식별. 코드 변경 없음.

작업:
1. 임시 진단 테스트 작성 (`tests/task317_diag.rs`)
2. TypesetEngine 의 paragraph 추가 시점에 current_height 출력 (env var trigger 또는 별도 모드)
3. 같은 paragraph index 의 direct vs reloaded current_height 누적 비교
4. 첫 차이 발생 paragraph 식별 + 차이 크기 기록

산출:
- `mydocs/working/task_m100_317_stage1.md`

### 2단계 — origin 식별 + 보강 후보 결정

목표: 1단계의 첫 차이 paragraph 에서 어떤 IR 필드가 typeset 결과를 바꾸는지 식별.

작업:
1. 첫 차이 paragraph 의 모든 IR 필드 비교 (char_count, char_offsets, controls, ctrl_data_records, 표/Shape 내부 등)
2. 각 후보 필드를 reloaded → direct 형태로 임시 패치 → 페이지 수 변화 측정 (binary search 식 좁혀가기)
3. origin 필드 확정

산출:
- `mydocs/working/task_m100_317_stage2.md`

### 3단계 — 보강 코드 작성

2단계 결과에 따라:
- a) HWPX 로드 시 normalize 추가 (현 `normalize_hwpx_paragraphs` 확장)
- b) HWP 직렬화기 보강
- c) HWP 파서 보강
- d) 어댑터 (`hwpx_to_hwp.rs`) 추가 변환

3샘플 (h-01, h-02, h-03) 동시 검증. 4샘플 (21_언어 등) 무회귀 확인.

산출:
- 코드 + `mydocs/working/task_m100_317_stage3.md`

### 4단계 — 격리 테스트 재활성화 + 최종 보고

작업:
- `tests/hwpx_to_hwp_adapter.rs` 의 `#[ignore]` 3건 제거
- 사유 메시지 제거
- 임시 진단 테스트 (`tests/task317_diag.rs`) 제거
- `cargo test` 전체 통과 확인
- 4샘플 무회귀 확인

산출:
- 코드 + `mydocs/working/task_m100_317_stage4.md`
- `mydocs/report/task_m100_317_report.md`
- 오늘할일 갱신

## 회귀 검증 명령

```bash
cargo test --test hwpx_to_hwp_adapter -- --include-ignored
cargo test
for f in samples/{21_언어_기출_편집가능본,exam_math,exam_kor,exam_eng}.hwp; do
  pages=$(cargo run --bin rhwp -q -- dump-pages "$f" 2>/dev/null | grep -c "^=== 페이지")
  echo "$(basename $f): $pages 쪽"
done
# 기대: 15 / 20 / 24 / 9 (변화 없음)
```

## 위험 / 롤백

| 위험 | 대응 |
|------|------|
| origin이 typeset 내부 처리 분기 | 1단계 trace 결과로 typeset 코드 직접 수정 |
| 다중 origin (한 가지 보강으로 부족) | 2단계 binary search 로 모두 식별 |
| h-01/h-03 회귀 | 매 단계 3샘플 동시 검증 |
| 보강 후 직렬화 round-trip 깨짐 | export_hwp_native 라운드트립 확인 |

## 승인 요청

위 분할 승인 시 1단계 시작.
