# PR #9453 검토 — 스타일 목록 유지와 재클릭 닫기

## 최종 판정

**승인.** 코드 head `9798954074a1130453fd63514fed1f48506c854f`의 로컬 프런트엔드 검사와 필수 GitHub Actions를 확인했다. 문서 기록 커밋의 필수 검사와 최신 `devel`에 대한 병합 가능 상태를 확인한 뒤 병합한다. [이슈 #9452](https://github.com/edwardkim/rhwp/issues/9452) · [PR #9453](https://github.com/edwardkim/rhwp/pull/9453)

## 재현과 수정 범위

기준 `3e97980785dcee996646ed399f46e65fbaaee261`에서 Mac Chrome으로 [제보 문서](../../../samples/hwp3-sample16-hwp5-2024.hwp)를 열었다. 스타일 51개를 불러왔고, 첫 마우스 누름과 뗌 직후 `select:open`은 참이었다. 추가 입력 없이 600ms가 지나자 목록이 닫혔으며 포커스와 선택 값 46은 유지됐다. 검증 입력의 Git blob `1e7b1834277effd9240a2aa24724799c55b210f1`은 로컬에서 실행한 원본과 일치했다.

스타일 콤보는 기존 스타일 값과 `change`에 연결된 적용 경로를 사용한다. Studio가 목록의 열림 상태와 키보드 탐색을 관리하도록 바꿨다. 같은 ▼를 다시 눌렀을 때, Escape를 눌렀을 때, 목록 밖을 클릭했을 때는 스타일 변경 없이 닫는다. 항목을 고르면 기존 스타일 적용 명령을 한 번 실행한다. 문서의 조판·출력과 글꼴 공급을 바꾸는 경로가 없으므로 조판 원칙 검토의 렌더링 항목은 비해당이다.

## 검증

- 브라우저 회귀는 기존 코드에서 열림 유지 단계에 실패했고 수정본에서 통과했다. 실제 클릭 후 600ms 대기, 휠 스크롤 후 열림 유지, 같은 ▼·Escape·외부 클릭으로 닫기, 키보드 탐색, 스타일 적용과 실행 취소 기록 한 건을 검사했다. [브라우저 회귀](../../../rhwp-studio/e2e/style-combo-issue9452.test.mjs)
- Studio 생산 빌드와 프런트엔드 테스트 2,838건이 통과했다. 2건을 건너뛰었다. E2E 매니페스트 205개 항목과 `git diff --check`를 확인했다. 로컬에서는 `3e97980785`의 기존 WASM을 사용했다.
- 코드 head에서 선택적 CI의 Frontend package gates, Canvas visual diff, CodeQL, Adapter inter-diff, Proptest roundtrip, CI Impact Policy가 통과했다. [코드 CI](https://github.com/edwardkim/rhwp/actions/runs/45989139359) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/45989139440) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/45989139246) · [Adapter](https://github.com/edwardkim/rhwp/actions/runs/45989139236) · [Proptest](https://github.com/edwardkim/rhwp/actions/runs/45989139341) · [CI 정책 판정](https://github.com/edwardkim/rhwp/actions/runs/45994071660)
- `devel`이 `2eba4e4f8e34638b586e2720b395a365dfb0906d`까지 전진했을 때 Studio 소스 차이가 없음을 확인했다. 병합 시뮬레이션은 충돌 없이 통과했다.
