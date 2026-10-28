# Task M100 #3319 구현 계획 — SO-SUEOP HWPX OLE 선택

## 변경 후보

| 영역 | 변경 방향 | 판정 근거 |
| --- | --- | --- |
| `rhwp-studio/e2e/issue-2069-ole-object-selection.test.mjs` | `한셀OLE` 외에 `SO-SUEOP.hwpx`를 실제 로드해 1쪽 OLE 클릭과 ref 일치를 검증 | 기존 E2E는 문제 fixture를 열지 않아 회귀를 놓쳤다. |
| `rhwp-studio/src/engine/input-handler-picture.ts` | 필요한 경우 OLE layout control의 type·원본 참조·cell/header 문맥을 `PictureObjectRef`로 보존 | 개체 선택은 이 변환 결과를 사용한다. |
| Rust render/layout 경로 | 필요한 경우 HWPX OLE preview 노드에 page control layout에 필요한 원본 control 메타를 보존 | Studio가 렌더된 OLE와 모델 control을 같은 대상으로 식별해야 한다. |

## 구현 원칙

- 먼저 E2E 실패를 재현한다. `한셀OLE`만 통과하는 것으로 #3319를 완료 처리하지 않는다.
- Rust에서 원인을 고칠 때 control ref와 page-local bbox를 생성하는 기존 OLE 경로를 재사용한다.
- Studio 보정은 `image`, `shape`, `equation`의 hit 순서·z-order 규칙을 바꾸지 않고 `ole`에만
  누락된 동일 메타를 전달한다.
- 테스트는 선택 상태뿐 아니라 선택 ref와 클릭한 layout control의 `(secIdx, paraIdx, controlIdx)`
  동일성을 검사한다.

## 검증 계획

1. `CARGO_TARGET_DIR=target/task-3319-hwpx-ole-selection CARGO_INCREMENTAL=0 wasm-pack build --target web --out-dir pkg`
2. headless Studio E2E `issue-2069-ole-object-selection.test.mjs`
3. Rust 변경 시 해당 integration test와 `cargo test --profile release-test --tests` 범위를 변경 위험도에 맞춰 실행
4. `npm run typecheck` 및 `git diff --check`
