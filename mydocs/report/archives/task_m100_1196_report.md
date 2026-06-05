# 최종 보고 — Task M100-1196

## 이슈

- GitHub: [#1196](https://github.com/edwardkim/rhwp/issues/1196)
- 제목: HWPX `gutterType="LEFT_RIGHT"` 맞쪽 편집 여백 교대 미적용

## 원인

대상 HWPX 샘플 `samples/hwpx/[2027] 온새미로 1 본교재.hwpx` 는 `pagePr`의
`gutterType="LEFT_RIGHT"`를 사용한다.

최신 upstream 기준 파서는 이미 이 값을 `PageDef.binding = BindingMethod::DuplexSided`로
materialize하고 있었다. 문제는 layout 계산 경로가 최종 쪽번호 홀짝을 받지 않아,
짝수쪽에서도 `margin_left + margin_gutter`가 계속 왼쪽에 적용된다는 점이었다.

#1276 이후 page 4는 section 1 본문 시작이자 최종 `page_num=4`로 정리되었지만,
본문 영역 x 좌표는 page 4/5/6 모두 같은 값으로 남아 PDF 기준 맞쪽 편집 방향과 어긋났다.

## 변경

- `src/model/page.rs`
  - `PageAreas::from_page_def_for_page(page_def, page_number)`를 추가했다.
  - 기존 `from_page_def()`는 page 1 기준 계산으로 위임해 호환성을 유지했다.
  - `BindingMethod::DuplexSided`에서 홀수쪽은 기존 방향, 짝수쪽은 좌우 여백을 교대하도록 했다.
  - `SingleSided`와 `TopFlip`은 기존 좌우 여백 정책을 유지했다.

- `src/renderer/page_layout.rs`
  - `PageLayoutInfo::from_page_def_for_page(...)`를 추가했다.
  - `PageLayoutInfo::apply_page_number_margins(page_def, page_number)`를 추가했다.
  - 기존 layout의 column 구성은 보존하고, 최종 page number 기준 body/header/footer/footnote/column x 좌표만 이동하도록 했다.

- `src/document_core/queries/rendering.rs`
  - `apply_page_number_layouts_for_section()` helper를 추가했다.
  - 구역 간 page number carry 보정 이후, 바탕쪽 선택 전에 최종 `page.page_number` 기준 layout 보정을 적용했다.
  - TypesetEngine과 fallback Paginator 모두 `DocumentCore::paginate()` 결과 경로에서 같은 보정을 받는다.

- `tests/issue_1196_hwpx_gutter_left_right.rs`
  - 대상 HWPX 샘플에서 page 4/5/6의 맞쪽 편집 여백 교대를 고정하는 통합 회귀 테스트를 추가했다.
  - page 4가 `section=1, page_num=4` 본문 시작이라는 #1276 전제도 함께 검증한다.

## 주요 확인

`dump-pages` 기준:

```text
page 4: body_area: x=189.0 y=113.4 w=510.2 h=895.8
page 5: body_area: x=94.5  y=113.4 w=510.2 h=895.8
page 6: body_area: x=189.0 y=113.4 w=510.2 h=895.8
```

판정:

- 최종 쪽번호 기준 짝수쪽 page 4/6의 body x가 홀수쪽 page 5보다 오른쪽으로 교대된다.
- body width는 모두 `510.2`로 유지된다.
- page 4는 `section=1, page_num=4`이고 `"강의 01."` 본문 시작을 유지한다.

## 시각 검증

SVG/debug overlay 산출물:

```text
output/poc/task1196/[2027] 온새미로 1 본교재_004.svg
output/poc/task1196/[2027] 온새미로 1 본교재_005.svg
output/poc/task1196/[2027] 온새미로 1 본교재_006.svg
output/poc/task1196/index.html
```

SVG 좌표 확인:

```text
page 4 body-clip x=188.973
page 5 body-clip x=94.480
page 6 body-clip x=188.973
```

WASM/rhwp-studio 검증:

- `wasm-pack build --target web`로 `pkg/`를 최신 소스 기준으로 갱신했다.
- `rhwp-studio` 서버를 `http://127.0.0.1:7700/`에서 실행했다.
- 작업지시자가 rhwp-studio에서 대상 문서를 직접 로드해 수정 적용을 확인했다.

참고:

- Docker daemon이 실행 중이 아니어서 Docker 기반 WASM 빌드는 수행하지 못했다.
- 로컬 `wasm-pack 0.15.0`과 `wasm32-unknown-unknown` target으로 WASM 빌드를 완료했다.
- `pkg/`와 `output/`은 `.gitignore` 대상이므로 PR에는 포함하지 않는다.

## 검증

기여자 체크리스트 기준 통과:

```text
cargo fmt --all -- --check
cargo test
cargo clippy -- -D warnings
```

결과:

```text
cargo fmt --all -- --check
통과

cargo test
종료 코드 0. lib/unit/integration/doc-test 통과

cargo clippy -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

추가로 Stage 5에서 먼저 확인한 명령:

```text
cargo test --lib
test result: ok. 1565 passed; 0 failed; 6 ignored

cargo test --tests
종료 코드 0. 전체 integration test 통과
```

주요 회귀:

```text
tests/issue_1196_hwpx_gutter_left_right.rs
test result: ok. 1 passed; 0 failed

tests/issue_1271_hwpx_behind_text_table.rs
test result: ok. 3 passed; 0 failed
```

## PR 판단

PR 생성 가능 상태다.

- PR 대상 브랜치: `upstream/devel`
- push 대상: fork `origin`
- `pr/` 폴더 문서는 작성하지 않는다.
- `pkg/`, `output/` 산출물은 포함하지 않는다.
- 기존 #1142~#1144 관련 untracked 문서는 이번 PR 범위에서 제외한다.

## 남은 작업

- 작업지시자 승인 후 #1196 관련 파일만 선별 stage/commit 한다.
- fork `origin`에 브랜치를 push하고 `devel` 대상으로 PR을 생성한다.
