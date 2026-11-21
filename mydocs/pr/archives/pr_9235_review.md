# PR #9235 검토 — feat(edit): 한컴 조판 부호 지우기·찾아가기의 갈래별 자리와 지우기 API

## 통합 후보의 검토 결과

**메인테이너 보정 후 수용 가능.** 활성 필드 앞 감추기를 지운 뒤에도 글자가 같은 필드에 입력된다. 공통 회귀와 보호 문서의 판정은 [17건 통합 검토](pr_9292_review.md)에 기록했다.

## 접수 당시 판정

**머지 보류.** 조판 부호를 일괄 삭제하면 편집 중인 누름틀의 컨트롤 인덱스가 어긋난다.

## 접수와 검토 범위

- 작성자 및 브랜치 `semanticist21/pr/control-codes`
- 검토 head `d77ecdbe5b647a99e09895a2f52fd9440632b465`
- 리뷰어 `edwardkim`, 요청 시각 `2026-11-20T10:37:46Z` (UTC), GitHub 요청 목록 재확인
- 최신 devel `0a437a7eefb19bfe81fc12866887e4548ea9554d` 기준 개별 및 번호순 누적 병합 시뮬레이션 성공
- 원 head의 Build & Test 및 CI Impact Policy 성공. [CI 집계](https://github.com/edwardkim/rhwp/actions/runs/44706670389/job/136201236807)

조판 부호를 검색·삭제하고 필드의 글을 보존하는 명령을 검토했다. 제출 검사에는 본문·셀·머리말·각주 선택 삭제와 필드 저장 검사가 있다. 공개 배포 경로에서 두 WASM 메서드의 직접 호출과 타입을 확인해야 한다.

## 메인테이너 실행 결과

누적 적용 10번째, 적용 직후 `c6407bf83c`. 충돌 없이 적용됐다. 처음 10건을 합친 `c6407bf83c`에서 이 PR의 집중 검사 6개가 통과했다.

누름틀 내용이 `AB`인 문서에서 누름틀 끝을 활성화했다. 쪽 감추기 하나를 전용 명령으로 지운 뒤 C를 입력하면 누름틀 값은 `ABC`였다. 같은 부호를 `eraseControlCodes`로 지운 뒤 같은 자리에 입력하면 값은 `AB`로 남고 C가 필드 밖에 들어갔다. 앞선 컨트롤이 사라질 때 편집 중인 필드 참조를 갱신해 후속 입력까지 보존해야 한다.

공개 API 대조 대상은 `getControlCodes, eraseControlCodes`이다. 직접 빌드한 WASM 패키지에서는 메서드 존재와 TypeScript 컴파일을 확인했다. 추적 중인 `rhwp-studio/public/rhwp.d.ts`를 import한 같은 소비자 코드는 타입 검사에서 실패했다.

## 2026-11-21 메인테이너 보완 착수

원 [PR #9235](https://github.com/edwardkim/rhwp/pull/9235)의 head는 `d77ecdbe5b647a99e09895a2f52fd9440632b465`이다. GitHub에서 담당 리뷰어 `edwardkim`을 확인했다. 기준 devel은 `2480af27df`이다.

활성 누름틀 앞의 감추기 부호를 일괄 삭제한 뒤 C를 입력하면 누름틀 값은 AB에 머문다. 개별 삭제 대조군은 ABC다. 부호 일괄 삭제 뒤 활성 필드 주소를 갱신하고 이어지는 편집과 저장 결과를 검증한다.

## 메인테이너 보완 검증 중간 결과

활성 필드 앞 감추기 삭제 후 글자를 입력해 값 ABC를 확인했다. 공통 주소 갱신과 같은 입력의 정식 회귀가 통과했다. [쪽별 비교 원장](../assets/semanticist21-20261121/maintainer-comparison.tsv)

## 병합 후 기여자 PR 댓글 계획

조판 부호를 찾아 지운 뒤에도 편집 위치가 유지됩니다. 누름틀 앞의 감추기를 일괄 삭제한 뒤 C를 입력하면 누름틀 내용이 AB에서 ABC로 바뀌는 것을 확인했습니다. 통합 PR과 확정된 병합 커밋, 이 PR의 검토 기록을 함께 연결한다.

## 통합 결과

2026년 11월 22일 02시 15분(KST)에 [통합 PR #9292](https://github.com/edwardkim/rhwp/pull/9292)를 `devel`에 병합했다. 원 기여 커밋 `d77ecdbe5b647a99e09895a2f52fd9440632b465`이 병합 이력에 포함된 것을 확인했다. [원 PR #9235](https://github.com/edwardkim/rhwp/pull/9235) · [병합 커밋 `0adeca439f`](https://github.com/edwardkim/rhwp/commit/0adeca439f1481e7245a344b7713d7d563df2ace) · [전체 CI와 검토](pr_9292_review.md)
