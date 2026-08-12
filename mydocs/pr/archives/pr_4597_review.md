---
kind: pr-review
status: local-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4597 리뷰 - 편집 중 재도색의 Subsecond 경계

## 라우팅과 접수

```text
base route: collaborator_external_pr.md
modifiers: intake_and_review.md, local_validation.md,
  visual_fixture_evidence.md, multi_pr_update_branch.md
```

| 항목 | 문서 작성 시점 기록 |
| --- | --- |
| 원 PR | [#4597](https://github.com/edwardkim/rhwp/pull/4597) · @humdrum00001010 |
| 관련 이슈 | [#4577](https://github.com/edwardkim/rhwp/issues/4577) |
| 원 head | `758d40ae9661d192bfb362f3e8c3acff30045919` |
| 규모 | 3 files, +507/-32 |
| 원 PR 상태 | `MERGEABLE`, `CLEAN`; source head CI·CodeQL·Canvas visual diff 성공 |
| 선행 의존 | #4584의 `&mut self` 파생 상태 재구성 뒤에 적용 |
| 누적 적용 | `a78def626`, `f0a771f9f`, `33f92a494` |
| 로컬 검증 후보 | rebase 전 `a08be5d1051016adb0378c40fc0010b677628c15` |
| 현재 rebase 후보 | `ed8e0387ad249cacae8edab85dd2283ea559ba21` |

## 변경 판단

wasm `HotFn::current`는 등록된 함수 하나만 새 patch 구현으로 전환하므로, JS export 아래의 호출이
자동으로 재링크되지는 않는다. 이 PR은 편집 중 부분 재도색, overlay/flow image와 layer tree 경로를
`hot_render_boundaries!` 하나의 목록으로 등록한다. 목록은 dispatcher, revision 조각, export manifest를
함께 생성하고, dispatcher의 `deny(dead_code)`로 export 배선 누락을 빌드 오류로 만든다.

부분 재도색 구현은 10인자라 subsecond의 9인자 `HotFunction` 상한을 넘었다. 좌표 네 개를
`BoundingBox`로 묶어 Rust 내부 인자를 줄였지만 JS API 시그니처는 유지했다. profile 없는
`getPageLayerTree`도 profile 경계로 위임해 같은 구현을 우회하지 않게 했다.

## 누적 통합과 완료한 검증

- #4584와 `src/wasm_api.rs`를 함께 변경하므로 사용자 지정 순서대로 #4584 뒤, #4590·#4594 뒤에
  적용했다. 충돌 없이 누적했고 `git diff --check`가 통과했다.
- rebase 전 누적 후보에서 전체 nextest 5,764건 통과, fmt·clippy·release build·release lib test·Native Skia
  3종(58/2/4)·WASM build가 통과했다.
- `src/wasm_api/subsecond_boundary.rs`의 compile-time registry 검증은 누적 build와 test에 포함돼,
  등록한 경계의 dispatcher·revision·export 배선 정합을 확인한다.
- 원 head의 CI, CodeQL, Canvas visual diff는 모두 성공했다. 일반 Canvas visual diff는 production
  renderer 회귀를 확인하는 근거이고, `subsecond-dev` 실제 patch 적용 성공을 대신하지는 않는다.
- rebase 전 후보에서 Dioxus `dx serve`는 `127.0.0.1:7711`, Studio Vite는 `0.0.0.0:7700`으로 기동했고,
  Vite·Dioxus·Vite WASM proxy 응답이 모두 `200`이었다.

## 한계와 권고

실제 브라우저에서 소스를 저장해 patch wasm을 전달하고, 타이핑한 줄의 부분 재도색이 새 구현으로
바뀌는 과정을 자동화하지 않았다. 현재 확인한 HTTP endpoint와 compile-time registry는 그 전제 조건이며,
비동기 patch instantiate의 성공 증거는 아니다. `getPageInfo`, source image bytes와 CanvasKit replay plan은
렌더 경계 밖에 남긴 이유가 source stage 문서에 기록돼 있으며, 해당 후속 범위는 #4595·#4596에서 다룬다.

**최종 권고: 수용.** 경계의 누락과 revision drift를 한 등록 목록에서 차단하고, JS API를 유지하면서
부분 재도색 경로를 hot-patch 대상으로 넓혔다. merge 전에는 통합 PR 최신 head의 CI와 작업지시자
승인을 다시 확인한다.
