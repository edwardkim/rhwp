# PR #2528 검토 - 비표준 HWPX 자동 보정 일시 중지

## 메타

| 항목 | 값 |
| --- | --- |
| 원 PR | [#2528](https://github.com/edwardkim/rhwp/pull/2528) |
| 작성자 | @planet6897 |
| base / 검토 head | `devel` / `8edcaf654b0c4a337b983529062d74eb8cc8fcef` |
| 체리픽 순서 | 4 (`8edcaf6`) |
| 충돌 | 없음 |
| 검토 시점 원 PR 상태 | `BEHIND`; 기존 head CI 전체 성공 |

## 변경 및 판단

- #2527의 빈 lineseg 문서에서 `reflowLinesegs()`가 글리프 좌표를 붕괴시키는 동안에는,
  감지 모달과 자동 보정을 실행하지 않고 항상 원본 lineseg 그대로 문서를 연다.
- validation report는 진단 로그로 유지하며, 로드 직후 문서를 clean으로 표시한다.
- `validation-modal.ts`는 reflow 근본 수정 뒤 재도입할 수 있도록 보존하지만 현재 호출하지 않는다.

## 검증

- `npx tsc --noEmit`: 통과
- `npm test`: 456/456 통과
- `npm run build`: 통과
- 호출부 검색 결과 `showValidationModalIfNeeded()`는 정의부만 남고, `reflowLinesegs()`는
  WASM bridge API 정의부만 남는다. 따라서 Studio 문서 로드 경로에서 자동 보정이 실행되지 않는다.
- 기존 head CI의 frontend package gates, Rust/CodeQL/Render Diff, default-feature test shards는 모두 성공했다.

## 리스크와 권고

- 비표준 HWPX 감지 UI와 자동 보정을 임시로 모두 우회하는 완화책이다. 근본 수정 전까지는
  validation warning을 사용자에게 대화상자로 알리지 않는다는 제품 동작 변화를 명확히 유지해야 한다.
- #2527의 실제 문제 fixture가 현재 PR/저장소 검토 범위에 없으므로, 후속 근본 수정 PR에는 해당
  HWPX 원본과 as-is/auto-fix 비교 회귀 테스트를 함께 보존해야 한다.
- 현재 head는 `devel`보다 뒤처져 있으므로 merge 전 최신 `devel` 위 head update와 새 CI가 필요하다.
- 위 조건을 충족하면 **수용 가능**이다.
