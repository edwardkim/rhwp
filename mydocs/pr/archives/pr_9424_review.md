# PR #9424 검토

## 판정

메인테이너 보정 후 수용 가능. cozykbin 님의 첫 기여로 #9139의 본문 첫 문단과 표 셀에서 자동 줄바꿈 아래의 선택 표시가 복원됐다. 원 PR에서 새로 생긴 본문 수식 선택 오류는 메인테이너가 보완했다. [원 PR](https://github.com/edwardkim/rhwp/pull/9424) · [원 이슈](https://github.com/edwardkim/rhwp/issues/9139) · [보완 전 재현](pr_9424_review_impl.md)

| 검토 버전 | 커밋 |
| --- | --- |
| 기여자의 원 커밋 | `1e9194008e7b599b4fb9777a88826ec2ba79f392` |
| 검증 기준 devel | `3fdaa042588cce42006d17e958a7f34d7c205b2e` |
| devel과 원 PR의 병합 | `c765fa2ac96ce5d612f4620bcea89756c2f3f199` |
| 원 기여 위의 메인테이너 코드·테스트 보정 | `a37c9cce4696598735b6d259e1ab3f21495fa90b` |
| 최신 devel과 보정을 함께 검증한 코드 | `45c5648d43c9e152c99874f2dff470c37f652044` |
| 원 브랜치에 fast-forward 가능한 검증 자료 포함 후보 | `9d8fb2e2bc9cde8401d9397b3b36269b2ce92413` |
| 최신 devel·원 기여·보정·검증 자료를 모두 담은 upstream 수용 후보 | `186bd1d7d4b314a41d38b263a9a36ba4d36c162e` |

## 구현과 재현

원본 HWPX에는 첫 문단의 저장 줄 정보가 생략돼 있다. 선택 범위와 복사 문자열은 이미 둘째 줄에 도달했지만 선택 사각형은 합성 줄 하나에 묶였다. 기여자는 이 경우 해당 문단 또는 표 셀의 실제 렌더 트리에서 줄 시작을 읽도록 고쳤다. 해당 표본의 본문·셀 드래그 16건 중 `devel`은 4건, 원 PR과 보정본은 각각 16건을 통과했다. 세 버전 모두 16건의 복사 문자열이 원문과 일치했다. [드래그 측정](../assets/pr9424/drag.tsv) · [실제 Studio 화면](../assets/pr9424/verified-wrapped.png)

본문 인라인 수식은 글줄에서 논리 한 칸을 차지한다. 원 PR의 캐럿 탐색은 수식의 원본 컨트롤 위치를 사용했지만, 줄 시작 수집은 뒤의 텍스트에서 시작했다. 보정으로 두 경로가 같은 본문 수식·인라인 양식 주소를 사용하게 했다. 줄 시작을 모을 때 컨트롤 위치는 문단마다 한 번 계산하며, 셀의 평면·중첩 경로는 해당 셀의 텍스트 소유 정보로 판별한다. Studio의 `WasmBridge.getSelectionRects`와 `InputHandler`가 이 사각형을 받아 편집 화면에 선택 표시를 그린다. [수식 화면 비교](../assets/pr9424/verified-equation.png) · [세 버전의 근거](../assets/pr9424/README.md)

## 로컬 검증

- Linux 최소 글꼴 환경에서 원 PR에 새 검사 두 건을 적용했다. 본문·셀 자동 줄바꿈 검사는 통과했고 본문 수식 하나의 선택 표시는 기대한 1개에 대해 0개여서 실패했다. 보정 뒤 같은 검사와 기존 중첩 셀·캐시 대조를 포함한 집중 검사 8건이 통과했다.
- 일반 Rust 전체 `release-test`에서 55개 실행 파일의 14,196건이 통과했다. 기본 프로필에서 45건은 건너뛰었다. 저장소 공개 글꼴 5개만 노출하고 시스템·Windows 글꼴 경로를 격리했다. Rustfmt, 기본·WASM32·workspace 전체 대상 Clippy, workspace 빌드와 suite 배정 검사를 통과했다.
- 고정 Cargo.lock과 wasm-opt를 사용해 세 소스의 WASM을 별도로 만들었다. Chrome 156에서 실제 사용한 바이너리 해시를 확인했다. 수식만 선택한 폭은 원 PR의 0px에서 35.1px로, 뒤 한 글자는 44.4px에서 9.2px로 복원됐다. 수식 위치와 저장 형식을 달리한 44개 한 칸 선택, 기존 양식의 26개 선택을 대조했다. [WASM 출처](../assets/pr9424/wasm-sha256.tsv) · [위치별 결과](../assets/pr9424/equation-placement.tsv) · [양식 대조](../assets/pr9424/form-controls.tsv)
- 같은 Rust WASM 산출물로 Cargo.lock에 맞는 wasm-bindgen 0.2.127의 Node 바인딩도 만들어 Studio 테스트 2,837건을 모두 통과했다. 혼합 글자 모양 Undo/Redo와 캐럿 대기 서식의 실제 WASM 왕복 검사를 포함한다. TypeScript·Vite 배포 빌드와 교차 구역 편집 E2E의 확인 항목 20개도 통과했다.
- 최신 devel의 #9428에서 반영한 세 기능의 공개 문서 6개·8쪽은 세 버전에서 렌더 트리와 SVG가 일치했다. 원 드래그 표본 두 쪽의 선택 전 Canvas PNG 또한 동일했다. 인쇄 출력에 포함되지 않는 편집 표시의 위치·범위는 실제 마우스·키보드와 브라우저의 사각형으로 확인했다. [#9428 쪽별 비교](../assets/pr9424/devel-controls.tsv) · [페이지 Canvas](../assets/pr9424/page-canvas.tsv)
- 같은 Chrome에서 세 WASM을 번갈아 실행해 긴 선택 구간도 확인했다. 31·127·255개 문단에서 사각형은 세 버전 모두 같았다. 준비 호출 2회 뒤 11회씩 측정한 중앙값은 기준 devel 11.9·25.6·51.9ms, 원 PR 12.8·26.7·55.0ms, 보정본 11.5·26.3·53.5ms였다. 측정 대상은 단일 호출의 선택 사각형 계산이었다.

## 별도로 관찰한 기존 동작

표 셀 맨 앞의 수식만 선택하면 표시가 없고, 본문 수식 바로 뒤 글자를 일반 텍스트로 복사하면 다음 글자가 복사됐다. 같은 HWPX·키 입력으로 최신 devel, 원 PR, 보정본의 결과가 같았다. 본문 수식 뒤에서 공개 `copySelectionLogical`은 의도한 글자를 반환했고 Studio가 사용하는 `copySelection`은 다음 글자를 반환했다. 이번 수용 판정은 #9139와 본문 수식의 선택 표시를 대상으로 한다.

## GitHub 상태

11월 26일 18시 55분(KST) 기준으로 원 PR의 head와 fork 브랜치는 `1e9194008e`였고 GitHub에서 병합 가능 상태였다. 두 제출 후보 모두 최신 `devel` 및 리뷰어 없는 본인 Draft #9301(`83fe8497e2`)과 충돌 없이 병합된다. 최신 devel에 두 제출 경로를 각각 병합한 결과에서 제품 코드와 회귀 테스트가 로컬 검증 커밋 `45c5648d43`과 동일했다. 원 PR의 필수 Build & Test와 CI Impact Policy가 성공했다. 원 커밋의 선택적 CI에서 Frontend·WASM build·Native Skia는 건너뛰었다. CodeQL의 Rust 분석은 성공했고, GitHub 코드 스캐닝 요약은 Actions·JavaScript·Python 분석이 없어 `neutral`이었다. [코드 스캐닝 요약](https://github.com/edwardkim/rhwp/runs/139724296071) 이후 작업지시자의 승인으로 보정본을 원 기여 브랜치에 fast-forward했다. [원 CI](https://github.com/edwardkim/rhwp/actions/runs/45758261860) · [정책 검사](https://github.com/edwardkim/rhwp/actions/runs/45766624095)

## 보정 코드의 GitHub 검증

원 PR 브랜치에 제출한 `9d8fb2e2bc9cde8401d9397b3b36269b2ce92413`에서 회귀 Archive A–E, Lint, Build & Test, Rust CodeQL 분석, Adapter 비교와 Proptest가 통과했다. CI Impact Policy도 성공했다. 코드와 검토 자료가 포함된 이 커밋 위에 검토 문서와 오늘할일을 추가한다. [코드 CI](https://github.com/edwardkim/rhwp/actions/runs/45804413619) · [코드 정책 감사](https://github.com/edwardkim/rhwp/actions/runs/45809101356) · [Rust CodeQL](https://github.com/edwardkim/rhwp/actions/runs/45804413547)

## 병합과 연결 이슈 종료

2026년 11월 26일 20시 21분(KST), 원 PR에 추가한 최종 커밋 `033cb40928`을 edwardkim이 승인하고 `devel`에 merge commit으로 병합했다. 이 커밋은 검토 문서·오늘할일만 추가했다. GitHub preflight가 검증된 `9d8fb2e2bc`를 재사용했고 Build & Test와 CI Impact Policy가 통과했다. GitHub의 병합 트리는 로컬에서 확인한 `6a2d16ab91`과 같고 원 기여 `1e9194008e`와 메인테이너 보정 `a37c9cce46`를 포함한다. [승인 리뷰](https://github.com/edwardkim/rhwp/pull/9424#pullrequestreview-6073269287) · [최종 CI](https://github.com/edwardkim/rhwp/actions/runs/45809323676) · [최종 정책 감사](https://github.com/edwardkim/rhwp/actions/runs/45809566032) · [병합 커밋](https://github.com/edwardkim/rhwp/commit/492f398bce48e1e695786d617a18bf9f62d3fc91)

같은 날 20시 21분(KST) `Close Issues on devel Push`가 이슈 #9139를 `completed`로 닫았다. 병합 후 nextest는 원 코드 CI에서 확보한 Archive B·C·D·E의 실행 시간 자료를 확인해 [측정 자료 커밋 `e27e7569cb`](https://github.com/edwardkim/rhwp/commit/e27e7569cb162c0fa9b83c9cd936c5f9c60b9be9)에 게시했다. [자동 종료 기록](https://github.com/edwardkim/rhwp/issues/9139#issuecomment-7190631396) · [병합 후 측정 자료 갱신](https://github.com/edwardkim/rhwp/actions/runs/45809886084)

## Merge 후 contributor PR comment 계획

cozykbin 님의 첫 기여를 환영하고, 원 기여의 본문·표 셀 자동 줄바꿈과 메인테이너의 본문 수식 선택 보완을 각각 설명한다. 병합 커밋의 실제 화면 두 장과 검증 기록을 연결한다. 연결 이슈에는 원본에서 확인한 16건의 선택·복사 결과와 완료 상태를 기록한다. [본문과 표 셀의 드래그](../assets/pr9424/verified-wrapped.png) · [수식과 다음 글자의 선택](../assets/pr9424/verified-equation.png) · [이슈 #9139](https://github.com/edwardkim/rhwp/issues/9139)
