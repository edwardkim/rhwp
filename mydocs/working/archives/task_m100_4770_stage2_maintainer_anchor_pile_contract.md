---
kind: working
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-14
---

# #4770 Stage 2 - 메인터너 앵커 그림 스택 계약 보정

## 배경

PR #4774의 첫 보정은 HPV 코호트의 본문 Square 그림 24장이 한글처럼 하나의 앵커 쪽에
겹쳐 남도록, 저장 첫 줄의 wrap-zone과 `vertical_pos`를 사용했다. 검토에서 두 경로의
판정 범위가 달라 일반 전면 그림의 #1995 낱장 배치까지 억제할 수 있고, 페이지 절대 좌표인
`vertical_pos`를 본문 높이와 직접 비교한다는 결함을 확인했다.

## 보정

- `rendering.rs`에 비-TAC Square, 겹침불허, 빈 문단, 동일 세로 band, 전면급 그림 다수를
  하나의 공통 저장 스택 계약으로 추출했다.
- `typeset.rs`의 #1995 억제는 자체적인 폭·높이 추정을 제거하고 공통 계약만 사용한다.
  `TopAndBottom`, 본문 텍스트, 서로 다른 anchor 또는 겹침 허용 그림은 기존 낱장 배치를 유지한다.
- `LineSeg.vertical_pos`는 페이지 상단 기준 절대 좌표이므로 두 경로 모두 `PageAreas.body_area.bottom`
  또는 layout의 본문 하단 절대 좌표와 비교한다.

## 회귀 계약

- 본문 하단에 정확히 닿는 Square 스택은 앵커 쪽에 남는다.
- 같은 wrap-zone과 `vertical_pos`라도 `TopAndBottom` 그림은 #1995 낱장 배치를 억제하지 않는다.
- HPV, #1995, #2004 셀 스택의 기존 실문서 검증은 후속 maintainer 검증 단계에서 다시 확인한다.

## 검증 상태

이 stage에서는 코드와 회귀 단정을 추가했다. 로컬 테스트와 원격 CI는 아직 실행하지 않았다.
