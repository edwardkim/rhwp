# PR #1137 검토 보고서

- PR: `#1137`
- 제목: `task 1129: 한컴오피스식 격자 보기와 쪽 테두리 정합`
- 기여자: `jangster77`
- 관련 이슈: `#1129`
- 대상 브랜치: `devel`
- Head: `5de4d1a771c2af5789a5d4a90b0af02b6f11ff40`
- 검토일: 2026-05-27

## 1. 검토 결론

권장안: **현재 PR의 기능 방향을 수용한다.**

PR은 `rhwp-studio`의 격자 보기 토글, 격자 설정 대화상자, 쪽 테두리/배경 설정 대화상자,
HWP5/HWPX/HWP3 쪽 기준 파싱 보강까지 포함한다. 이슈 #1129의 핵심인 격자 보기 활성화와
쪽/종이 기준 분리 방향은 타당하다.

다만 PR 규모가 크고 UI가 노출하는 기능 중 일부가 실제 편집 동작과 아직 연결되지 않았다.
또한 `글 뒤` 격자 표시가 일반 canvas 렌더링 경로에서는 실제로 글 뒤가 아니라 canvas 위에
그려질 수 있다. 이 항목들은 PR 수용을 막는 blocker라기보다 후속 개선 항목으로 분리한다.
기여자가 이어서 보강하거나 maintainer가 직접 구현하면 된다.

## 2. PR 정보

| 항목 | 값 |
|------|-----|
| 상태 | open |
| mergeable | true |
| 변경 파일 | 63 |
| 변경량 | +4139 / -100 |
| PR 댓글 | 없음 |
| 리뷰 | 없음 |
| CI | success |

CI 상태:

```text
CI / Build & Test: success
CodeQL / rust, js-ts, python: success
Render Diff / Canvas visual diff: success
```

로컬 확인:

```text
cargo fmt --all -- --check: success
cargo check --target wasm32-unknown-unknown --lib: success
cd rhwp-studio && npm run build: success
git diff --check devel...pr/1137: fail
```

`git diff --check` 실패 항목:

```text
mydocs/working/task_m100_1129_stage1.md:27: new blank line at EOF.
mydocs/working/task_m100_1129_stage3.md:32: new blank line at EOF.
```

## 3. 변경 요약

주요 코드 변경:

```text
rhwp-studio/index.html
rhwp-studio/src/command/commands/view.ts
rhwp-studio/src/ui/grid-settings-dialog.ts
rhwp-studio/src/ui/page-border-dialog.ts
rhwp-studio/src/view/canvas-view.ts
rhwp-studio/src/view/grid-overlay.ts
rhwp-studio/src/view/grid-settings.ts
src/document_core/queries/rendering.rs
src/parser/body_text.rs
src/parser/hwp3/mod.rs
src/parser/hwpx/header.rs
src/parser/hwpx/section.rs
src/renderer/layout.rs
src/renderer/layout/border_rendering.rs
src/serializer/control.rs
src/wasm_api.rs
```

기능 범위:

```text
1. studio 격자 보기 버튼/메뉴 연결
2. 페이지별 DOM grid overlay 추가
3. 격자 설정 대화상자 확장
4. 쪽 테두리/배경 대화상자 추가
5. HWP5 SectionDef line_grid/char_grid 보존
6. HWPX hp:grid 및 paraPr@snapToGrid 보존
7. HWP3/HWP5/HWPX 쪽 테두리 기준 분리
```

## 4. 발견 사항

### 4.1 후속 개선: 격자 방식 옵션이 실제 편집 동작에 연결되지 않음

`GridSettingsDialog`는 다음 옵션을 제공한다.

```text
상관 없이
자석 효과
격자에만 붙이기
```

관련 위치:

```text
rhwp-studio/src/ui/grid-settings-dialog.ts:66
rhwp-studio/src/view/grid-settings.ts:3
rhwp-studio/src/command/commands/view.ts:196
```

하지만 `snapMode`는 `grid-settings.ts` 내부 상태와 대화상자 왕복에만 사용된다.
실제 표/개체 이동 경로는 여전히 `InputHandler.gridStepMm` 기반 이동 간격만 사용한다.

현재 검색 결과:

```text
snapMode 사용처:
  grid-settings.ts
  grid-settings-dialog.ts
  commands/view.ts

실제 이동/스냅 경로:
  input-handler-table.ts 는 gridStepMm만 참조
```

영향:

```text
사용자가 "자석 효과" 또는 "격자에만 붙이기"를 선택해도 실제 객체/표 이동 정책은 바뀌지 않는다.
이슈 #1129의 기대 동작 중 "격자 방식" 항목이 UI만 구현된 상태가 된다.
```

후속 권장:

```text
1. snapMode를 InputHandler의 표/개체 이동 경로에 연결한다.
2. 아직 구현하지 않을 경우 대화상자에서 해당 옵션을 disabled 처리하고 보고서에 한계로 명시한다.
```

### 4.2 후속 개선: `글 뒤` 격자 위치가 일반 canvas 경로에서 실제 글 뒤가 아님

격자 overlay는 canvas sibling DOM으로 추가된다.

```text
rhwp-studio/src/view/grid-overlay.ts:20
  behindText -> z-index: 1
  inFrontOfText -> z-index: 4

rhwp-studio/src/styles/editor.css:49
  canvas -> z-index: 0
```

일반 canvas 렌더링에서는 본문 텍스트와 페이지 배경이 같은 canvas에 이미 그려진다.
따라서 `behindText`라도 overlay가 canvas 위에 놓이면 실제로는 글 위에 표시된다.

영향:

```text
격자 위치 "글 뒤" 옵션이 한컴 UI 의미와 다르게 동작할 수 있다.
본문 글자가 많은 페이지에서 격자점/선이 글자 위로 겹쳐 보일 수 있다.
```

후속 권장:

```text
1. 글 뒤 격자는 render layer 안에서 page background와 flow text 사이에 그리도록 구현한다.
2. 또는 현재 DOM overlay 방식에서는 "글 앞" 표시만 지원하고, "글 뒤" 옵션은 disabled 처리한다.
```

### 4.3 `git diff --check` 실패

문서 2개에 EOF 빈 줄이 남아 있다.

```text
mydocs/working/task_m100_1129_stage1.md
mydocs/working/task_m100_1129_stage3.md
```

권장:

```text
수용 전 두 문서의 EOF 빈 줄을 제거한다.
```

## 5. 타당한 부분

다음 방향은 수용 가능하다.

```text
1. HWP5 SECTION_DEFINE의 line_grid/char_grid를 버리지 않고 SectionDef에 보존
2. HWPX hp:grid lineGrid/charGrid 파싱
3. HWPX paraPr@snapToGrid 기본값 true 및 bit 8 보존
4. HWP3 원본과 HWP5/HWPX 변환본의 쪽 테두리 기준을 분리
5. getPageBorderFill/setPageBorderFill WASM API 추가
6. rhwp-studio에서 쪽 테두리/배경 설정 UI를 열 수 있게 한 방향
```

## 6. 권장 처리

수용 후보:

```text
1. PR 방향은 수용한다.
2. 4.3의 문서 EOF 빈 줄은 수용 과정에서 정리한다.
3. 4.1, 4.2는 후속 작업으로 남긴다.
4. wasm 빌드와 작업지시자 시각 판정을 게이트로 둔다.
```

검증 게이트:

```text
cargo fmt --all -- --check
git diff --check
cargo check --target wasm32-unknown-unknown --lib
cargo test --lib
cd rhwp-studio && npm run build
docker compose --env-file .env.docker run --rm wasm
```

시각 판정 대상:

```text
samples/hwp3-sample16-hwp5.hwp
samples/hwp3-sample16-hwp5.hwpx
samples/종이기준.hwp
samples/종이기준.hwpx
samples/쪽기준.hwp
samples/쪽기준.hwpx
```

## 7. PR 코멘트 초안

```text
jangster77님, 큰 규모의 PR 감사합니다.

격자 보기 활성화, HWP5/HWPX grid/snapToGrid 보존, 쪽/종이 기준 분리 방향은 이슈 #1129의
요구와 잘 맞습니다. CI와 로컬 기본 빌드도 확인했습니다. 현재 PR의 기능 방향은 수용하는
방향으로 보겠습니다.

검토 중 확인한 후속 개선 항목도 있습니다. 격자 설정 대화상자의 "자석 효과",
"격자에만 붙이기"는 현재 snapMode 상태로는 보존되지만 실제 표/개체 이동 경로와는 아직
연결되지 않습니다. 또한 DOM overlay 방식에서는 "글 뒤" 격자가 일반 canvas 렌더링에서
실제로는 canvas 위에 표시될 수 있어 한컴 UI 의미와 다르게 보일 가능성이 있습니다.

이 두 항목은 이번 PR 수용을 막는 blocker로 두기보다, 기여자가 이어서 보강하거나
maintainer가 후속 패치로 구현하는 방식이 좋겠습니다. 수용 과정에서는 문서 2개의
`git diff --check` EOF 빈 줄만 같이 정리하겠습니다.
```
