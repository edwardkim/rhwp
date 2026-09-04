# Task M100 #2156 — Stage 3 완료보고서 (Haansoft 대체 구현 + 구성 판정)

**날짜**: 2026-07-10 / **브랜치**: `local/task2156`

## 1. 구현

`src/renderer/layout/text_measurement.rs`:
- `HAANSOFT_BATANG_ASCII` (95개, HBATANG.TTF upm=1024 추출) + U+00B7=0.333
- `haansoft_latin_override(primary_name, c)` — 검증된 `함초롬바탕`/`HCR Batang`의
  0x21..0x7F + U+00B7 오버라이드. **공백(0x20) 제외** (useFontSpace=0 고정
  0.5em 기존 경로 유지). 한글 음절 등은 기존 HCR hmtx.
- `measure_char_width_embedded` 최상단 삽입 — native/WASM 공통 내장 메트릭
  단일 관문. Stage 4에서 브라우저 경로도 검증 대상으로 재분류.
- 자가검증: `rhwp measure-width` — ( 6.667 / , 3.880 / 0 7.773 / A 10.0 ✓

## 2. 구성 판정 (2 라운드)

| 구성 | issue_1891 (4문서) | 1842/1623/2146/byeolpyo4 | 359 recount vs 기준선 | 판정 |
|------|--------------------|--------------------------|----------------------|------|
| **폭 단독** | **전부 통과** (82/157/17/65) | 통과 | **쪽수 변화 0** (완전 중립) | **채택** |
| 폭 + em 공식(#2150) | 80168 157→**153** ✗ | 통과 | — | 기각 |

**em 공식 상쇄는 폭 정합으로 풀리지 않음**: 80168 은 폭 단독에선 157 유지,
em 추가 시 -4쪽 — ASCII 폭 차로 상쇄되던 것이 아니라 별도 축(한글이 우리
em 모델보다 크게 렌더하는 NO_LS 셀 계열)이 존재. #2150 보류 유지, em 공식의
적용 조건 규명은 후속.

21761835: 폭 단독에서 쪽수 7·세그 오라클 불변 (이 문서의 래핑 편차 임계는
폭 수정으로 안 넘어감 — 개선은 셀 단위 임계 통과 문서에서 발생).

## 3. 산출물

- 소스: `text_measurement.rs` (테이블+오버라이드+유닛 테스트
  `issue_2156_hcr_batang_latin_uses_haansoft_metrics`)
- 픽스처: `samples/task2156/width_ladder.hwpx` + README
- 도구: `tools/make_width_ladder.py`, `tools/probe_width_ladder.py`,
  `tools/extract_haansoft_table.py`, `rhwp measure-width` (기 커밋)

## 4. 후속

1. WASM 측정 경로(JS canvas)에서 바탕 별칭 대체의 Studio 검증.
2. 함초롬돋움 등 타 한컴 폰트 대체 규칙 범위 확인 (Stage 4에서 검증되지 않은
   돋움/확장 계열의 Haansoft Batang 적용을 제거 — 돋움 계열은 Haansoft Dotum 가능성).
3. 80168 계열 em 상쇄 잔여 축 분해 (#2150 해금 조건).
