# Task #1510 Stage 5 — CI svg_snapshot #157 회귀 보정

## 1. 출발점

PR #1518 최신 head `a2e5baff` 기준 GitHub Actions에서 `Build & Test`가 실패했다.

실패 체크:

- Workflow: `CI`
- Job: `Build & Test`
- 실패 단계: `Run tests`
- 실패 테스트: `cargo test --test svg_snapshot` 중 `issue_157_page_1`

다른 체크는 통과했다.

- CodeQL JavaScript/TypeScript: 통과
- CodeQL Python: 통과
- CodeQL Rust: 통과
- Canvas visual diff: 통과
- WASM Build: skipped

## 2. 실패 내용

CI 로그:

```text
SVG snapshot mismatch for issue-157/page-1.
expected: tests/golden_svg/issue-157/page-1.svg
actual:   tests/golden_svg/issue-157/page-1.actual.svg
```

로컬 재현:

```bash
cargo test --test svg_snapshot issue_157_page_1 -- --nocapture
```

동일하게 실패했다. expected/actual SVG를 PNG로 변환해 비교한 결과, 단순 golden stale이 아니라
실제 시각 회귀였다.

- 첫 번째 TopAndBottom 표가 기존보다 약 25.6px 위로 이동
- 표 뒤 큰 빈 공간 발생
- 하단 위임인 표가 본문과 겹침
- 마지막 문단들이 본문 하단을 넘는 `LAYOUT_OVERFLOW` 로그 출력

## 3. 원인 가설

#1510 보정은 "실제 제목 텍스트가 있는 visible host 문단"에 함께 앵커된 para-relative
`TopAndBottom` float 표를 대상으로 한다.

하지만 현재 `para_has_visible_text()`는 공백 문자도 visible text로 판정한다.
`samples/hwpx/issue_157.hwpx`의 문제 표 host 문단 `pi=7`은 텍스트가 공백 위주인데,
이 문단이 #1510 전용 visible-float exclusion/stacking 경로에 들어가면서 기존 #157 배치가 깨졌다.

반면 #1510 샘플의 host 문단은 `ISSUE 1510 CENTER TITLE`이라는 실제 비공백 제목 텍스트가 있으므로
visible-float 경로를 유지해야 한다.

## 4. 수정 방향

- 공용 `para_has_visible_text()` 의미는 유지한다.
  - 기존 여러 레이아웃 가드가 공백/PUA/문단 상태를 이 함수에 의존할 수 있기 때문이다.
- #1510 전용 visible-float 판정만 `para_has_non_whitespace_text()` 기준으로 좁힌다.
  - 공백-only host 문단은 기존 TopAndBottom 흐름을 유지한다.
  - 실제 비공백 텍스트가 있는 host 문단만 #1510의 control 순서/active exclusion 경로를 탄다.

## 5. 검증 계획

- `cargo test --test svg_snapshot issue_157_page_1 -- --nocapture`
- `cargo test --test svg_snapshot -- --nocapture`
- `cargo test --test issue_1510 -- --nocapture`
- `cargo test --test issue_986 -- --nocapture`
- `cargo test --test issue_712 -- --nocapture`
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `wasm-pack build --target web --out-dir pkg`
- `git diff --check`

## 6. 추가 시각 피드백

작업지시자가 HWPX를 한컴 2024에서 열었을 때의 화면과 rhwp PDF/WASM 렌더가 아직 다르다고 지적했다.
재비교 결과, #157 회귀 보정만으로는 #1510 표 테두리 위치가 완전히 맞지 않았다.

수정 전 PNG 선 검출 결과:

- HWPX 한컴 2024 PDF: `204, 264, 324, 335, 387, 543, 599, 655`
- HWPX rhwp PDF: `198, 264, 329, 387, 538, 600, 661`
- HWP 한컴 2024 PDF: `164, 204, 224, 255, 283, 543, 599, 655`
- HWP rhwp PDF: `158, 198, 224, 256, 289, 538, 600, 661`

원인은 세 가지였다.

- visible host float 표의 렌더 높이가 `common.height`보다 커졌다. 셀 문단 line spacing까지
  행 높이에 반영되어 A/B 표가 한컴보다 약 7.3px(96dpi 기준) 높게 나왔다.
- Para-relative visible float 표에서 `outer_margin_top=1mm`가 y 기준에 반영되지 않아 표가 약 3.8px 위로 올라갔다.
- HWPX에서 B처럼 non-positive offset인 visible float 뒤에 C 같은 non-positive sibling이 이어질 때,
  한컴은 host line spacing 600HU(8px)를 B/C 사이 간격으로 보존했다.

## 7. 구현 내용

- `fit_measured_table_to_declared_height()`를 추가해 measured table을 declared `common.height` 기준으로
  복제 보정할 수 있게 했다.
- #1510 visible host 조건은 `para_has_non_whitespace_text()`로 좁혀 공백-only host 문단의 #157/#986 기존 흐름을 유지했다.
- `typeset`과 `layout` 양쪽에서 visible para float 표에만 fitted measured table을 사용했다.
- visible para float y 배치에서 `outer_margin_top`을 para 기준 y에 반영했다.
- HWPX source이고 뒤에 non-positive visible float sibling이 남은 경우에만, 현재 표 하단 뒤에 host line spacing을
  inter-float gap으로 예약했다.

## 8. 시각 검증 결과

수정 후 PDF를 다시 내보낸 뒤 `pdftoppm -png -r 144`로 PNG를 만들고, 검은 수평선 run을 검출했다.

- HWPX 한컴 2024 PDF: `204, 264, 324, 335, 387, 543, 599, 655`
- HWPX rhwp PDF: `204, 264, 324, 336, 388, 544, 600, 656`
- HWP 한컴 2024 PDF: `164, 204, 224, 255, 283, 543, 599, 655`
- HWP rhwp PDF: `164, 204, 224, 256, 284, 544, 600, 656`

테두리 위치 기준으로 HWP/HWPX 모두 0~1px 차이까지 들어왔다.

render tree 주요 좌표:

- HWPX B: `y=136.0, h=80.0`, C: `y=224.0, h=34.7`, A: `y=362.7, h=74.7`
- HWP B: `y=109.4, h=80.0`, C: `y=136.0, h=34.7`, A: `y=362.7, h=74.7`

## 9. 검증 결과

- `cargo test --test issue_1510 -- --nocapture`: 통과
- `cargo test --test svg_snapshot issue_157_page_1 -- --nocapture`: 통과
- `cargo test --test svg_snapshot -- --nocapture`: 통과
- `cargo test --test issue_712 -- --nocapture`: 통과
- `cargo test --test issue_986 -- --nocapture`: 통과
- `cargo fmt --check`: 통과
- `cargo clippy --all-targets -- -D warnings`: 통과
- `wasm-pack build --target web --out-dir pkg`: 통과
- `git diff --check`: 통과

`svg_snapshot`, `issue_712`에서는 기존과 같은 `LAYOUT_OVERFLOW` 진단 로그가 출력되지만 assertion은 모두 통과했다.
