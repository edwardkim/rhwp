# Task #314 구현 계획서

상위: 수행계획서 `task_m100_314.md`
브랜치: `task314`

## 단계 구성

4단계 분할.

### 1단계 — LINE_SEG 변환 전후 비교 (조사)

목표: hwpx-h-02 의 HWPX 직접 로드 vs 어댑터 변환 후 LINE_SEG 데이터 비교. 차이 정량화. 코드 변경 없음.

작업:
1. 임시 스크립트 또는 cargo test 작성으로 두 경로 LINE_SEG 덤프
2. 페이지/단/문단/줄 레벨 차이 식별
3. paragraph 속성 차이 (sb/sa, page_break, control_mask 등) 도 비교

산출:
- `mydocs/tech/hwpx_adapter_diff_analysis.md`
- `mydocs/working/task_m100_314_stage1.md`

### 2단계 — 차이 origin 식별

목표: 1단계 차이가 어디서 발생하는지 식별 — 어댑터 변환 / HWP 직렬화 / HWP 재파싱 중.

작업:
1. HWPX → IR (직접) IR 덤프
2. HWPX → IR → 어댑터 → IR 비교
3. IR → HWP 직렬화 → HWP → IR 비교 (직렬화/재파싱 손실)
4. 손실 발생 지점 정확히 표시

산출:
- `mydocs/working/task_m100_314_stage2.md`

### 3단계 — 보강 코드 작성

2단계 결과에 따라:
- a) 어댑터 (`hwpx_to_hwp.rs`)에서 추가 변환
- b) HWP 직렬화기에서 누락된 필드 출력
- c) HWP 파서에서 보존 안 되던 필드 복원

각 변경마다 hwpx-h-01/h-02/h-03 동시 검증.

산출:
- 코드 + `mydocs/working/task_m100_314_stage3.md`

### 4단계 — 격리 테스트 재활성화 + 최종 보고

작업:
- `tests/hwpx_to_hwp_adapter.rs` 의 `#[ignore]` 3건 제거
- 사유 메시지도 제거
- `cargo test` 전체 통과 확인
- 4샘플 (21_언어 등) 무회귀 확인 (페이지 수 보존)

산출:
- 코드 + `mydocs/working/task_m100_314_stage4.md`
- `mydocs/report/task_m100_314_report.md`
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
| 차이 origin이 직렬화기 깊숙이 있음 | 보강 범위 확장, 회귀 검증 강화 |
| h-01/h-03 회귀 | 매 단계 3샘플 동시 검증 |
| 보강해도 +1쪽 잔존 | 부분 개선 + 잔존 차이 정량화, 추가 sub-issue |

## 승인 요청

위 분할 승인 시 1단계 시작.
