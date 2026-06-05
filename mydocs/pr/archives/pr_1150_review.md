# PR #1150 검토 — 표 셀 내부 도형의 개체 속성 다이얼로그 지원

## 1. PR 메타

| 항목 | 값 |
|---|---|
| PR | #1150 |
| 제목 | Task #1138: 표 셀 내부 도형의 '개체 속성' 다이얼로그 지원 |
| 작성자 | johndoekim |
| base ← head | `devel` ← `local/task1138` |
| 상태 | OPEN |
| mergeable | MERGEABLE |
| 변경 규모 | 20 files, +1338 / -26 |
| 연결 이슈 | closes #1138 |

## 2. 문제 요약

`rhwp-studio`에서 표 셀 내부의 도형을 선택한 뒤 우클릭 → `개체 속성(P)...`을 실행하면,
선택된 객체가 셀 내부 control임에도 외부 본문 좌표 path로 shape properties API를 호출해
다음 오류가 발생한다.

```text
지정된 컨트롤이 Shape이 아닙니다
표 컨트롤이 아닙니다
```

PR의 핵심 원인 분석은 타당하다.

```text
1. 표 셀 내부 Rectangle/Line/Ellipse/Path render node에 cell context가 없다.
2. getPageControlLayout JSON에도 cellIdx/cellParaIdx/outerTableControlIdx가 없다.
3. UI handler가 cellPath를 만들 수 없어 기존 외부 shape API로 들어간다.
```

## 3. 변경 내용

Rust:

```text
src/renderer/render_tree.rs
  - Rectangle/Line/Ellipse/Path에 cell_index, cell_para_index, outer_table_control_index 추가

src/renderer/layout/shape_layout.rs
src/renderer/layout/table_cell_content.rs
src/renderer/layout/table_layout.rs
src/renderer/layout/table_partial.rs
  - 표 셀 내부 도형 layout 시 cell context 전달

src/document_core/queries/rendering.rs
  - Rectangle/Line/Ellipse/Path export JSON에 cellIdx/cellParaIdx/outerTableControlIdx 추가

src/document_core/commands/object_ops.rs
src/wasm_api.rs
  - getCellShapePropertiesByPath / setCellShapePropertiesByPath API 추가

tests/issue_1138.rs
  - inner-table-01.hwp 셀 내부 사각형 속성 조회/변경/경계 케이스 테스트 추가
```

TypeScript:

```text
rhwp-studio/src/core/types.ts
rhwp-studio/src/core/wasm-bridge.ts
rhwp-studio/src/ui/picture-props-dialog.ts
rhwp-studio/src/engine/cursor.ts
rhwp-studio/src/engine/input-handler-picture.ts
rhwp-studio/src/engine/input-handler-mouse.ts
rhwp-studio/src/command/commands/insert.ts
```

핵심 흐름:

```text
findPictureAtClick
  -> selectedPictureRef에 outerTableControlIdx 보존
  -> insert:picture-props handler에서 shape/line + cellIdx + outerTableControlIdx이면 cellPath 구성
  -> PicturePropsDialog가 cellPath 분기에서 by_path API 호출
```

## 4. 로컬 병합 검증

검토 브랜치:

```text
local/pr1150-review
```

현재 `devel`은 #1149까지 반영되어 PR base보다 앞서 있다.

```text
PR base: a5b78854
current devel: 4ca3e1e6
```

병합 테스트:

```text
git checkout -B local/pr1150-review devel
git merge --no-commit --no-ff local/pr1150-upstream
```

결과:

```text
conflict: none
```

## 5. 메인테이너 정리

PR 코드/문서에 작은 정리 사항이 있어 검토 브랜치에서 추가로 반영했다.

```text
1. src/renderer/layout/shape_layout.rs
   - 중복 #[allow(clippy::too_many_arguments)] 제거

2. tests/issue_1138.rs
   - module comment에서 이미 제거된 picture by_path API 언급 삭제

3. mydocs/report/task_m100_1138_report.md
   - 최종 scope를 "picture/shape"가 아니라 "shape 한정"으로 정정
   - ImageNode/picture API가 최종 구현에 남아 있는 것처럼 보이는 표기 정정
```

## 6. 검증

GitHub PR CI:

```text
CI: SUCCESS
Render Diff: SUCCESS
CodeQL: SUCCESS
```

로컬 검증:

```text
cargo fmt --all -- --check
  success

cargo test --test issue_1138 -- --nocapture
  7 passed

cargo clippy --lib -- -D warnings
  success

npx tsc --noEmit
  success

cargo test --test svg_snapshot
  8 passed

docker compose --env-file .env.docker run --rm wasm
  success

maintainer visual judgment
  2026-05-27: pass
```

## 7. 검토 의견

수용 가능한 점:

```text
1. render tree -> layout -> getPageControlLayout -> wasm API -> UI dialog까지 필요한 레이어가 모두 연결된다.
2. 기존 외부 도형 API 시그니처는 유지하고, 표 셀 내부 도형에는 신규 by_path API를 추가한다.
3. picture는 기존 paragraph_layout path로 정상 동작한다는 분석에 따라 scope에서 제외했다.
4. 신규 테스트가 정상 경로와 오류 경계를 함께 다룬다.
```

주의할 점:

```text
1. nested table 2단계 이상은 path schema상 확장 가능하지만 이번 PR 검증 대상은 1-level이다.
2. 셀 안 글상자 안 picture, group child, textbox content cell context는 후속 task로 남긴다.
3. UI 시각 판정은 maintainer가 `inner-table-01.hwp`로 직접 확인하는 게 좋다.
```

## 8. 권장안

**수용 권장.**

조건:

```text
1. WASM 빌드
2. maintainer 시각 판정
   - samples/inner-table-01.hwp
   - 표 셀[5] 사각형 도형 우클릭 -> 개체 속성(P)
   - 다이얼로그 표시 및 크기/색 변경 확인
3. 통과 시 현재 검토 브랜치의 메인테이너 정리 패치까지 함께 커밋
4. local/devel -> devel -> origin/devel 순서로 반영
5. PR #1150 코멘트 후 close/정리
```

## 9. PR 코멘트 초안

```text
johndoekim님, PR 감사합니다.

표 셀 내부 도형에서 개체 속성 다이얼로그가 열리지 않던 원인을 render tree의 cell context
누락과 UI handler의 cellPath 부재까지 추적한 분석이 타당했습니다.

검토 결과, 현재 devel(#1149 포함) 위에서도 충돌 없이 병합되며 다음 검증을 통과했습니다.

- cargo fmt --all -- --check
- cargo test --test issue_1138 -- --nocapture
- cargo clippy --lib -- -D warnings
- npx tsc --noEmit
   - cargo test --test svg_snapshot
   - docker compose --env-file .env.docker run --rm wasm

수용 전 maintainer 쪽에서 작은 문서/주석 정리와 중복 attribute 제거를 함께 반영했습니다.
WASM 빌드와 `inner-table-01.hwp` maintainer 시각 판정도 통과했습니다. devel에 반영하겠습니다.
```
