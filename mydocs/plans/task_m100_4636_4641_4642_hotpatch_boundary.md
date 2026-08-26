# Task #4636, #4641, #4642 수행계획 - 개발용 핫패치 경계 소유권 정리

- Issue: [#4636](https://github.com/edwardkim/rhwp/issues/4636), [#4641](https://github.com/edwardkim/rhwp/issues/4641), [#4642](https://github.com/edwardkim/rhwp/issues/4642)
- Branch: `task_m100_4636_4641_4642-hotpatch-boundary-20260814`
- Base: `upstream/devel` `be7dabdd170117d6022656063b0482f04361bfcb`
- 작성일: 2026-08-14 KST

## 문제와 범위

개발 전용 렌더 코드 교체의 소켓과 감시자가 `WasmBridge`, `CanvasView`, `subsecond-runtime`에
나뉘어 있어 일반 Studio 클래스의 멤버가 프로덕션 번들에 남는다. Rust 쪽도 기능이 꺼진 일반
빌드에서 벤더 이름 모듈을 컴파일하고, 공용 helper가 벤더 trait 제네릭을 노출한다.

이번 PR은 이 세 이슈의 공통 경계만 고친다. 핫패치 기능 자체, WASM의 JavaScript 공개 시그니처,
렌더러 계산과 문서 포맷 동작은 바꾸지 않는다.

## 구현 계획

1. `main.ts`의 DEV 동적 import가 소켓·패치 계수·리비전 감시를 시작하고, `CanvasView`는 일반
   화면 재도색만 소유하게 한다.
2. `WasmBridge`의 개발 전용 메서드와 상태를 제거하고, 개발 런타임에 필요한 일반 wasm export와
   선형 메모리 조회만 빌려준다.
3. 프로덕션 번들 검사가 `dist/assets` 직계 파일만 보지 않고 배포 `dist` 아래의 모든 JavaScript를
   재귀적으로 검사하게 한다.
4. Rust 모듈을 도메인 이름 `render_patch_boundary`로 바꾸고, `HotFunction` 제네릭 helper를
   제거한다. 현재 함수 주소는 점프 테이블을 반영하는 `HotFn::current(...).ptr_address()`를
   경계 구현 안에서 직접 사용한다.

## 검증 계획

- `npm --prefix rhwp-studio test`
- `npm --prefix rhwp-studio run build`
- `node --test scripts/frontend-studio-dist.test.mjs`
- `cargo test --profile release-test --target-dir target/pr-review --lib wasm_api::render_patch_boundary`
- `git diff --check`

## 완료 조건

- 일반 Studio 클래스와 프로덕션 번들에 개발용 소켓·감시자·벤더 식별자가 남지 않는다.
- 배포 산출물의 중첩 JavaScript chunk도 번들 감시 대상이다.
- Rust의 일반 모듈명과 공용 helper가 벤더 trait 계약을 노출하지 않으며, 패치 리비전은 적용된
  함수 주소를 계속 반영한다.
