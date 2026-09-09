# 최종 결과보고서 — Task M100 #2156

**날짜**: 2026-07-10 / **브랜치**: `local/task2156` / **이슈**: #2156

## 1. 요약

문자폭 사다리 통제 프로브(#2150 줄높이 공식 역이용)로 **한글이 함초롬바탕(HCR Batang)
문서의 비한글 문자(라틴·숫자·구두점·U+00B7)를 HCR hmtx 가 아닌
Haansoft Batang(한컴바탕) 메트릭으로 렌더**함을 확정하고, 본 환경 측정을
동일하게 대체했다. 게이트 전부 통과, 359 recount 완전 중립.

## 2. 발견 (프로브 3라운드)

1. rhwp 측정 = HCR Batang(HANBatang.ttf) hmtx 와 완전 일치 — 종전 폭 오차는
   본 환경 메트릭 손상이 아니라 **한글의 폰트 대체 동작** 미모사.
2. 전 판별 클래스에서 Haansoft 적합·HCR 배제 (k=1600 정밀 사다리 포함):
   괄호 0.320→**0.500em(+56%)**, 쉼표·마침표 0.320→0.291, 숫자 0.550→0.583,
   A 0.706→0.750, a 0.569→0.500, · 0.320→0.333. 음절은 HCR(0.970) 유지,
   공백은 useFontSpace=0 고정 0.5em(기존 정합).
3. 괄호 편차가 21761835 r74(우리 3줄 vs 한글 4줄), 쉼표·소문자 편차가
   r75(9줄 vs 8줄) — 양방향 래핑 편차를 정확히 설명.

## 3. 구현·검증

- `text_measurement.rs`: `HAANSOFT_BATANG_ASCII`(95) + `haansoft_latin_override`
  — 검증된 `함초롬바탕`/`HCR Batang` 별칭에만 `measure_char_width_embedded` 단일 관문 삽입.
- 유닛 테스트 `issue_2156_hcr_batang_latin_uses_haansoft_metrics` + 픽스처
  `samples/task2156/width_ladder.hwpx` + 도구 3종 + `rhwp measure-width`.

| 게이트 | 결과 |
|--------|------|
| issue_1891 (76076=82/80168=157/80250=17/86712=65) | 통과 |
| issue_1842 / 1623 / 2146 / byeolpyo4=26 | 통과 |
| 359 recount vs 기준선(recount2146d) | **쪽수 변화 0** (완전 중립) |
| svg_snapshot | 함초롬 라틴 x좌표 이동 2건 골든 갱신(issue-157/form-002), CRLF 4건 기지 노이즈 |
| 전체 cargo test | 통과 |

2026-07-10 통합 재검토에서 메인터너가 적용 범위를 `함초롬바탕`/`HCR Batang` 정확한 별칭으로
한정했고, Dotum/확장 별칭 negative 및 HCR Batang alias positive 회귀를 추가했다. 깨끗한
`target` 전체 검증(release build/lib, release-test integration, Clippy, doctest, Studio,
WASM)도 통과했다.

**구성 판정**: #2150 em 공식과의 쌍 구성은 issue_1891 80168 이 157→153 으로
재실패 — 80168 상쇄는 ASCII 폭 축이 아닌 별도 축. 폭 단독 채택, em 공식은
#2150 보류 유지.

## 4. 후속

1. 함초롬돋움 등 대체 규칙 범위(Haansoft Dotum 가능성) 확인.
2. 80168 계열 em 상쇄 잔여 축 분해 — #2150 공식 해금 조건.
