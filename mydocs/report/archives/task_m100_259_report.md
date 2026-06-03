---
타스크: #259 한글 폰트명 → 메트릭 DB 영문명 매핑 누락 (HY 계열 + 본한글 계열)
브랜치: local/task259
작성일: 2026-04-23
상태: 완료
---

# 최종 보고서

## 1. 배경 요약

작업지시자가 `samples/text-align.hwp` 4번 문단 (HY중고딕) 을 웹 에디터에서 확인 중 **글자 겹침** 현상 발견. 디버깅 결과 한컴 한국어 폰트명 → 메트릭 DB 영문명 매핑이 누락되어 `find_metric` 이 None 을 반환 → 기본 폭 (fallback) 사용 → 영문/숫자 글자 간격이 좁게 겹치는 것이 원인.

작업지시자 지시로 본한글/본한글vf (Source Han Sans 계열) · 본명조 (Source Han Serif 계열) 도 함께 조사. 조사 결과 본한글 메트릭 자체가 DB 에 부재 → **Pretendard / Noto Serif KR 근사 매핑** 전략 채택 (한글 원천 동일 · OFL 호환 · 이미 번들).

## 2. 문제 본질 재확인

rhwp 의 폰트 이름 해석은 2-계층:

1. **`style_resolver.rs`**: 한국어 별칭 → 한국어 정규명 (예: 한양중고딕 → HY중고딕) — 이미 구현
2. **`font_metrics_data.rs::resolve_metric_alias`**: 한국어 정규명 → 영문 DB 이름 (예: HY중고딕 → HYGothic-Medium) — **여기 누락**

2계층 중 두 번째 단계가 HY / 본한글 계열에 대해 미구현이었던 것이 본 이슈의 근본 원인.

## 3. 변경 사항

### 3.1 코드

`src/renderer/font_metrics_data.rs::resolve_metric_alias` 에 **17 match arm 추가**:

| 계열 | 한글 정규명 | → 영문 DB 이름 | 대상 엔트리 |
|---|---|---|---|
| HY 정식 매핑 (7건) | HY중고딕 | HYGothic-Medium | DB 존재 |
| | HY견고딕 | HYGothic-Extra | DB 존재 |
| | HY헤드라인M | HYHeadLine-Medium | DB 존재 |
| | HY견명조 | HYMyeongJo-Extra | DB 존재 |
| | HY신명조 | HYSinMyeongJo-Medium | DB 존재 |
| | HY그래픽 | HYGraphic-Medium | DB 존재 |
| | HY궁서 | HYGungSo-Bold | DB 존재 |
| 본한글 근사 (13건, 1 arm) | 본한글 / 본한글vf / 본한글 Medium / 본한글M / 본고딕 / 본고딕vf / Source Han Sans (K/KR) / SourceHanSans(K/KR) / Noto Sans CJK KR | Pretendard | DB 존재 (한글 원천 동일) |
| 본명조 근사 (10건, 1 arm) | 본명조 / 본명조vf / 본명조M / Source Han Serif (K/KR) / SourceHanSerif(K/KR) / Noto Serif CJK KR | Noto Serif KR | DB 존재 (serif 원천) |

또한 함수 상단에 2-계층 해석 체계 · 근사 매핑 근거 · 한계 · 후속 이슈 안내를 명시하는 doc comment 추가.

### 3.2 테스트

동일 파일 내 `#[cfg(test)] mod tests` 신규 추가:

| # | 테스트명 | 검증 |
|---|---|---|
| 1 | hy_gothic_medium_maps_correctly | HY중고딕 기본 매칭 |
| 2 | hy_family_all_map | HY 7건 loop 검증 |
| 3 | source_han_sans_family_maps_to_pretendard | 본한글 계열 13건 → Pretendard |
| 4 | source_han_serif_family_maps_to_noto_serif_kr | 본명조 계열 10건 → Noto Serif KR |
| 5 | hy_family_bold_fallback | HY bold 요청 시 Regular 폴백 + bold_fallback=true |
| 6 | non_korean_font_unchanged | 함초롬바탕 기존 매핑 회귀 방지 |

## 4. 검증 결과

### 4.1 자동화

| 게이트 | 결과 |
|---|---|
| `cargo test --lib` | **953 passed** (기존 947 + 신규 6), 0 failed |
| `cargo clippy --lib -- -D warnings` | clean |
| `cargo test --test svg_snapshot` | 3 passed (golden 재생성 불필요) |

### 4.2 시각 검증 (text-align.hwp)

- 4번 문단 (HY중고딕, 지자체 열거) 의 숫자 advance 폭: **7.67px → 9.04px** (기본 fallback → HYGothic-Medium DB 실측)
- 변경된 `<text>` 요소의 `font-family` 속성 전수 점검 → **모두 HY중고딕 단일**. 다른 폰트 영향 0
- 3-way 비교 페이지: `output/svg/text-align-259-compare.html`

### 4.3 스모크 스위프 (Stage 4)

| 샘플 | 페이지 | 적용된 HY 폰트 | 비-HY 페이지 회귀 |
|---|---|---|---|
| `samples/exam_kor.hwp` | 25 | HY중고딕/견고딕/견명조/신명조 | — (전 페이지 HY 사용) |
| `samples/biz_plan.hwp` | 6 | HY헤드라인M (p3~6) | **p1~2 (함초롬바탕만) 바이트 동일** |
| `samples/hwpx/hwpx-02.hwpx` | 6 | HY헤드라인M (p2, p4, p5) | **p1, p3, p6 (맑은 고딕/바탕만) 바이트 동일** |

비-HY 폰트 단독 페이지 **10 페이지 전수가 바이트 수준 동일** → 기존 매핑 회귀 0 증명.

## 5. 한계 (문서화)

본한글/본명조 근사 매핑의 한계:

1. **Latin 폭 미세 차이**: Pretendard Latin 은 Inter 기반, 본한글 Latin 과 상이. 한글/CJK 는 정확, Latin 만 근사.
2. **Weight 2단계 근사**: Pretendard 메트릭은 Regular + Bold 2단계만 제공. 본한글 ExtraLight/Light/Medium/Heavy 나 본한글vf 의 임의 wght 요청은 Regular/Bold 중 가까운 쪽으로 근사. CJK 는 weight 별 한글 폭 차이가 작아 실무 허용.
3. **정식 DB 엔트리 추가는 별도 이슈**: 본한글 TTF 를 `extract_metrics` 파이프라인으로 정식 FONT_METRICS 엔트리화하는 작업은 범위 외.

## 6. 산출물 목록

### 코드
- `src/renderer/font_metrics_data.rs` — resolve_metric_alias 17 arm 추가 + `mod tests` 신규

### 문서
- `mydocs/plans/task_m100_259.md` — 수행계획서
- `mydocs/plans/task_m100_259_impl.md` — 구현계획서
- `mydocs/working/task_m100_259_stage1.md` ~ `_stage4.md` — 단계별 보고서 4건
- `mydocs/report/task_m100_259_report.md` — 본 문서
- `mydocs/tech/font_fallback_strategy.md` — 부록 갱신 (Pretendard 근사 정책)

### 시각 증거
- `output/svg/text-align-fix-after/text-align.svg`
- `output/svg/text-align-259-compare.html` — 3-way 비교 (Before / PR #256 / #259 fix)
- `output/svg/stage4-{exam-kor,biz-plan,hwpx-02}-{before,after}/` — 스모크 회귀 증거 (37 페이지 × 2)

### 빌드
- WASM 재빌드 완료 (pkg/rhwp_bg.wasm 4,076,166 bytes)

## 7. 메모리 체크리스트 (후속 작업 재발 방지)

신규 메모리 엔트리 `feedback_font_alias_sync.md` 에:

> 한글 폰트 추가 시 `src/renderer/font_metrics_data.rs::resolve_metric_alias` 에도 매핑 동기화 필수.
> 누락 시 `find_metric` 이 None 반환 → 기본 폭 사용 → 글자 겹침.
> 2-계층 체계: style_resolver (별칭 → 정규명) + font_metrics_data (정규명 → 영문 DB 이름).

## 8. 결론

- 목표 달성: HY 7종 정확 매핑 + 본한글/본명조 근사 매핑 구현
- 회귀 0: 단위 테스트 + svg_snapshot + 스모크 스위프 3중 검증 완료
- 시각 겹침 해소: text-align.hwp 4번 문단에서 HY중고딕 글자 폭 정상화 확인
- 한계 명시: 근사 매핑의 Latin 폭 · weight 축 한계는 코드 주석 + 문서에 기록
- 후속: 본한글 TTF 정식 DB 엔트리화는 별도 이슈로 분리

Task #259 종료 승인 요청.
