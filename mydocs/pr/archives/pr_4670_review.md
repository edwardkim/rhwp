---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4670 검토 - Studio JavaScript 브리지와 플러그인 호스트

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4670](https://github.com/edwardkim/rhwp/pull/4670) |
| 작성자 / source | @planet6897 / `feat/studio-bridge-plugin` |
| base / source head | `devel` / `3cd54cfa5a658992f95714e71c48f07a776359df` |
| 규모 | 55 files, +4,400 / -99, 1 commit |
| reviewer | @jangster77 지정 완료 |
| mergeable 참고값 | 작성 시점 `CONFLICTING` / `DIRTY` |
| 관련 이슈 | 없음 |
| 통합 검토 branch | `review/planet6897-20260812-r2` |

웹 페이지가 studio 인스턴스를 생성하고 커맨드·HwpCtrl 플러그인·문서 바이트를 호출하는
automation, plugin host, RPC, standalone HwpCtrl 연동을 추가한다. Canvas 조판 규칙이나
renderer geometry를 바꾸지 않으므로 PDF/페이지 fidelity sweep은 적용 대상이 아니다. 대신
실제 브라우저 E2E와 WASM·번들 검증을 수행했다.

## 충돌과 메인터너 보정

원 PR은 최신 `devel`의 `rhwp-studio/src/main.ts`와 충돌했다. 누적 검토에서 plugin
automation 초기화는 유지하고, 최신 `devel`의 embed 모드 file/edit command 필터를 보존해
두 기능이 함께 적용되도록 해소했다.

추가로 아래 보정을 `6ba0838d7`에 분리했다.

- `PluginHostFacade`가 studio 소유 `loadDocument`/`createBlankDocument` 뒤에 swap
  알림을 다시 보내던 중복을 제거했다. studio 원 경로가 이미 알림을 내보내므로, plugin 호출에서
  한 문서 교체가 두 번 관측될 수 있었다.
- dev-probe 플러그인의 실제 문서 교체와 E2E TC1.1을 추가해 swap 알림이 정확히 한 번임을
  고정했다.
- 최신 dispatcher의 `dispatchWithResult(...).ok` 계약에 맞게 chrome-mode source 계약을
  갱신했다.
- Node 24의 `ℹ pass/fail` 테스트 요약도 `gate_bridge.mjs`가 읽도록 해, 성공한 unit
  test를 `-1 pass / -1 fail`로 잘못 보고하지 않게 했다.

## 완료한 검증

- `npm test` (studio): 862 passed, 0 failed.
- `npm run gate:bridge -- --only=unit`: studio 862/0, HwpCtrl package 21/0 통과.
- headless browser E2E: automation 23/0, plugin lifecycle 21/0, HwpCtrl plugin 14/0,
  bridge lifecycle 18/0, bridge performance 11/0 통과. TC1.1은 plugin 경유 blank
  document 교체의 swap 알림 1회를 확인했다.
- `npx tsc --noEmit -p tsconfig.ci-unit.json`, `npm run build`,
  `npm --prefix npm/hwpctrl-ocx run gate`: 모두 종료 코드 0. production build에서
  `studio-plugin` 별도 청크가 생성됐다.
- `wasm-pack build --target web --out-dir pkg`: 종료 코드 0.
- 통합 candidate에서 `cargo nextest run --cargo-profile release-test --target-dir
  target/pr-review --tests --test-threads 12 --no-fail-fast`: 5,881/5,881 통과.

## 판단

**통합 수용 권고.** 원 PR 자체는 최신 `devel`과 충돌하므로 직접 merge하지 않는다. 이
검토 branch의 충돌 해소와 메인터너 보정을 포함한 통합 PR을 `devel` 대상으로 만들고, 그
최신 code head의 GitHub Actions와 작업지시자 승인을 확인한 뒤 병합한다. 병합 뒤 원 PR은
통합 반영 사실과 보정 이유를 적어 close한다.
