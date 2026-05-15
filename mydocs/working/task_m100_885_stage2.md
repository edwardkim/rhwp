# Task #885 Stage 2 — 완료 보고서

작성일: 2026-05-15
브랜치: `local/task885`

## 1. 변경 사항

### 1.1 `src/renderer/font_metrics_data.rs` — `resolve_metric_alias` 확장

HY 계열 8개 별칭 추가 (Stage 1 식별 결과):

```rust
"HY수평선B"   => "HYsupB",
"HY수평선M"   => "HYsupM",
"HY울릉도B"   => "HYwulB",
"HY울릉도M"   => "HYwulM",
"HY태백B"     => "HYtbrB",
"HY동녘B"     => "HYdnkB",
"HY동녘M"     => "HYdnkM",
"HY각헤드라인M" => "HYHeadLine-Medium",  // 정확 메트릭 부재, 헤드라인M 근사
```

기존 7개 HY 별칭 (HY중고딕/HY견고딕/HY헤드라인M/HY견명조/HY신명조/HY그래픽/HY궁서) 직하에 배치.

### 1.2 회귀 테스트

`tests` 모듈에 `task885_hy_extra_aliases_resolve` 함수 추가. 8개 별칭 모두에 대해:
- `find_metric()` 이 `Some` 반환
- `metric.name` 이 기대 DB 영문명과 일치

Memory rule `feedback_font_metrics_alias_sync` 준수 — 우변 메트릭 8개 모두 `FONT_METRICS` 에 실재함을 테스트로 보장.

## 2. 검증 결과

| 검증 | 결과 |
|------|------|
| `cargo test --release --lib -- task885` | 1 passed |
| `cargo test --release --lib -- font` (회귀) | 33 passed, 0 failed |
| `cargo clippy --release --lib -- -D warnings` | 통과 |

## 3. 영향 범위

- 메트릭 별칭만 추가. IR 변경 없음 → `dump`/`ir-diff` 출력 무영향 (Stage 3 에서 확인 예정).
- 한컴 폰트 미설치 환경에서 8종 한국어 폰트 face 이름이 들어오면 폴백 메트릭이 적용되어 폭/높이 근사가 개선됨.

## 4. 다음 단계 (Stage 3)

- 변경 전/후 `samples/table-in-tbox.hwp` p1 SVG 비교
- PDF (`pdf/table-in-tbox-2022.pdf`) 와 RMSE 측정 → 개선폭 보고
- 추가 샘플 1~2개 회귀 확인
- `ir-diff` 변경 없음 확인

승인 후 Stage 3 진행.
