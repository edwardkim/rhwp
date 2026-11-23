# Studio 입력 예열 경계와 실제 WASM 검증

예열의 두 RAF 뒤에도 지연 페이지 배치가 남아 있어 첫 입력의 글꼴 준비 호출 수에 섞였다.
공개 `hasDeferredPaginationPending()`이 false가 된 뒤 화면 두 프레임을 기다리게 했다.
20회 입력의 준비 호출 20회·재등록 0회·쪽별 수집·IME·붙여넣기·undo/redo·캐럿·paint/layout
검사는 유지했다. 제품 구현은 바꾸지 않았다.

[요약과 파일 해시](summary.json)에 원 실패, 진단, 실제 정식 재실행의 경로를 기록했다.
Source139는 원 테스트에 이 보정만 적용한 실행이며 진단용 계측을 넣지 않았다.
base·후보 각각 none·empty·populated 조건이 통과했고 실제 WASM 응답 해시를 확인했다.
[14개 여정의 결합 판정](source141-studio-completed-summary.json)은 원 실패를 보존하고
해당 여정에만 Source139의 결과를 사용한다. 단위 테스트는 2,834개 통과했고 TypeScript도 통과했다.

이 자료는 #9318 병합 전 base `8a196ca7aea2aa3cf32d394f5b7da2bdf4520479`와 제품 후보
`8f9fd0414ffcfb720fa4a601c79c4590480de0e8`의 근거다. #9318 병합 후 검증으로 재사용하지 않는다.
사전 병합 시뮬레이션은 충돌 0건이며 실제 병합 후 리베이스·기능 보존 검증을 별도로 진행한다.
