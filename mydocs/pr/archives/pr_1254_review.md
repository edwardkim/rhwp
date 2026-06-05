# PR #1254 검토 - 사각형 글상자 안 picture 선택/속성/삽입 지원

- **작성일**: 2026-06-03
- **PR**: #1254 (OPEN)
- **제목**: `사각형 글상자 안 picture 클릭 hit-test / 속성 / 삽입 지원 (#1171)`
- **컨트리뷰터**: @johndoekim
- **연결 이슈**: #1171
- **base/head**: `devel` <- `fix/issue-1171`
- **Head SHA**: `cf5fe54b33e15fd39313a0bc47e75fa83dedf862`
- **merge-base**: `a9c49dd6`
- **현재 local/devel**: `3bd66137`
- **규모**: 17 files, +1009 / -24, 7 commits
- **GitHub 상태**: `MERGEABLE`
- **CI**: `CI`, `CodeQL`, `Render Diff` 통과
- **PR 댓글/리뷰**: 없음

## 1. PR 요약

PR #1254는 `samples/tac-img-02.hwp`에서 사각형 글상자 안에 들어 있는 picture를 rhwp-studio에서 직접 선택/속성 변경할 수 없던 문제를 처리한다.

기존 구조는 다음과 같다.

```text
본문 paragraph
  -> Shape(Rectangle, InFrontOfText)
     -> text_box
        -> paragraph
           -> picture
```

기존 구현은 글상자 컨테이너 Shape까지만 hit-test/렌더 컨트롤 수집이 되고, 내부 picture에는 표 셀과 같은 위치 식별자(`cellPath`)가 부여되지 않았다. 그 결과 프런트엔드는 글상자 텍스트 편집으로 먼저 진입하고, picture 속성 API도 표 셀 전용 경로만 처리했다.

이번 PR은 글상자를 `cell_index=0` sentinel을 가진 단일 셀 컨테이너처럼 취급해, 표 셀 picture와 같은 by-path 흐름으로 선택/속성 변경을 가능하게 한다.

## 2. 주요 변경 범위

| 파일 | 변경 |
|---|---|
| `src/renderer/layout/shape_layout.rs` | 글상자 내부 picture에 `CellContext` sentinel 부여 |
| `src/document_core/queries/rendering.rs` | Rectangle/Shape 수집 후 자식 컨트롤 재귀 탐색 유지 |
| `src/document_core/commands/object_ops.rs` | by-path picture getter/setter에서 Shape text_box 경로 지원 |
| `rhwp-studio/src/engine/input-handler-picture.ts` | 글상자 Shape와 내부 picture가 함께 hit될 때 picture 우선 선택 |
| `rhwp-studio/src/engine/input-handler-mouse.ts` | 글상자 텍스트 편집 진입 전 picture hit-test 선처리 |
| `rhwp-studio/src/engine/input-handler-table.ts` | 글상자 위 이미지 드롭은 본문 floating sibling으로 삽입 |
| `tests/issue_1171_textbox_picture_cellpath.rs` | 글상자 picture `cellPath` 노출 및 by-path 속성 round-trip 검증 |
| `rhwp-studio/e2e/*1171*.test.mjs` | 글상자 picture 선택/속성 변경/이미지 삽입 E2E 추가 |
| `mydocs/*task_m100_1171*` | 계획, 단계 기록, 완료 보고서 추가 |

## 3. 타당한 부분

### 3.1 글상자 내부 picture를 표 셀과 같은 식별 체계로 연결한다

`Shape text_box`를 `cell_index=0` sentinel 컨테이너로 취급하는 방식은 새 API를 늘리지 않고 기존 `cellPath` 기반 속성 변경 흐름을 재사용한다.

`layout_picture`에 `CellContext`를 전달하고, `object_ops.rs`의 mutable resolver도 `Control::Shape` arm을 추가해 immutable resolver와 의미를 맞춘 점이 좋다.

### 3.2 hit-test 우선순위를 실제 사용자 기대와 맞춘다

글상자 위 picture를 클릭했을 때 텍스트 편집으로 들어가지 않고 picture 객체 선택으로 진입해야 한다. PR은 `input-handler-mouse.ts`에서 `hit.isTextBox` 텍스트 편집 분기 전에 `findPictureAtClick`을 먼저 호출해 이 경로를 보정한다.

### 3.3 글상자 위 이미지 드롭 동작은 한컴 방식에 맞춘다

컨트리뷰터 설명대로 한컴은 글상자 위에 이미지를 넣으면 글상자 내부 셀 경로가 아니라 글상자의 본문 sibling floating object로 삽입한다.

PR은 `isTextBox` hit의 `cellPath`를 표 셀 삽입 경로로 넘기지 않고, 글상자를 소유한 본문 문단에 floating picture를 삽입한다. 이 방향은 기존 표 셀 이미지 삽입과 의미가 충돌하지 않는다.

### 3.4 자동 테스트가 문제 구조를 직접 겨냥한다

Rust 테스트는 `samples/tac-img-02.hwp`의 문단 25/44에서 글상자 picture가 `cellPath=[{controlIndex:0, cellIndex:0, cellParaIndex:0}]`로 노출되는지 확인한다.

E2E는 `findPictureAtClick`과 by-path property round-trip, 글상자 위 이미지 드롭 -> 본문 sibling 삽입까지 포함한다.

## 4. 주의 사항

### 4.1 PR branch가 현재 devel보다 뒤처져 있다

PR 자체 변경은 merge-base `a9c49dd6..pr/1254` 기준 17개 파일이다. 단순 `local/devel..pr/1254` diff를 보면 최근 devel 문서/샘플 삭제처럼 보이는 항목이 섞이지만, 이는 PR head가 최신 devel을 포함하지 않아서 생기는 비교 착시다.

현재 `git merge-tree local/devel pr/1254`는 충돌 없이 merge tree를 생성한다. 따라서 검증 브랜치에서 현재 `local/devel` 위로 병합해 테스트해야 한다.

### 4.2 `collect_controls` 재귀 확장의 영향 범위

`rendering.rs`에서 Rectangle/Shape 노드를 수집한 뒤 `return`하지 않고 자식 탐색을 계속한다. 글상자 내부 picture 수집에는 필요하지만, 다른 Shape 자식 컨트롤이 예기치 않게 control layout에 노출되는지 확인해야 한다.

현재 주석과 구현상 장식 노드는 section/para/control 좌표가 없어 방출되지 않는 구조라 위험은 제한적이다.

### 4.3 nested picture hit-test의 z-order는 후속 확인 필요

새 우선 패스는 Shape와 nested picture가 모두 hit될 때 첫 nested picture를 반환한다. 동일 글상자 내부에서 picture가 여러 개 겹치는 특수 케이스는 별도 z-order 정렬이 필요할 수 있다.

이번 이슈의 샘플 목적에는 충분하지만, 향후 중첩 picture가 겹치는 문서가 나오면 재검토 대상이다.

## 5. 권장 검증

현재 `local/devel` 기준 검증 브랜치를 만들고 PR #1254를 병합한 뒤 다음을 실행한다.

```text
git diff --check HEAD
cargo fmt --all --check
cargo test --test issue_1171_textbox_picture_cellpath -- --nocapture
cargo test --test issue_1151_cell_picture_properties -- --nocapture
cargo test --test issue_1183_cell_internal_picture_save -- --nocapture
cargo test --tests
docker compose --env-file .env.docker run --rm wasm
cd rhwp-studio && npm run build
cd rhwp-studio && node e2e/textbox-picture-1171.test.mjs
cd rhwp-studio && node e2e/textbox-picture-insert-1171.test.mjs
```

시각/동작 판정 후보:

| file | 확인 항목 |
|---|---|
| `samples/tac-img-02.hwp` | 6/7쪽 글상자 내부 picture 직접 선택 |
| `samples/tac-img-02.hwp` | picture 속성 대화상자 열기/수정 |
| `samples/tac-img-02.hwp` | 글상자 위 이미지 드롭 시 본문 sibling floating 삽입 |

## 6. 권장 처리

권장안: **수용 후보로 진행한다.**

근거:

- PR head 기준 `CI`, `CodeQL`, `Render Diff`가 모두 통과했다.
- 현재 `local/devel` 기준 사전 병합 충돌이 없다.
- 구현 방향이 기존 표 셀 picture by-path 흐름과 정합적이다.
- Rust/E2E 테스트가 이번 이슈의 핵심 구조를 직접 겨냥한다.
- 글상자 위 이미지 드롭 동작도 한컴의 sibling floating 삽입 방식으로 정리되어 있다.

단, PR head가 최신 devel보다 뒤처져 있으므로 반드시 검증 브랜치에서 현재 `local/devel` 위에 병합한 뒤 테스트/WASM/메인테이너 동작 판정을 통과해야 한다.

## 7. 다음 승인 요청

다음 단계로 진행하려면 작업지시자 승인이 필요하다.

권장 절차:

```text
1. `local/pr1254-verify` 브랜치를 현재 `local/devel`에서 생성
2. PR #1254를 병합
3. Rust 회귀 테스트와 rhwp-studio E2E 실행
4. WASM/Studio 빌드
5. `samples/tac-img-02.hwp`로 메인테이너 동작/시각 판정
6. 판정 통과 후 local/devel 반영 및 PR/issue 종료 처리
```
