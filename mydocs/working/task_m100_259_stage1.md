# Task #259 Stage 1 완료 보고서 — HY 매핑 테이블 확정

- 일자: 2026-04-23
- 브랜치: `local/task259`
- 작업: HY 한글 정규명 → 메트릭 DB 영문명 매핑 확정

## 수행 요약

1. `src/renderer/style_resolver.rs` 를 grep 하여 실제로 출력되는 HY 정규명 6종 특정.
2. 각 정규명에 대응하는 DB 영문명을 한컴 폰트 PS name 관례 + 공개 폰트 메타데이터(Fontke / Wfonts / Fontsgeek / Koreafont) 교차 확인으로 확정.
3. 작업지시자 요청으로 `HY궁서` / `HY궁서B` 방어적 매핑 1종 추가 → 총 **7종**.
4. 근거 문서 `mydocs/tech/task_259_hy_mapping.md` 작성.

## 확정 매핑 (7종)

| 정규명 | DB 영문명 | 근거 |
|--------|----------|------|
| HY중고딕 | HYGothic-Medium | 한컴 폰트 PS name 관례 |
| HY견고딕 | HYGothic-Extra | 한컴 폰트 PS name 관례 (Extra = 견) |
| HY견명조 | HYMyeongJo-Extra | 한컴 폰트 PS name 관례 (Extra = 견) |
| HY헤드라인M | HYHeadLine-Medium | 직역 |
| HY신명조 | HYSinMyeongJo-Medium | Fontke/Wfonts/Fontsgeek 공개 폰트 메타 (신문명조와 구분) |
| HY그래픽 | HYGraphic-Medium | Fontke/Koreafont 공개 폰트 메타 (한양정보통신) |
| HY궁서 / HY궁서B | HYGungSo-Bold | 한컴 궁서는 Bold 단일 weight · 작업지시자 요청(Stage 2 중) 으로 방어적 추가 |

## 수행계획 대비 변경점

- 원 수행계획서(`task_m100_259.md`)의 7종 중 `HY궁서` 는 style_resolver 가 실제로 출력하지 않음(`한양궁서 → 궁서 → Gungsuh` 경로)을 확인. 초기에는 6종으로 축소했으나, Stage 2 진행 중 작업지시자 요청으로 **방어적 매핑**(HWP 파일에 `HY궁서`/`HY궁서B` 가 직접 저장된 케이스 대비)을 추가하여 최종 **7종** 으로 확정.
- HY신명조/HY그래픽은 한컴 관례만으로는 불확실했으나 외부 공개 폰트 메타(Fontke/Wfonts/Koreafont) 교차 확인으로 확신도 100% 격상.

## 소스 수정

없음. Stage 1 은 조사 단계.

## 산출물

- [`mydocs/tech/task_259_hy_mapping.md`](../tech/task_259_hy_mapping.md) — 매핑표 + 근거
- 본 보고서

## 다음 단계 (Stage 2)

`src/renderer/font_metrics_data.rs::resolve_metric_alias` 에 6종 매핑 추가 + 단위 테스트.

## 승인 요청

Stage 1 완료 승인 및 Stage 2 착수 승인을 요청드립니다.
