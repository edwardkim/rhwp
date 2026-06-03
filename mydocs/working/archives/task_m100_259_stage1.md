---
타스크: #259 한글 폰트명 → 메트릭 DB 영문명 매핑 누락
단계: Stage 1 — 매핑 테이블 실측 확정
브랜치: local/task259
작성일: 2026-04-23
---

# Stage 1 완료 보고서

## 1. 목적

수행계획서 §4.1.A 의 HY 7건 매핑이 FONT_METRICS 배열 실체와 일치하는지 실측 확인. 본한글/본명조의 target (Pretendard/Noto Serif KR) 엔트리 존재도 확인.

## 2. 실측 결과

### 2.1 HY 계열 7건 — 모두 DB 에 존재 (1:1 매핑 확정)

| 한글 정규명 | 영문 DB 이름 | DB 엔트리 라인 | 검증 |
|---|---|---|---|
| `HY중고딕` | `HYGothic-Medium` | 9924 | ✅ |
| `HY견고딕` | `HYGothic-Extra` | 9923 | ✅ |
| `HY헤드라인M` | `HYHeadLine-Medium` | 9927 | ✅ |
| `HY견명조` | `HYMyeongJo-Extra` | 9928 | ✅ |
| `HY신명조` | `HYSinMyeongJo-Medium` | 9933 | ✅ |
| `HY그래픽` | `HYGraphic-Medium` | 9925 | ✅ |
| `HY궁서` | `HYGungSo-Bold` | 9926 | ✅ |

엔트리 수 = 각 1개 (모두 Regular 만, bold 변형 DB 에 없음).

### 2.2 HY 계열 DB 에 있는 나머지 28종

Stage 1 의 7개 대표 이름 외에도 DB 에는 **28종의 추가 HY 계열 엔트리** 가 존재:

- `HYPMokGak-Bold`, `HYPost-Light`, `HYPost-Medium`, `HYShortSamul-Medium`
- `HYbdaL`, `HYbdaM`, `HYbsrB`, `HYcysM`, `HYdnkB`, `HYdnkM`, `HYgprM`, `HYgsrB`, `HYgtrE`, `HYhaeseo`, `HYkanB`, `HYkanM`, `HYmjrE`, `HYmprL`, `HYnamB`, `HYnamL`, `HYnamM`, `HYporM`, `HYsanB`, `HYsnrL`, `HYsupB`, `HYsupM`, `HYtbrB`, `HYwulB`, `HYwulM`

이들은 한컴 폰트 파일명의 첫 5~6자 코드 (예: `HYbdaL` = 함초롬바탕 Light 계열 추정) 로, 한글 정규명 대응이 style_resolver 에 없음. **본 이슈 범위 외** — 필요 시 후속 이슈에서 처리.

### 2.3 본한글 계열 target 엔트리 확인

| Target | DB 엔트리 | Regular | Bold | em_size |
|---|---|---|---|---|
| `Pretendard` | ✅ | Line 10251 | Line 10252 | 2048 |
| `Noto Serif KR` | ✅ | Line 9671 | Line 9672 | 1000 |

Regular + Bold 양쪽 엔트리 존재. 본한글 Medium/Regular 요청 시 Regular 매칭, Bold 요청 시 Bold 매칭.

### 2.4 Bold 요청 시 fallback 동작

HY 계열은 DB 에 Bold 변형이 없음. `find_metric("HY중고딕", true, false)` 호출 시:

```rust
// font_metrics_data.rs:98
pub fn find_metric(name: &str, bold: bool, italic: bool) -> Option<MetricMatch> {
    let name = resolve_metric_alias(name);
    // 정확한 매칭 (name + bold + italic) — 실패
    // bold만 매칭 (italic 무시) — 실패
    // Regular 폴백 — bold 요청이었으면 bold_fallback=true 표시
    FONT_METRICS.iter().find(|m| m.name == name)
        .map(|m| MetricMatch { metric: m, bold_fallback: bold })
}
```

→ `HYGothic-Medium` Regular 가 매칭되고 `bold_fallback: true` 반환. 기존 로직 그대로 동작.

## 3. 매핑 확정

### 3.1 HY 계열 (7건)

수행계획서 초안 그대로 확정. 수정 불필요.

### 3.2 본한글 계열

- **본한글 / 본한글vf / 본고딕 / 본고딕vf / Source Han Sans 변형 · Noto Sans CJK KR** → `Pretendard`
- **본명조 / 본명조vf / Source Han Serif 변형 · Noto Serif CJK KR** → `Noto Serif KR`

수정 불필요.

## 4. 회귀 방지 확인

기존 매핑 24건 (`resolve_metric_alias` 현재 본문) 은 그대로 유지. 신규 매핑은 **추가** 만.

단위 테스트에서 기존 매핑 1건 (`함초롬바탕 → HCR Batang`) 의 회귀 방지 확인 포함.

## 5. 다음 단계

Stage 2 — `resolve_metric_alias` 수정 + 단위 테스트 추가.

- 수정 파일: `src/renderer/font_metrics_data.rs`
- 추가 매핑: HY 7 + 본한글 변형 13 + 본명조 변형 10 = **30건 (17 match arm)**
- 단위 테스트: 6건 (`mod tests` 신규)

## 6. 산출물

- 본 문서 (`mydocs/working/task_m100_259_stage1.md`)

Stage 1 완료.
