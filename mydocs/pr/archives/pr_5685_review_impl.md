# PR #5685 통합 구현 기록

## 적용 순서

1. 최신 `upstream/devel`에서 `integration/lpaiu-cs-20260820`을 만들었다.
2. #5675의 세 commit을 오래된 순서로 `-x` 체리픽했다.
3. #5676, #5684의 각 고유 commit을 `-x` 체리픽했다.
4. Studio 기본 검증과 code candidate Full CI를 통과한 뒤 이 review 기록을 trailing commit으로 추가한다.

## 충돌과 보정 경계

- 다섯 commit은 충돌 없이 누적됐다.
- 메인터너 코드 보정은 추가하지 않았다. #5675의 그림·OLE snapshot 유지, #5676의 cell 좌표 보수적 거절,
  #5684의 generated type 대조는 원 변경의 실제 범위 안에서 검토했다.
- contributor fork branch에는 review 기록이나 보정 commit을 push하지 않는다.

## 후속 단계

- trailing 문서 head의 fast-pass CI가 성공하면 작업지시자 승인 뒤 #5685을 병합한다.
- 병합 뒤 원 PR·관련 issue의 상태와 `edwardkim/rhwp` 통합 branch 정리는 `post_merge.md` 순서로 처리한다.
