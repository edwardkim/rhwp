# Task #296 Stage 3 보고서 — 검증

## 목표

1. WASM Docker 빌드 → 브라우저 시각 검증
2. 진단 로그 제거 후 회귀 재확인

## WASM 빌드

```bash
docker compose --env-file .env.docker run --rm wasm
```

결과:
- `pkg/rhwp_bg.wasm` 4,089,055 bytes (이전 4,085,137 대비 +3,918 bytes)
- Docker 빌드 시간 34.40s + wasm-opt 1m 30s = 약 2분
- 크기 증가 = `WasmTextMeasurer` 의 inline_tabs 분기 추가분 반영 확인

## 브라우저 시각 검증

**대상**: `samples/exam_math.hwp` 페이지 7, 18번 "수열" 문항

| 항목 | Before (PR #292 merge 직후, Canvas 경로) | After (이번 Task #296 WASM 수정) |
|------|------------------------------------------|-----------------------------------|
| `18.` 과 `수열` 사이 공백 | 거대한 공백 (~290px 로 "수열" 밀림) | 정상 — `18. 수열 {a_n}이 모든 자연수 n에 대하여` |
| 작업지시자 판정 | 밀림 | **성공** |

## 진단 로그 제거

Stage 1~2 에서 추가한 진단 로그 4개 (`EmbeddedTextMeasurer` estimate 3개 + compute 1개) 모두 제거.

`grep "TAB296\|_diag\|_total_before\|_x_before" src/renderer/layout/text_measurement.rs` → 0 hit 확인.

## 로그 제거 후 회귀 재검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib task296` | ✅ 4 passed |
| `cargo test --test svg_snapshot` | ✅ 6 passed |
| `cargo test --test tab_cross_run` | ✅ 1 passed |
| `cargo clippy --lib -- -D warnings` | ✅ clean |
| `cargo check --target wasm32-unknown-unknown --lib` | ✅ clean |

## 최종 변경 범위 (Stage 3 종료)

| 파일 | 변경 | 비고 |
|------|------|------|
| `src/renderer/layout/text_measurement.rs` | +51 -0 | 헬퍼 `inline_tab_type` + `WasmTextMeasurer` 2곳 inline_tabs 분기 + 네이티브 측 주석 추가 |
| `src/renderer/layout/tests.rs` | +32 | task296 단위 테스트 4건 |

## 다음 단계

- Stage 4: 최종 보고서 + orders 갱신 + 트러블슈팅 문서 갱신
