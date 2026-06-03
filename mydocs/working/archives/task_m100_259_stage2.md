---
타스크: #259 한글 폰트명 → 메트릭 DB 영문명 매핑 누락
단계: Stage 2 — `resolve_metric_alias` 수정 + 단위 테스트
브랜치: local/task259
작성일: 2026-04-23
---

# Stage 2 완료 보고서

## 1. 목적

Stage 1 에서 확정한 매핑 30건 (HY 7 + 본한글 13 + 본명조 10) 을 `resolve_metric_alias` 에 반영하고, 매핑 검증 · 회귀 방지용 단위 테스트 6건을 추가.

## 2. 변경 사항

### 2.1 `src/renderer/font_metrics_data.rs` — `resolve_metric_alias` 확장

- 매핑 정책 설명 주석 블록 추가 (2-계층 별칭 시스템 · 근사 근거 · 한계 · 후속 이슈)
- HY 계열 7건 match arm 추가
- 본한글 계열 13건 (vf variants · Source Han Sans 공식명 · Noto Sans CJK KR) → `Pretendard` 하나의 arm
- 본명조 계열 10건 → `Noto Serif KR` 하나의 arm

총 **17 match arm 추가**, 기존 24건 정책 보존.

### 2.2 `src/renderer/font_metrics_data.rs` — `#[cfg(test)] mod tests` 신규

6건 단위 테스트:

| # | 테스트명 | 검증 대상 |
|---|---|---|
| 1 | `hy_gothic_medium_maps_correctly` | HY중고딕 → HYGothic-Medium 정확 매칭 |
| 2 | `hy_family_all_map` | HY 7건 전체 loop 검증 |
| 3 | `source_han_sans_family_maps_to_pretendard` | 본한글/본고딕/Source Han Sans/Noto Sans CJK KR 13건 → Pretendard |
| 4 | `source_han_serif_family_maps_to_noto_serif_kr` | 본명조/Source Han Serif/Noto Serif CJK KR 10건 → Noto Serif KR |
| 5 | `hy_family_bold_fallback` | HY 계열 bold 요청 시 Regular 매칭 + `bold_fallback=true` |
| 6 | `non_korean_font_unchanged` | 함초롬바탕 → HCR Batang 기존 매핑 회귀 방지 |

## 3. 검증 결과

### 3.1 단위 테스트

```
cargo test --lib font_metrics_data

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 948 filtered out
```

6건 전부 그린.

### 3.2 전체 lib 테스트

```
cargo test --lib

test result: ok. 953 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

기존 947 + 신규 6 = 953, 모두 그린. 회귀 0.

### 3.3 clippy

```
cargo clippy --lib -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s)
```

Clean.

## 4. 영향 범위

### 4.1 런타임 동작 변화

`find_metric("HY중고딕", …)` 기준:

| Before | After |
|---|---|
| `resolve_metric_alias("HY중고딕")` = `"HY중고딕"` (passthrough) | `"HYGothic-Medium"` |
| FONT_METRICS 에서 "HY중고딕" 검색 → None | FONT_METRICS 에서 "HYGothic-Medium" 검색 → Some |
| 기본 폭 fallback (글자 겹침) | DB 실측 폭 사용 (정상 렌더) |

본한글/본명조도 동일 경로로 Pretendard/Noto Serif KR 메트릭 사용.

### 4.2 잠재 영향

- 기존에 기본 폭 fallback 으로 렌더되던 문서는 폭이 **변경** 됨 → svg_snapshot golden 재생성 필요 가능 (Stage 3 에서 확인)
- 회귀 방지 체계: 단위 테스트 6건 + `non_korean_font_unchanged` 로 기존 매핑 보존 확인

## 5. 다음 단계

Stage 3 — 회귀 검증:

- `cargo test --test svg_snapshot` — form-002 · table-text · determinism 영향 여부 확인, 필요 시 `UPDATE_GOLDEN=1` 재생성
- `samples/text-align.hwp` 렌더 → 4번 문단 (HY중고딕) 글자 겹침 해소 시각 확인
- WASM 재빌드 후 작업지시자 웹 에디터 확인

## 6. 산출물

- `src/renderer/font_metrics_data.rs` 수정 (resolve_metric_alias 확장 + 단위 테스트 모듈 신규)
- 본 문서 (`mydocs/working/task_m100_259_stage2.md`)

Stage 2 완료.
