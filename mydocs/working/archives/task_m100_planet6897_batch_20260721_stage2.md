# planet6897 열린 PR 통합 검토 - Stage 2

## 목적

Stage 1에서 통합한 planet6897 PR의 구현을 메인터너 관점에서 보정하고, 렌더와
기준 자료의 검증 범위를 확정한다.

## 작업 범위

1. #2671의 SVG 임베디드 글꼴 테스트가 SVG 전체가 아닌 요청한 글꼴 face의
   `@font-face` 규칙을 검사하도록 보강한다.
2. #2666의 r18 survey를 제출 당시 결과와 현재 `#2702` 통합 상태가 혼동되지 않도록
   역사적 기록으로 명확히 한다.
3. #2663 HWPX와 Hancom 2022 PDF를 visual sweep으로 비교하고, #2665/#2669/#2671의
   렌더 결과를 focused 테스트와 실제 산출물로 다시 확인한다.

## 완료 기준

- 대상 SVG face가 `data:font` URI를 사용하고 `local(...)` fallback을 쓰지 않음을
  회귀 테스트가 직접 검증한다.
- r18 문서의 기준 commit, 측정 시점, 현재 통합 상태가 분명하다.
- 시각 검증 결과와 focused 검증의 한계가 최종 PR review 문서에 기록될 수 있다.

## 결과

### #2671 SVG 임베디드 글꼴 회귀 가드

- `tests/issue_2524_embedded_font_svg.rs`가 SVG 전체 문자열이 아닌
  `RHWP Bitmap SVG Glyph Smoke`의 정확한 `@font-face` 규칙을 추출하도록 보강했다.
- Subset, Style, Full 모드 모두에서 그 규칙이 `data:font` URI를 포함하고
  `local(...)` fallback을 포함하지 않는지 확인한다.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --test
  issue_2524_embedded_font_svg`는 3/3 통과했다.
- 샘플 HWPX의 `BinData/font-native-smoke.ttf`는 TrueType magic(`0001 0000`)을
  사용하므로 현재 `font/ttf` MIME 판정과 일치한다.

### #2666 r18 survey 기록 보정

- r18 수치는 2026-07-20의 `HEAD=abae64173` 제출본에서 측정한 역사적 기록임을
  문서 첫머리에 명시했다. 현재 `upstream/devel` 재실행 결과로 해석하지 않는다.
- 측정상 이탈 2건은 후속 기여자 검토에 따라 실제 각주 밴드 knife-edge 1건과
  한글 캐럿 측정 아티팩트 1건으로 구분했다.

### 시각 검증

- #2663 `36382471_masked.hwpx`: RHWP/Hancom PDF 모두 2쪽, sweep flagged 0/2.
- #2663 `36341511_masked.hwpx`: RHWP 9쪽, Hancom PDF 8쪽이며 6쪽에서
  line-order/column drift가 감지됐다. 이는 PR이 명시한 #2279 잔여이고 이번 PR은
  해당 입력과 기준 PDF, page-pin 회귀 가드를 추가하는 범위다.
- #2669 `hwpx-02.hwpx`: 1쪽에서는 과밀 글리프가 재현되지 않았으나 전체 쪽수는
  RHWP 7쪽, Hancom PDF 5쪽이다. PR 본문이 이미 밝힌 비목표 잔여로 최종 리뷰에
  분리 기록한다.
- #2665 원형대원형/3차원원형 기준 PDF 1쪽은 sweep flagged 0으로 확인했으며,
  축·글꼴의 기존 fidelity 차이는 이번 OOXML 차트 파싱 보정의 범위를 넘는다.
