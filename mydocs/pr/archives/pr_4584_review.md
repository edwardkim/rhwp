---
kind: pr-review
status: local-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4584 리뷰 - 파생 렌더 상태 전면 재구성

## 라우팅과 접수

```text
base route: collaborator_external_pr.md
modifiers: intake_and_review.md, local_validation.md,
  visual_fixture_evidence.md, multi_pr_update_branch.md
```

| 항목 | 문서 작성 시점 기록 |
| --- | --- |
| 원 PR | [#4584](https://github.com/edwardkim/rhwp/pull/4584) · @humdrum00001010 |
| 관련 이슈 | [#4576](https://github.com/edwardkim/rhwp/issues/4576) |
| 원 head | `692c546c2ed6af68f1843001fc04a444c1e19b63` |
| 규모 | 5 files, +346/-18 |
| 원 PR 상태 | `MERGEABLE`, `CLEAN`; source head CI·CodeQL·Canvas visual diff 성공 |
| 최초 누적 기준선 | `upstream/devel` `a70797db431e42cb29ee8140ddb0e3259eb99ae2` |
| rebase 기준선 | `upstream/devel` `4f9e4ae694d53162a1a8fd2e2606562d7635085d` |
| 누적 적용 | `beec1758d`, `4d5a2fd2a`, `b83adc610`, `497011131` |
| 로컬 검증 후보 | rebase 전 `a08be5d1051016adb0378c40fc0010b677628c15` |
| 현재 rebase 후보 | `ed8e0387ad249cacae8edab85dd2283ea559ba21` |

## 변경 판단

핫패치 뒤 원본 IR은 같아도 조합 문단, 측정 캐시, 페이지네이션과 페이지 트리가 이전 코드에서 만든
상태로 남으면 새 구현과 이전 레이아웃이 섞인다. 이 PR은 기존 snapshot 복원 경로에 흩어져 있던
전면 재구성 순서를 `DocumentCore::rebuild_derived_state()`로 모으고, Subsecond 무효화 진입점을
`&mut self`로 넓혀 해당 재구성을 호출한다.

문서 본체, snapshot store, event log, binary-data epoch를 변경하지 않고 문서에서 파생된 상태만
다시 만들므로 일반 편집이나 문서 교체의 epoch와 혼동하지 않는다. `subsecond` 이름을
`DocumentCore` 내부 계약에 넣지 않아, snapshot 복원에도 같은 연산을 사용할 수 있다.

## 누적 통합과 완료한 검증

- 원 PR commit은 지정 순서의 첫 단계로 누적 branch에 적용했다. 이후 #4590, #4594, #4597을 같은
  기준선 위에 적용했으며 충돌은 없었다.
- `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --lib
  issue_4576_rebuild_derived_state_recomputes_composition_and_pagination --no-fail-fast`를 실행해
  회귀 1건이 통과했다.
- rebase 전 누적 후보에서 `cargo nextest run --cargo-profile release-test --target-dir target/pr-review
  --tests --test-threads 12 --no-fail-fast`를 실행해 5,764건 통과, slow 7건, 정책 skip 36건으로
  종료했다.
- 같은 rebase 전 후보에서 `cargo fmt --check`, `cargo clippy --all-targets --target-dir target/pr-review --
  -D warnings`, `cargo build --release --target-dir target/pr-review`, `cargo test --release
  --target-dir target/pr-review --lib`를 실행해 모두 통과했다. release lib는 3,498건 통과,
  13건 ignored였다.
- Native Skia 3종(58/2/4), `wasm-pack build --target web --out-dir pkg`, Studio TypeScript 검사와
  `npm --prefix rhwp-studio test`(847건 통과)도 누적 후보에서 통과했다.
- renderer/layout의 일반 시각 회귀는 원 head의 Canvas visual diff가 성공했다. 이번 변경은
  `subsecond-dev` 전용 무효화 경로이며 production 문서의 PDF fidelity를 직접 바꾸지 않는다.

## 한계와 권고

실제 브라우저에서 Rust 소스를 수정해 패치된 조합·페이지네이션이 즉시 재구성되는 end-to-end
hot-patch 왕복은 이 누적 검토에서 자동화하지 않았다. Dioxus endpoint, Vite proxy와 개발 WASM
응답은 로컬에서 모두 `200`으로 확인했지만, 이는 패치 적용 성공의 증거가 아니다. 이 한계는
#4576의 Rust 회귀와 source CI 범위를 넘어서는 개발 도구 런타임 검증 항목으로 남긴다.

**최종 권고: 수용.** 원 PR의 구조적 회귀와 누적 전체 회귀가 통과했고, 파생 상태 재구성 범위가
문서 원본 상태와 분리되어 있다. 실제 merge 전에는 통합 PR 최신 head의 CI와 작업지시자 승인을
다시 확인한다.
