# Task M100 #6149 — 1단계 완료 보고서

- **이슈**: [#6149](https://github.com/edwardkim/rhwp/issues/6149)
- **단계**: 저배율 표시 순수 계약
- **기준 commit**: `upstream/devel` `9be8b0562`
- **완료일**: 2026-08-27 KST

## 완료 내용

- 1mm의 화면 폭과 최소 픽셀 간격으로 숫자·세부 눈금 단위를 고르는
  `resolveRulerScale()`을 추가했다.
- 눈금 단계는 `1·2·5 × 10ⁿ mm`만 사용하며 숫자는 최소 30px, 세부 눈금은 최소 3.5px를
  확보한다.
- 페이지 간격은 100%에서 기존 10px를 유지하고, 저배율에서는 최소 6 CSS px를 보장하는
  `resolvePageGap()`으로 분리했다.
- 10/20/25/50/100/500% 대표 배율과 잘못된 입력을 DOM 없는 단위 테스트로 고정했다.

## 변경 파일

- `rhwp-studio/src/view/ruler-scale.ts`
- `rhwp-studio/src/view/page-gap.ts`
- `rhwp-studio/tests/ruler-scale.test.ts`
- `rhwp-studio/tests/page-gap.test.ts`

## 검증

```text
$ node --test rhwp-studio/tests/ruler-scale.test.ts rhwp-studio/tests/page-gap.test.ts
tests 6, pass 6, fail 0
```

## 다음 단계

2단계에서 가로·세로 눈금자가 같은 순수 표시 계약을 소비하고, 마지막 편집 focus 페이지의
용지 범위만 표시하도록 연결한다.
