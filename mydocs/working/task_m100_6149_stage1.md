# Task M100 #6149 — Stage 1 재검증 결과

- **이슈**: [#6149](https://github.com/edwardkim/rhwp/issues/6149)
- **단계**: 저배율 표시 순수 계약
- **기준 commit**: `upstream/devel` `9be8b0562`
- **WIP 실측일**: 2026-08-27 KST
- **절차 상태**: 정정 계획 승인 후 재검증 통과, Stage 1 결과 승인 완료

> 최초 기록은 Stage 1 수행 전에 승인받은 계획에 따른 완료 보고서가 아니었다. 기존 WIP 이력을
> 보존한 상태에서 정정 수행·구현 계획 승인 뒤 순수 계약을 다시 검증했으며, 아래 결과의 작업지시자
> 승인 전에는 Stage 2로 넘어가지 않는다.

## WIP 구현 내용

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

## 최초 WIP 검증 기록

```text
$ node --test rhwp-studio/tests/ruler-scale.test.ts rhwp-studio/tests/page-gap.test.ts
tests 6, pass 6, fail 0
```

## 정정 계획 승인 후 재검증

- **검증 기준**: `2f0f1106a` (`docs: #6149 Hyper-Waterfall 절차 복구`)
- **실행일**: 2026-08-27 KST
- **소스 변경**: 없음

```text
$ node --test rhwp-studio/tests/ruler-scale.test.ts rhwp-studio/tests/page-gap.test.ts
tests 6, pass 6, fail 0, skipped 0
duration_ms 88.045791
```

재검증 결과는 최초 WIP 실측과 일치한다. 대표 배율의 숫자·세부 눈금 최소 화면 간격, 배율 증가에
따른 눈금 단계 단조성, 잘못된 배율 정규화, 저배율 6px 페이지 간격 하한과 100% 기존 간격 보존이
모두 통과했다.

## 작업지시자 승인

Stage 1 재검증 결과와 다음 게이트를 제시한 뒤 작업지시자가 다음과 같이 승인했다.

> 진행해줘.

이 승인으로 Stage 1 결과를 채택하고 해당 기록을 로컬 커밋으로 고정한 뒤 Stage 2 재검증에 진입한다.

## 다음 단계

Stage 2에서는 가로·세로 눈금자가 같은 순수 표시 계약을 소비하고 마지막 편집 focus 페이지의 용지
범위만 표시하는 WIP를 재검증한다. Stage 2 결과 보고 뒤 다시 작업지시자 승인을 기다린다.
