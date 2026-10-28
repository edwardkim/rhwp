# Issue #1320 — 편집 액션 라우터와 Undo/Redo 트랜잭션 아키텍처 정비

## 배경

#1319에서 문단 서식 변경 Undo/Redo를 기존 `CommandHistory`에 편입하는 과정에서, rhwp-studio의 편집
작업 라우팅이 아직 완전히 통합되어 있지 않음을 확인했다.

현재 구조는 `CommandHistory + EditCommand`라는 기본 골격은 갖추고 있지만, 일부 UI 경로는 여전히 WASM
mutation API를 직접 호출한다. 그 결과 다음 문제가 반복될 수 있다.

- 어떤 동작은 history에 기록되고, 어떤 동작은 기록되지 않는다.
- 같은 사용자 작업이 command, snapshot, recordWithoutExecute 중 서로 다른 방식으로 들어간다.
- undo payload를 JS에서 재구성하면서 UI 단위와 core raw 단위가 섞일 위험이 있다.
- 렌더 invalidation 범위가 command별 dirty scope가 아니라 full refresh 중심으로 흐를 수 있다.

## 목표

rhwp-studio와 브라우저 확장 앱의 WASM 아키텍처에 맞게 편집 액션 라우팅과 Undo/Redo 트랜잭션 모델을
체계화한다.

## 제안 방향

1. history stack은 하나로 유지한다.
2. 복원 payload는 도메인별 command/transaction이 책임진다.
3. UI handler는 WASM mutation을 직접 호출하지 않고 edit action/router를 통과한다.
4. WASM core는 가능한 경우 before/after delta와 dirty scope를 반환한다.
5. renderer invalidation은 command 결과의 dirty scope를 기준으로 최적화한다.
6. snapshot은 붙여넣기, 복합 삭제, 구조 변경처럼 delta 정의가 부담되는 작업에 제한적으로 사용한다.

## 1차 조사 대상

- `CommandHistory`
- `executeOperation()`
- `recordWithoutExecute()`
- `SnapshotCommand`
- 텍스트 입력/삭제 command
- 글자/문단 서식 command
- 표 구조 변경 command
- 이미지/도형 이동 및 크기 조절 command
- paste/cut/delete snapshot 경로
- 직접 `wasm.*` mutation 호출 경로

## 성공 기준

- 주요 편집 작업이 공통 라우터를 통해 history에 기록된다.
- command별 undo/redo 복원 계약이 문서화된다.
- dirty scope가 명확한 작업은 full refresh를 피할 수 있다.
- 기존 텍스트, 문단, 표, 이미지 Undo/Redo UX가 회귀하지 않는다.

## 관련

- GitHub Issue: #1320
- 선행 작업: #1319
