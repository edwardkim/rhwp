# Task #885 Stage 1 — 누락 별칭 폰트 식별

작성일: 2026-05-15
이슈: [#885](https://github.com/edwardkim/rhwp/issues/885)

## 1. 조사 방법

1. **HWPX**: `samples/**/*.hwpx` 의 `Contents/header.xml` 에서 `face="..."` 추출 (40종 수집)
2. **HWP5**: `samples/*.hwp` + `samples/basic/*.hwp` 를 `rhwp export-svg --font-style` 으로 변환 후 SVG `font-family` 의 첫번째 가족명 추출 (40종 수집)
3. 두 목록 합집합 (64종) 을 `resolve_metric_alias` 별칭과 `FONT_METRICS` DB 직등록명에 대조

원본 자료: `/tmp/task885/hwpx_fonts.txt`, `/tmp/task885/hwp5_used_fonts.txt`, `/tmp/task885/all_used.txt`

## 2. 분류 결과

| 분류 | 개수 |
|------|------|
| ALIAS (별칭 등록됨) | 13 |
| DB_DIRECT (DB에 한국어/원명 직접 등록) | 8 |
| MISSING (별칭/DB 모두 부재) | 43 |

MISSING 그룹은 다음 하위로 분리한다.

### 2.1 본 이슈 핵심 대상 — HY 계열 (별칭 추가 + DB 매핑)

| 한국어 사용명 | DB 영문명 (실재) | 추가 별칭 |
|--------------|----------------|----------|
| **HY수평선B** | `HYsupB` | "HY수평선B" → "HYsupB" |
| **HY수평선M** | `HYsupM` | "HY수평선M" → "HYsupM" |
| **HY울릉도B** | `HYwulB` | "HY울릉도B" → "HYwulB" |
| **HY울릉도M** | `HYwulM` | "HY울릉도M" → "HYwulM" |
| **HY태백B** | `HYtbrB` | "HY태백B" → "HYtbrB" |
| **HY동녘M** | `HYdnkM` | "HY동녘M" → "HYdnkM" |
| **HY각헤드라인M** | (없음) → `HYHeadLine-Medium` (근사) | "HY각헤드라인M" → "HYHeadLine-Medium" |

### 2.2 패턴 확장 — 샘플에는 없으나 DB에 짝이 있어 함께 등록

| 한국어 사용명 | DB 영문명 | 비고 |
|--------------|----------|------|
| HY동녘B | `HYdnkB` | M 짝 |
| HY바다L | `HYbdaL` | 잠재 사용 |
| HY바다M | `HYbdaM` | 잠재 사용 |
| HY간기B | `HYkanB` | 잠재 사용 |
| HY간기M | `HYkanM` | 잠재 사용 |
| HY산B | `HYsanB` | 잠재 사용 |
| HY나무B | `HYnamB` | 잠재 사용 |
| HY나무L | `HYnamL` | 잠재 사용 |
| HY나무M | `HYnamM` | 잠재 사용 |
| HY백송B | `HYbsrB` | 잠재 사용 |
| HY해서 | `HYhaeseo` | 잠재 사용 |
| HY견고딕E | `HYgtrE` | (HY견고딕 = HYGothic-Extra 별도; gtr 약자 의미 불명, 보류) |

> **불확실 매핑 (보류)**: `HYcysM`, `HYmprL` 의 한국어 본명을 자료에서 확정하지 못함. 잘못된 별칭은 폭/높이를 악화시킬 수 있어 본 단계에서는 제외.

### 2.3 본 이슈 범위 외 (별도 후속 이슈 권장)

이슈 #885 본문은 "HY 계열" 폴백 메트릭에 한정한다. 아래는 시각 정합성에 영향을 주지만 본 이슈 범위 외:

- **함초롬돋움 / 함초롬바탕** — 이미 `HCR Dotum / HCR Batang` 별칭 있음 (별칭 파이프 첫 키 형태로 등록되어 검사 스크립트 false negative)
- **Pretendard ExtraBold / Medium** — Pretendard family weight 분기 미지원 (현재 Regular/Bold 만)
- **나눔바른고딕 / 나눔고딕 Light / 산돌고딕B / KoPub돋움체 ?** — NanumGothic/Pretendard 폴백 후보
- **08서울남산체, 한컴 윤고딕/소망/쿨재즈, 양재튼튼체, DX새고딕, HCI Hollyhock/Poppy/Tulip** — 메트릭 DB 자체 추가 필요 (Stage 2 범위 외)
- **신명 디나루, 신명 태명조, 신명 신문명조, 하이텔울릉도제목체, 가는각진제목체** — 메트릭 DB 자체 추가 필요
- **`-윤명조120/150/320`, `Yoon가변 윤고딕 310_TT`, `한컴 윤고딕 230/240/250/760`** — DTP 폰트 별칭/메트릭 추가 필요
- **`Latin Modern Math`, `Tom's Handwriting`, `Garamond` 등** — Latin 폰트 메트릭 폴백

이 항목들은 Task #885 후속으로 별도 이슈에 분리한다.

## 3. Stage 2 별칭 추가 안 (확정)

`src/renderer/font_metrics_data.rs` `resolve_metric_alias` 에 아래 8개 별칭을 추가:

```rust
// HY 계열 추가 (Task #885) — 한국어 사용명 → 메트릭 DB 영문명
"HY수평선B" => "HYsupB",
"HY수평선M" => "HYsupM",
"HY울릉도B" => "HYwulB",
"HY울릉도M" => "HYwulM",
"HY태백B"   => "HYtbrB",
"HY동녘M"   => "HYdnkM",
"HY동녘B"   => "HYdnkB",
"HY각헤드라인M" => "HYHeadLine-Medium",  // 정확한 메트릭 부재, 헤드라인M으로 근사
```

확장 패턴(2.2 후반): 별도 PR/커밋에서 추가 검토. 현재 단계에서는 **샘플 사용 폰트 7종 + 짝 1종 (HY동녘B) = 8종** 으로 최소화.

## 4. 검증 계획 (Stage 2 회귀 테스트)

각 추가 별칭에 대해:

```rust
#[test]
fn task885_alias_resolves_to_db() {
    let cases = [
        ("HY수평선B", "HYsupB"),
        ("HY수평선M", "HYsupM"),
        ("HY울릉도B", "HYwulB"),
        ("HY울릉도M", "HYwulM"),
        ("HY태백B",   "HYtbrB"),
        ("HY동녘M",   "HYdnkM"),
        ("HY동녘B",   "HYdnkB"),
        ("HY각헤드라인M", "HYHeadLine-Medium"),
    ];
    for (input, expected_name) in cases {
        let m = find_metric(input, false, false).expect(input);
        assert_eq!(m.metric.name, expected_name);
    }
}
```

`feedback_font_metrics_alias_sync` Memory rule 준수 — 모든 우변이 `FONT_METRICS` 에 실재함을 8건 모두 확인 완료 (`grep -E "FontMetric { name: \"HY(sup|wul|tbr|dnk)[BM]\"" src/renderer/font_metrics_data.rs`).

## 5. 승인 요청

- 2.1 의 7개 + 2.2 의 1개 (HY동녘B) 별칭 추가 안에 대해 승인을 요청합니다.
- 2.2 의 나머지 패턴 확장 (HY바다, HY간기, HY산B, HY나무, HY백송, HY해서)는 본 타스크에서 함께 추가할지 / 별도 이슈로 분리할지 결정 필요.
- 2.3 범위 외 항목은 별도 이슈로 분리 예정 (Stage 4 보고서에 후속 이슈 권장 명시).
