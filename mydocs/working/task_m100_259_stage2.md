# Task #259 Stage 2 완료 보고서 — resolve_metric_alias 매핑 추가

- 일자: 2026-04-23
- 브랜치: `local/task259`
- 작업: `font_metrics_data.rs::resolve_metric_alias` 에 HY 계열 7종 매핑 추가 + 단위 테스트

## 변경 파일

`src/renderer/font_metrics_data.rs`

### 1. `resolve_metric_alias` 에 7 arm 추가 (line 93~ 부근)

```rust
// HY 계열 한글 정규명 → 메트릭 DB 영문명 (Task #259)
"HY중고딕" => "HYGothic-Medium",
"HY견고딕" => "HYGothic-Extra",
"HY견명조" => "HYMyeongJo-Extra",
"HY신명조" => "HYSinMyeongJo-Medium",
"HY그래픽" => "HYGraphic-Medium",
"HY헤드라인M" => "HYHeadLine-Medium",
"HY궁서" | "HY궁서B" => "HYGungSo-Bold",
```

### 2. 모듈 하단에 `#[cfg(test)] mod tests` 추가

7종 모두에 대해:
- `resolve_metric_alias(한글)` 이 기대 영문명 반환
- `find_metric(한글, false, false)` 가 `Some` 반환
- 반환된 metric.name 이 기대 DB 영문명과 일치

## 검증 결과

| 검증 | 결과 |
|------|------|
| `cargo test --lib renderer::font_metrics_data::tests` | ✅ 1 passed (HY 7종 전부 통과) |
| `cargo test --lib` (전체) | ✅ 948 passed · 1 ignored · 0 failed |
| `cargo test --test svg_snapshot` | ✅ 3 passed (table_text / form_002 / determinism) |
| `cargo clippy --lib --tests` | ✅ 기존 warning 만 (신규 경고 0) |

## 회귀 영향

- 신규 alias 7종은 기존 HWP 경로를 바꾸지 않는다 (이전에 `None` 반환하던 케이스 → 이제 `Some` 반환).
- SVG 스냅샷(table_text / form_002) 불변 — 해당 샘플은 HY 계열 폰트를 사용하지 않음 추정. Stage 3 에서 `text-align.hwp` 로 실제 HY 경로 회귀 확인 예정.

## 미해결 사항

없음. Stage 3 (text-align.hwp 실제 렌더링 회귀) 에서 기능적 재현/해소 검증.

## 산출물

- `src/renderer/font_metrics_data.rs` (+~20 lines: 7 alias + test module)
- 본 보고서

## 승인 요청

Stage 2 완료 승인 및 Stage 3 착수 승인을 요청드립니다.
