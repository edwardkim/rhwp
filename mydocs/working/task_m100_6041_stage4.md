# Task M100 #6041 Stage 4 — Draft 범위 동결과 후속 이슈 인계

- 날짜: 2026-08-31
- Issue: #6041
- 현재 범위: #6467 구현·증적 동결, #6521 생성, #6041·#6042 계획 코멘트까지 완료
- 코드 변경, 새 구현 브랜치·PR 생성, Ready 전환, merge, 이슈 close: 수행하지 않음

## #6467 정리

- 제목을 `perf(studio): surface 예산 초과 시 비포커스 쪽 렌더 해상도를 조정한다`로 정리했다.
- 고배율 전용 또는 비가시 전용으로 오해하지 않도록 예산 초과·offscreen 우선·비포커스 visible 조정
  가능성을 본문에 명시했다.
- 37.49%는 200% 실문서의 offscreen prefetch pixel 절감이며 저배율 LOD/속도 개선 증거가 아님을 유지했다.
- UI 자동화와 약 500ms 안정화 확인을 포함한 1352/1360ms는 직접 UX latency가 아님을 보충했다.
- `Closes #6041`을 `Refs #6041`로 바꾸고 Stack 2/4 및 Draft 유지·추후 bottom-up Ready 방침을 기록했다.
- 기존 수정 전후 PNG 9개와 immutable evidence 링크를 보존했다.

## 신규 이슈와 코멘트

- [#6521 저배율 overview 화질·절감량 기반 DPR ceiling](https://github.com/edwardkim/rhwp/issues/6521)
  - zoom eligibility, 실제 절감량 gate, DPR 1.5 우선, 히스테리시스, 네 단계 zoom 계측,
    DPR 변경 페이지만 추가 reraster를 포함했다.
  - 실문서 측정으로 threshold를 정하고, 이득이 작은 문서의 raw DPR 유지와 자동 정책 보류 가능성을
    수용 기준에 명시했다. `postmelee`, 기존 네 label, milestone `v1.0.0`으로 등록했다.
  - 등록 전 열린 이슈에서 `저배율`, `overview LOD`를 검색했다. #6187은 눈금자/resize 작업으로
    목적이 다르고 동일 LOD 후속 이슈는 없었다.
- [#6041 계획 코멘트](https://github.com/edwardkim/rhwp/issues/6041#issuecomment-5474955283)
  - #6467 budget-first 계약과 #6521 저배율 목적을 분리하고, #6041을 두 작업의 상위 추적으로 유지한다.
- [#6042 인계 코멘트](https://github.com/edwardkim/rhwp/issues/6042#issuecomment-5474955846)
  - #6521의 최종 tier·계측을 LRU/scheduler가 소비하는 경계를 기록했다.

계획된 native stack: **#6458 → #6467 → #6521 PR → #6042 PR**. 후속 PR 번호는 아직 없으며 예측하지 않았다.

## 검증 범위

- 시작 head `5fc2542005ca271c9ac3452ce11416e7a0855ba7`에서 code candidate
  `e37d483fd5f16b2c710a95389f882c9985d50851` 이후 source/test/Cargo 변경이 없음을 확인했다.
- `node --test tests/render-surface-budget.test.ts`: 13 pass, 0 fail 재확인.
- `npx tsc --noEmit`: exit 0 재확인.
- `git diff --check` 및 변경 Markdown 5개 상대 링크 검사: 통과.
- 게시 후 API로 #6467의 Draft/Stack 2/4/Refs #6041/#6521 링크/비교 PNG 9개를 확인했고,
  #6521 본문·assignee·labels·milestone 및 두 계획 코멘트의 한글·실제 줄바꿈·상호 참조를 재확인했다.
- 이번 tail은 mydocs-only다. 전체 npm/Cargo/browser 시각 캡처는 재실행하지 않고 이전 실행 증적을 보존했다.
- 원격 확인 시점의 Draft / MERGEABLE / CLEAN과 빈 statusCheckRollup은 CI 통과를 뜻하지 않는다.
  최신 head CI·최종 시각 검증·Ready/merge 승인은 전체 stack의 후속 게이트로 남는다.

이번 문서는 #6041 전체의 최종 보고서가 아니다. 다음 작업은 #6521의 Hyper-Waterfall 수행·구현 계획
작성과 승인이고, 현재 지시 범위에 포함된 구현은 없다.
