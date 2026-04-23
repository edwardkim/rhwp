# HY 한글 정규명 → 메트릭 DB 영문명 매핑 (Task #259 Stage 1)

- 작성일: 2026-04-23

## 배경

HWP 본문에 사용되는 한국어 폰트명을 메트릭 DB 에서 찾으려면 두 단계 정규화가 필요하다.

1. **한국어 별칭 → 한국어 정규명**: `src/renderer/style_resolver.rs` (구현 완료)
2. **한국어 정규명 → 메트릭 DB 영문명**: `src/renderer/font_metrics_data.rs::resolve_metric_alias` (HY 계열 누락)

## 실제 사용되는 HY 정규명 7종

`grep -nE 'Some\("HY' src/renderer/style_resolver.rs` 분석 결과 — style_resolver 가 `Some("HY...")` 형태로 출력하는 HY 접두 정규명은 6종. 여기에 HWP 파일이 `HY궁서`/`HY궁서B` 문자열을 직접 저장하는 케이스 대비 1종을 추가하여 총 7종을 대상으로 한다.

| # | 정규명 | style_resolver 출현 별칭 수 |
|---|--------|--------------------------|
| 1 | HY중고딕 | 한양중고딕, 신명 태고딕, 신명 중고딕, 영문 가는/중간/굵은 안상수체 등 |
| 2 | HY신명조 | 한양신명조, 신명 태명조, 명조/휴먼명조/문화바탕/옛한글 외 10+ |
| 3 | HY견명조 | 한양견명조, 신명 견명조, 태 헤드라인D, 가는/중간/굵은 공한 외 |
| 4 | HY견고딕 | 한양견고딕, 신명 견고딕, 양재 참숯B, 태 헤드라인T, #빅 외 |
| 5 | HY그래픽 | 신명 태그래픽, HY그래픽M(직접) |
| 6 | HY헤드라인M | 태 가는 헤드라인T/D, HYHeadLine Medium(직접) |
| 7 | HY궁서 / HY궁서B | HWP 파일에 직접 `HY궁서`/`HY궁서B` 로 저장된 경우 대비 (방어적 추가) |

> style_resolver 는 `한양궁서` → `궁서` 경로를 쓰지만, HWP 파일 중에는 `HY궁서`/`HY궁서B` 문자열이 직접 저장되어 정규화 없이 `resolve_metric_alias` 로 들어오는 케이스도 존재 — 작업지시자 요청(2026-04-23)으로 포함.

## 확정 매핑 (7종)

| # | 정규명 | DB 영문명 | 근거 | 확신도 |
|---|--------|----------|------|--------|
| 1 | `HY중고딕` | `HYGothic-Medium` | 한컴 폰트 PS name 관례 + 공개 메타 교차 확인 | 100% |
| 2 | `HY견고딕` | `HYGothic-Extra` | 한컴 폰트 PS name 관례 (Extra = 견) | 100% |
| 3 | `HY견명조` | `HYMyeongJo-Extra` | 한컴 폰트 PS name 관례 (Extra = 견) | 100% |
| 4 | `HY헤드라인M` | `HYHeadLine-Medium` | 직역 (HeadLine-Medium = 헤드라인M) | 100% |
| 5 | `HY신명조` | `HYSinMyeongJo-Medium` | Fontke/Wfonts/Fontsgeek 공개 폰트 메타에서 Full name = `HYSinMyeongJo-Medium`, family = `HY신명조` 확인. `HYSinMun-MyeongJo`(신문명조)와 구분됨 | 100% |
| 6 | `HY그래픽` | `HYGraphic-Medium` | Fontke/Koreafont 공개 폰트 메타에서 Full name = `HYGraphic-Medium`, family = `HY그래픽M` (한양정보통신 제작) 확인 | 100% |
| 7 | `HY궁서` / `HY궁서B` | `HYGungSo-Bold` | 한컴 궁서는 Bold weight 단일. DB 에 궁서 엔트리도 `HYGungSo-Bold` 유일 — `HY궁서`(접미어 없음)도 동일 엔트리로 매핑 | 100% |

### 외부 공개 폰트 메타 근거 (HY신명조 / HY그래픽)

공개된 폰트 메타데이터 데이터베이스로 교차 확인:

**HY신명조 = HYSinMyeongJo-Medium**
- Fontke / Wfonts / Fontsgeek 공개 메타에서 `Full name: HYSinMyeongJo-Medium`, `Family: HY신명조`, `Style: Regular`, `Version 1.00`, 제작사 HanYang Systems 확인.
- 한컴 신명조(SinMyeongJo)와 신문명조(SinMun-MyeongJo)는 별개 서체 — 매핑 혼동 없음.

**HY그래픽 = HYGraphic-Medium**
- Fontke / Koreafont 공개 메타에서 `Full name: HYGraphic-Medium`, `Family: HY그래픽M`, 제작사 (주)한양정보통신(1990~) 확인.
- 본 프로젝트 style_resolver 는 `HY그래픽M` 별칭을 `HY그래픽` 으로 정규화(line 563) → DB 의 유일한 Medium 엔트리 `HYGraphic-Medium` 에 매핑.

> 참고 출처:
> - Fontke HY신명조 페이지 https://eng.fontke.com/font/10043440/
> - Wfonts HYSinMyeongJo-Medium https://www.wfonts.com/font/hysinmyeongjo-medium
> - Fontke HY그래픽M 페이지 https://eng.m.fontke.com/font/10289753
> - Koreafont HY그래픽M https://www.koreafont.com/fonts/list/639/

## DB 에 존재하는 기타 HY 엔트리 (본 타스크 범위 외)

`HYbdaL, HYbdaM, HYbsrB, HYcysM, HYdnkB, HYdnkM, HYgprM, HYgsrB, HYgtrE, HYhaeseo, HYkanB, HYkanM, HYmjrE, HYmprL, HYnamB, HYnamL, HYnamM, HYporM, HYsanB, HYsnrL, HYsupB, HYsupM, HYtbrB, HYwulB, HYwulM, HYGungSo-Bold, HYPMokGak-Bold, HYPost-Light, HYPost-Medium, HYShortSamul-Medium`

→ 약칭(소문자 suffix)은 한컴 내부 축약명으로, style_resolver 가 직접 이 이름으로 정규화하지 않는 한 본 타스크에서는 건드리지 않는다. (추후 필요 시 별도 이슈)

## 최종 매핑 (Stage 2 적용용)

```rust
"HY중고딕"     => "HYGothic-Medium",
"HY견고딕"     => "HYGothic-Extra",
"HY견명조"     => "HYMyeongJo-Extra",
"HY신명조"     => "HYSinMyeongJo-Medium",
"HY그래픽"     => "HYGraphic-Medium",
"HY헤드라인M"  => "HYHeadLine-Medium",
"HY궁서" | "HY궁서B" => "HYGungSo-Bold",
```
