# Task #885 Stage 1 — 완료 보고서

작성일: 2026-05-15
브랜치: `local/task885` (from `stream/devel`)

## 1. 수행 내용

1. `samples/**/*.hwpx` 의 `Contents/header.xml` 에서 폰트 face 추출 (40종)
2. `samples/*.hwp` + `samples/basic/*.hwp` 를 `rhwp export-svg --font-style` 으로 변환 후 SVG `font-family` 첫번째 가족명 추출 (40종)
3. 합집합 64종을 현재 `resolve_metric_alias` / `FONT_METRICS` 와 대조
4. HY 계열 누락 별칭 식별 및 매핑 안 작성

## 2. 식별 결과

- **본 이슈 핵심 (HY 계열)**: 샘플에서 실제 사용 중인 7종 + 짝 1종 = **8개 별칭 추가 안 확정**
- **패턴 확장 후보**: HY바다·간기·산·나무·백송·해서 등 (Stage 2 추가 여부 결정 대기)
- **범위 외 후속 이슈 후보**: 함초롬 weight 분기, Pretendard weight, KoPub, 한컴 윤고딕/소망/쿨재즈, 양재, DX, HCI, 08서울남산체 등

상세 매핑 표와 사유는 `mydocs/tech/task885_missing_aliases.md` 참조.

## 3. 산출물

- `mydocs/tech/task885_missing_aliases.md` — 누락 폰트 식별 및 매핑 안
- `/tmp/task885/*.txt` — 조사 원본 자료 (휘발)

## 4. Stage 2 진입 조건

다음 결정이 필요합니다:
1. 본 이슈 핵심 8개 별칭 추가 안 승인
2. 패턴 확장 (HY바다/간기/산/나무/백송/해서) 본 타스크 포함 여부

승인 후 Stage 2 (`resolve_metric_alias` 확장 + 회귀 테스트) 진행하겠습니다.
