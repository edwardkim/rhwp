# Task #4635, #4643 수행계획 - Studio 개발용 핫패치 경계 정리

- Issue: [#4635](https://github.com/edwardkim/rhwp/issues/4635), [#4643](https://github.com/edwardkim/rhwp/issues/4643)
- Branch: `task_m100_4635_4643-hotpatch-20260814`
- Base: `upstream/devel` `c121f6185`
- 작성일: 2026-08-14 KST

## 문제와 범위

Studio의 개발 전용 렌더 코드 교체 경로에는 엔진 결과 코드와 계수 제어 흐름이 서로 다른
리터럴을 쓰는 지점, WASM 초기화 순서가 틀렸을 때 조용히 비활성화되는 지점, 동적 import 실패가
앱 초기화를 중단시키는 지점이 남아 있다. 이전 자동 셀 투명선 기능에서 남은 주석 구독과 실제
구독자가 없는 이벤트 발행도 함께 정리한다.

이번 작업은 개발 전용 런타임의 실패 관측성과 소스 계약만 다룬다. 렌더러 계산, HWP/HWPX 파서,
WASM 공개 ABI, 사용자 문서의 출력은 변경하지 않는다.

## 구현 경계

1. 적용 요청 결과 코드 `patch-dispatched`를 진단 표와 누적 계수가 공유하는 상수로 만든다.
2. CanvasView가 WASM 초기화 전에 생성된 경우 개발 콘솔에 기동 순서 원인을 남긴다.
3. 개발 전용 런타임 동적 import 또는 devtools 연결 실패는 경고를 남기고 일반 Studio 초기화를
   계속 진행하게 한다.
4. 더는 소비자가 없는 `transparent-borders-changed` 이벤트 발행과 그 주석 처리된 구독 흔적을
   제거한다.
5. 프로덕션 번들 감시 표지에서는 이미 도메인 이름이 된 `rebuildDerivedState`를 제외하고, 문서의
   Cargo.toml 줄 참조를 현재 위치로 바로잡는다.

## 검증 계획

- `npm --prefix rhwp-studio test`로 개발 런타임 결과 코드·누적 계약과 초기화 경계 source contract를
  확인한다.
- `npm --prefix rhwp-studio run build`와 `node --test scripts/frontend-studio-dist.test.mjs`로
  번들 감시와 빌드 산출물을 확인한다.
- `npm --prefix rhwp-studio test`로 Studio 단위 테스트 전체를 실행한다.
- `git diff --check`와 TypeScript 컴파일을 포함한 build 성공을 확인한다.

## 완료 조건

- 코드 계수와 진단 표가 같은 적용 요청 결과 상수를 소비한다.
- 개발용 초기화 순서와 동적 import 실패가 조용히 사라지지 않으며, 일반 Studio 초기화를 막지 않는다.
- 사용하지 않는 이벤트와 주석 처리된 구독이 소스에 남지 않는다.
- 번들 감시가 개발 전용 표지를 검사하되 일반 도메인 메서드 이름을 오탐하지 않는다.
