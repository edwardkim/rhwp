# 호스트 글꼴 입력의 예열 완료와 측정 경계 보존

원 이슈 #9165의 해결 PR #9168(`1695089d21fb5ff116eeba263e4935f04b4ac494`)은
입력한 쪽의 글꼴 요청만 다시 수집하게 했다. 후속 PR #9174 통합
`3360f5fedf63cd918378fcf8d6fb95f7a13b92d4`는 문서별 등록을 재사용하고, 20회 입력의
준비 호출 20회·이미 등록한 프로그램 바이트 재등록 0회·후반 입력의 0쪽 수집·IME·붙여넣기·
undo/redo·캐럿·그림 보존을 함께 확인했다. 현재 테스트의 예열은 한 번 입력하고 두 RAF만
기다린 뒤 호출 수를 초기화한다.

실제 Source105 후보에서 원 테스트는 21/20으로 간헐적으로 실패했고 sparse stack 진단은
초반 입력 중 별도 `refreshPagesForMutation` 호출을 확인했다. 본문·쪽수·캐럿·paint/layout
내용은 보존됐다. Source138에서 공개 `hasDeferredPaginationPending()`을 확인하니 새 base와
후보의 none·empty·populated 여섯 조건 모두 예열 뒤 지연 배치가 아직 남아 있었다.
배치 종료까지 3–5 RAF를 더 기다리고 화면 두 프레임을 마친 뒤 측정한 여섯 조건은 원
assertion 그대로 정확히 20회였다. 이는 renderer의 수치나 기대 호출 수를 바꾸는 문제가
아니라 예열의 비동기 작업이 본 측정에 들어온 테스트 setup의 경계다.

예열 입력 → 공개 지연 배치 상태가 false → 완료 변경의 화면 두 프레임 → 등록 수 초기화와
20회 본 입력의 순서로 연결한다. 배치 완료 대기는 15초 상한을 두고 실패를 숨기지 않는다.
본 입력 횟수·준비 호출 20회·재등록 0회·페이지 수집·내용·커서·IME·undo/redo assertion은
유지한다. 제품 코드와 frozen Native/WASM은 바꾸지 않는다.

수정 전 실패와 stack 진단은 보존하고, 수정한 정식 e2e를 같은 새 base·후보 패키지에서
다시 실행한다. 실제 브라우저가 받은 WASM 응답 해시도 확인한다. 전체 Studio unit과
TypeScript도 실행한 뒤 원 14개 여정의 나머지 완료 결과와 합쳐 판정한다.

## 실제 검증 결과와 다음 단계

Source139의 수정한 정식 테스트는 base·후보의 세 catalog 조건 모두 정확히 준비 호출 20회로
통과했다. 실제 브라우저가 받은 WASM 응답은 각 frozen 패키지의 SHA-256과 일치했다.
이 결과를 원 Source117의 다른 여정과 합친 Source141은 14개 실제 여정 모두 통과했다.
최종 `npm --prefix rhwp-studio test`는 2,834개 통과·실패 0개·skip 0개였고,
`cd rhwp-studio && ./node_modules/.bin/tsc --noEmit`도 exit 0이었다.
수정 전 두 실패·sparse stack·예열 지연 상태·정식 재실행·공급 해시·단위 테스트 로그는
[Source50](../pr/assets/issues7754_7765/source50-studio-warmup-boundary/summary.json)에 보존했다.
Source106 Native lib는 최종 exit 0을 확인했으며 후속 Native 검사를 실행 중이다.

사용자의 최신 지시에 따라 PR #9318의 CI와 병합 여부를 모니터링한다. 2026-11-22 확인한
head는 `93482b2d0069cfa7648c5f08df95ddc2eee9c179`이며 아직 병합되지 않았다.
현재 후보와 이 head의 사전 병합 시뮬레이션은 충돌 없이 종료했다. 이는 실제 리베이스의
검증을 대신하지 않는다. 실제 병합을 확인한 뒤 최신 `upstream/devel`로 리베이스하고,
현재 작업과 새로 반영된 기능을 같은 새 base·후보 조건에서 다시 검증한다.
이 문서의 결과는 병합 전 `8a196ca7aea2aa3cf32d394f5b7da2bdf4520479` base와
`8f9fd0414ffcfb720fa4a601c79c4590480de0e8` 제품 후보의 근거로만 사용한다.
PR 준비까지만 진행하고 대기하는 범위를 유지한다.
