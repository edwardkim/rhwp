# PR #7670 통합 호환 보정 — 탭 단위 helper와 기존 채우기 대화상자 검사

## 충돌 해소

원 head `4967286840897ed2ab611f8240d93c2131f1493a`를 기준
`2efefc6a1f9812a188e32c1cb283e9ef7962ea3e` 위에 체리픽할 때
`para-shape-dialog.ts`, `para-shape-tab-builders.ts`의 import가 충돌했다.
최신 `devel`의 `fill-selection`과 원 PR의 `tab-units` import를 모두 유지했다.
원 기여는 탭 위치·구역 기본 탭 간격의 1pt=200 저장 단위 변환이다.

## 기존 검사 helper의 의존성 보완

통합 후보의 전체 Studio 실행은 2,010개 중 1,995 PASS / 15 FAIL이었다.
실패 15개 모두 `issue-7661-fill-preservation.test.ts`의 ParaShapeDialog 실행에서
`ReferenceError: tabUnitsToPt is not defined`였다. 이 검사는 import를 제거한 제품 메서드를
수동 VM context에서 실행하므로, 새로 사용하는 실제 탭 helper도 전달해야 한다.

생산 helper `tabUnitsToPt`를 직접 import해 VM context에 추가했다. 채우기 상태·테두리 소유·
대화상자 재사용 assertion은 유지했으며 검사 삭제·skip·기준값 완화는 없다.

- 수정 후 focused: 채우기 보존 30개 + 탭 단위 3개 = 33 PASS / 0 FAIL / 0 SKIP.
- 명령: `node --test rhwp-studio/tests/issue-7661-fill-preservation.test.ts rhwp-studio/tests/tab-units.test.ts`
- raw 로그: ignored `output/pr-review/semanticist21-20261015/logs/`의
  `studio-full-unit-20261015.log`, `tab-dialog-harness-compat-after-20261015.log`.

전체 Studio 재실행2010/2010PASS 및 실제150pt/default50pt 대화상자·HWP/HWPX 저장 재열기를
확인했다. 캐시 보정 후 새 WASM의 초기 실제 응답 SHA를 확인한 브라우저에서도 같은 UI 계약을
재실행했고, 출력 HWP/HWPX가 미계산 탭의 재생성 캐시를 싣지 않는 것까지 대조했다.
최종 Print·Rust/WASM·원격 상태는 [개별 review](pr_7670_review.md)와
[공통 검증 기록](../assets/semanticist21_20261015/validation.md)에 연결한다.
