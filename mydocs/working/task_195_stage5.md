# Task #195 단계 5 완료보고서 — 검증 및 마무리

> 구현계획서: [task_195_impl.md](../plans/task_195_impl.md)
> 단계: 5 / 5

## 작업 결과

### 검증 (회귀 테스트)

**단위 테스트**:
```
cargo test --release --lib:  878 passed; 0 failed; 1 ignored
cargo test --release --tests: 13 passed; 0 failed
```

**export-svg 회귀** (shape 포함 샘플):
```
samples/draw-group.hwp  → 1 SVG  OK
samples/aift.hwp        → 74 SVG OK
samples/biz_plan.hwp    → 6 SVG  OK
```
크래시/오류 없음.

**1.hwp(로컬 검증 파일, 저작권으로 samples/ 제외)**:
- 이전: 2개 OLE 차트가 완전 빈 사각형으로 렌더
- 현재: 연한 회색(#F0F0F0) + 점선 테두리로 식별 가능한 placeholder

### 자체 제작 샘플 — 보류

한컴오피스 authoring 환경 부재로 단계 5 계획의 `samples/chart-basic.hwp` 자체 제작은 **별도 작업**으로 분리. 기존 samples/ 137개 회귀 테스트로 갈음.

### 오늘 할일 갱신

`mydocs/orders/20260419.md`에 #195 완료 항목과 커밋 이력 추가, 후속 이슈 2건 신규 제안 기입.

## 전체 성과 요약

| 항목 | 이전 | 현재 |
|------|------|------|
| OLE 개체 | 빈 사각형 (Rectangle 폴백) | placeholder 회색 + 점선 |
| HWP 네이티브 차트 | 동일 폴백 | Chart variant 별도 분기 (raw 보존) |
| ShapeObject enum | 8 variants | 10 variants (+Chart +Ole) |
| 라운드트립 | 정보 손실 | raw_chart_data/raw_tag_data로 보존 |
| 단위 테스트 | 875 | 878 (+3 OLE) |

## 분리된 후속 이슈 제안

1. **OLE 프리뷰 이미지 추출**: BinData 스트림(`ec...` 매직) 압축 해제 + 중첩 CFB 파싱 + WMF/EMF→SVG 변환
2. **CHART_DATA 하위 태그 구조화 파싱**: ChartType/Series/Axis 실제 렌더

## 남은 프로세스

1. 작업지시자 최종 보고서 승인
2. `local/task195` → `local/devel` merge (`--no-ff`)
3. `local/devel` → `origin/devel` push
4. GitHub Issue #195 close

## 커밋 대상

- mydocs/orders/20260419.md (갱신)
- mydocs/working/task_195_stage5.md (본 문서)
- mydocs/working/task_195_report.md (최종 보고서, 단계 5와 함께 커밋)

**커밋 메시지**: `Task #195: 검증 + 최종 보고서 (단계 5)`
