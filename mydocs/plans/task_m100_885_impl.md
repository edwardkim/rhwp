# 구현 계획서 — Task #885

## 1. 개요

- **수행 계획서**: [`task_m100_885.md`](task_m100_885.md) (승인 완료)
- **브랜치**: `local/task885` (분기점: `stream/devel`)
- **목표**: `resolve_metric_alias` 확장 및 폴백 메트릭 보정으로 한컴 폰트 미설치 환경에서의 시각 정합성 개선

## 2. 단계 구성 (4단계)

### Stage 1: 누락 별칭 폰트 식별

**산출물**: `mydocs/tech/task885_missing_aliases.md`

- `samples/`에서 표/시각 정합성 검증 빈도 높은 HWP/HWPX 1~5개 선정
- HWP5: FaceName 레코드, HWPX: header.xml `<hh:fontface>` 추출하여 한국어 폰트명 목록 수집
- 현재 `resolve_metric_alias` 와 `FONT_METRICS` 테이블 대조 → 별칭 미등록 폰트 분류
  - (A) DB에 영문명 메트릭 존재, 별칭만 누락 → 별칭 추가 대상
  - (B) DB에 메트릭 자체 없음 → 가장 가까운 기존 메트릭(또는 폴백 Pretendard/Noto Serif KR)으로 매핑
- 우선순위 결정: `samples/table-in-tbox.hwp` p1 사용 폰트 우선

**검증**: 식별 폰트 목록 ≥ 5개, 매핑 사유 명시. 작업지시자 승인.

### Stage 2: `resolve_metric_alias` 확장 + 회귀 테스트

**산출물**:
- `src/renderer/font_metrics_data.rs` — `resolve_metric_alias` 별칭 추가
- 회귀 테스트 (모듈 내부 `#[test]` 또는 `tests/font_alias.rs`)

**구현**:
- Stage 1 식별 폰트 별칭 추가
- 각 별칭에 대해 `find_metric()` 호출 결과가 `None` 이 아님을 단위 테스트로 보장
- Memory rule `feedback_font_metrics_alias_sync` 준수 — 모든 별칭 우변(영문명)이 `FONT_METRICS`에 실재함을 확인

**검증**:
```
cargo test --release font_alias
cargo clippy --release -- -D warnings
```

### Stage 3: 시각 정합 측정 + 효과 보고

**산출물**: `mydocs/working/task_m100_885_stage3.md`

- 변경 전/후 SVG 내보내기:
  ```
  ./target/release/rhwp export-svg samples/table-in-tbox.hwp -o output/svg/task885_before/
  # 빌드 적용 후
  ./target/release/rhwp export-svg samples/table-in-tbox.hwp -o output/svg/task885_after/
  ```
- `pdf/table-in-tbox-2022.pdf` 와 RMSE 비교 (스크립트는 #696 보고서 절차 재사용)
- 추가 샘플 1~2개 (표 다수 포함) 동일 측정 → 회귀 없는지 확인
- 개선폭 ≥ 5% 미달 시: 옵션 2(자동 임베딩) 분리 이슈 제안서를 보고서에 포함

**검증**: RMSE 수치, 비교 스크린샷, 작업지시자 승인.

### Stage 4: 측정 방법론 문서 + 최종 보고

**산출물**:
- `mydocs/tech/font_diff_rmse_normalization.md` — RMSE 측정 시 폰트 차이 보정 절차 (옵션 3)
- `mydocs/report/task_m100_885_report.md` — 최종 결과 보고서
- `mydocs/orders/{오늘}.md` 갱신 (Task #885 상태 → 완료)

**검증**:
- `cargo test --release` 전체 통과
- 작업지시자 승인 후 issue close

## 3. 변경 파일 예상

| 파일 | 변경 유형 |
|------|---------|
| `src/renderer/font_metrics_data.rs` | `resolve_metric_alias` match 절 확장 |
| `src/renderer/font_metrics_data.rs` 또는 `tests/font_alias.rs` | 회귀 테스트 추가 |
| `mydocs/tech/task885_missing_aliases.md` | 신규 (조사 결과) |
| `mydocs/tech/font_diff_rmse_normalization.md` | 신규 (선택) |
| `mydocs/working/task_m100_885_stage1~4.md` | 신규 (단계별 보고) |
| `mydocs/report/task_m100_885_report.md` | 신규 (최종 보고) |

## 4. 위험 관리

- **잘못된 별칭 매핑 위험**: 한국어 패밀리명을 무관한 영문 메트릭에 매핑하면 폭/높이가 더 부정확해질 수 있음.
  - 대응: Stage 1 매핑 사유 문서화 + Stage 3 회귀 측정에서 RMSE 악화 시 매핑 철회.
- **IR 변경 없음 가정**: 메트릭 별칭은 렌더링 단계에서만 영향. `dump`/`ir-diff` 결과는 변하지 않아야 함.
  - 대응: Stage 3 검증 단계에서 `ir-diff` 출력 비교 확인.
- **Stage 3 효과 미흡 위험**: 메트릭 보정만으로 RMSE 개선폭이 작을 경우.
  - 대응: 보고서에 한계 명시하고 옵션 2 분리 이슈로 후속 처리.

## 5. 승인 요청

본 4단계 구현 계획 승인 후 Stage 1 부터 진행합니다.
