---
kind: investigation
status: completed
canonical: mydocs/plans/task_m100_3514.md
last_verified: 2026-08-20
---

# Task M100 #3514 Stage 1 — 오류 조사·범위·계획 복구

Issue: [#3514](https://github.com/edwardkim/rhwp/issues/3514)

## 조사 결론

기존 인쇄 시나리오만으로는 실제 확장 package 회귀를 충분히 막지 못한다. PR #168의 options CSP,
#1444의 viewer CSP·다크 자산 404, #3433·#3446의 `print.html` 누락, #432·#1520·#1521의
content-script 회귀를 함께 조사해 다음 최소 surface를 #3514에 반영했다.

- 실제 `dist` 설치와 동적 extension ID
- MV3 service worker target·정책 응답·오류 진단
- 실제 HWP3 fixture의 viewer canvas와 최종 파일명 상태
- 다크 자산, options hydration, same-origin print surface
- loopback content script와 HWP 배지 1개
- 외부망 차단, 탭 예산, 임시 profile·download 정리

상세 설정·다운로드 수명주기는 #3513, CI 선택 실행은 #3515로 유지한다.

## 계획 결과

수행계획과 구현계획을 Stage 1 조사·계획, Stage 2 구현, Stage 3 최종 검증·문서의 세 승인
게이트로 재구성했다. Stage마다 working 보고서를 제시하고 승인 뒤 커밋해야 다음 단계로 이동한다.

## 절차 이탈과 현재 경계

초기에는 구현과 검증까지 단일 Stage 1로 묶고 9개 경로를 한 번에 stage했다. 작업지시자의 지적 뒤
전체 index를 비웠고, 이탈과 복구 결정은
`mydocs/feedback/task_m100_3514_hyper_waterfall_recovery.md`에 기록했다.

Stage 2 구현 후보와 Stage 3 문서 후보는 작업트리에 존재하지만 이 Stage 1의 일부가 아니며 stage하지
않는다. 이미 실행한 검증 결과도 최종 Stage 3 증적으로 소급하지 않고 Stage 2 커밋 뒤 다시 실행한다.

## Stage 1 승인 대상

- `mydocs/orders/20260820.md`
- `mydocs/plans/task_m100_3514.md`
- `mydocs/plans/task_m100_3514_impl.md`
- `mydocs/feedback/task_m100_3514_hyper_waterfall_recovery.md`
- `mydocs/working/task_m100_3514_stage1.md`

이 다섯 문서의 승인 전에는 Stage 1 커밋과 Stage 2 staging을 진행하지 않는다.

## 승인 결과

작업지시자는 2026-08-20 22:09 KST에 “진행해줘”로 Stage 1 문서와 복구 경계를 승인했다.
Stage 1 문서 커밋 뒤 계획에 정의한 Stage 2 구현 경로만 stage한다.
