---
kind: implementation
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-08
---

# PR #6863 메인터너 보정

## 1단계: 원인과 회귀 고정

- 원 contributor head: `1e82075a8ba4d79ac12fb11255a8563f87ce2281`.
- 로컬 적용 head: `2e17c6a80f7050d92f9b4a28476f3e6dec6e5c5b`.
- 사용자 승인: 머지 보류 사유를 메인터너 보정으로 처리. 원격 push/PR/merge는 별도다.
- 원인: 글상자 내장 표 셀의 Shape 경로가 모든 인라인 도형에 첫 줄과 같은 영역을 전달한다.
  일반 표 셀은 문단이 등록한 개별 좌표를 소비하지만 이 경로에는 해당 처리가 없다.
- 공개 합성 입력의 같은 줄/서로 다른 저장 줄에서 두 도형의 글자가 모두 `(0, 0)`에 겹쳤다.
- 기존 `tests/cases/issue_6735_textbox_table_cell_shape.rs`에 위치 분리 회귀를 추가한다.
  기존 inline/floating 내용 보존과 3단계 cell path 계약도 유지한다.

## 보정 범위와 검증 계획

1. 문단 조판이 등록한 인라인 Shape 좌표를 우선 사용한다.
2. 빈 셀 문단의 미등록 도형은 기존 줄 소유권 해석기로 대상 줄을 찾고, 같은 줄 TAC 폭을
   합산해 문단 정렬과 앞 개체의 가로 진행을 반영한다. 부동 도형 경로는 변경하지 않는다.
3. 새 위치 회귀와 원 PR focused test, 독립 SVG/PNG 재현을 검증한다.
4. 기존 유사 문서/PDF를 재사용해 보정 전후를 비교한다. 임시 log/SVG/JSON은 커밋하지 않는다.
5. 집중 검증 결과 이후 PR 직전 전체 검증은 승인 단계에서 수행한다. 기존 원 head CI를
   새 보정 code head의 전체 회귀나 Clippy 성공으로 기록하지 않는다.

## 결과

수정 전 focused nextest: 4건 중 기존 2건 통과, 새 위치 회귀 2건 실패(exit 100).
두 label 좌표는 두 경우 모두 `(0, 0)`이다. 첫 실행은 자동 suite 재배정 전 번호를 사용해
0건(exit 4)이었으며, 새 manifest의 suite 009로 재실행한 위 결과만 회귀 근거로 삼는다.
