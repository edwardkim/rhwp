---
kind: working
status: completed
issue: 4245
last_verified: 2026-08-08
---

# Task ime_reanchor Stage 1 - 조합 replace 거부 시 재정박 복구

## 구현

- `input-handler-text.ts` 조합 분기(`onInput`)의 `replaceTextAtRaw(anchor, compositionLength,
  text)`를 try/catch 로 보호. wasm 의 deferred replace 범위 가드가 거부하면(외부 변이로
  앵커·길이가 낡은 경합) warn 후 조합을 현재 캐럿에 재정박(`compositionAnchor` 갱신,
  `compositionLength=0`)하고 이번 조합 텍스트를 새로 삽입해 입력 스트림을 잇는다. 재삽입도
  실패하면 이번 업데이트만 버린다.
- 동기: 종전에는 거부가 uncaught 로 `onInput` 전체를 죽여 조합 추적이 낡은 값으로 wedge —
  이후 모든 조합 업데이트 연쇄 실패. `onCompositionEnd`는 raw 뮤테이션이 없어(command 기록만)
  대상 아님을 확인.

## 검증 결과

- 소스 계약 테스트 2종(`composition-replace-reanchor.test.ts`): try 보호 존재, catch 의
  재정박(현재 캐럿 기준 + 길이 리셋) — pass.
- 실브라우저 강제 트립(:7701, 거대 셀 문서): 조합 중 `compositionLength=9999` 오염 →
  다음 업데이트에서 uncaught 없이 복구(재정박 앵커·compLen=1), 이어지는 업데이트·compositionend
  정상, 최종 텍스트 정합(+1), undo 로 원문 완전 복원.
- 발견 경위: 겹친 합성 테스트 배터리(도구 타임아웃 후 페이지에서 계속 실행)가 조합 스트림
  2개를 인터리브해 가드 트립을 유발 — 실사용 IME 는 조합을 겹칠 수 없어 사용자 도달 경로는
  아니나, 가드 거부가 uncaught 인 취약성 자체는 실재해 방어를 추가.
