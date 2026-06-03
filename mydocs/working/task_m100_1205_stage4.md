# Task M100-1205 Stage 4 완료 보고서 — 실제 샘플 검증과 최종 정리

## 1. 작업 범위

구현계획서 Stage 4에 따라 자동 검증을 완료하고, 재현 샘플 SVG export 및 `rhwp-studio` 실행 상태를 확인했다.

브랜치:

```text
issue-1205-para-border-none-side
```

## 2. 자동 검증

실행 명령:

```text
cargo fmt --all --check
cargo test --lib task_1205 -- --nocapture
cargo test --lib test_469_partial_start_box_does_not_cross_col_top -- --nocapture
cargo test --lib test_471_cross_column_box_no_bottom_line_in_col0 -- --nocapture
cargo test --tests
```

결과:

```text
cargo fmt --all --check ... ok
task_1205_para_border_none_sides_do_not_render_vertical_edges ... ok
task_1205_rect_stroke_path_requires_four_visible_same_stroke ... ok
test_469_partial_start_box_does_not_cross_col_top ... ok
test_471_cross_column_box_no_bottom_line_in_col0 ... ok
cargo test --tests ... ok
```

`cargo test --tests` 최초 실행에서 SVG snapshot 4건이 실패했다. 차이는 `stroke` 없는 `fill="none"` 문단 박스 노드가 사라진 구조 차이였고, 실제 표시되는 선 변경은 #1205 의도 범위였다. 이에 다음 골든 SVG를 갱신한 뒤 전체 테스트를 재실행했다.

- `tests/golden_svg/issue-147/aift-page3.svg`
- `tests/golden_svg/issue-157/page-1.svg`
- `tests/golden_svg/issue-617/exam-kor-page5.svg`
- `tests/golden_svg/issue-677/bokhakwonseo-page1.svg`

## 3. 실제 샘플 export

실행 명령:

```text
cargo run --bin rhwp -- export-svg "/Users/melee/Downloads/[2027] 온새미로 1 본교재.hwpx" -o /private/tmp/task1205_para_border -p 9
```

결과:

```text
문서 로드 완료: 47페이지
→ /private/tmp/task1205_para_border/[2027] 온새미로 1 본교재_010.svg
내보내기 완료: 1개 SVG 파일
```

출력 파일은 PR 대상이 아닌 `/private/tmp/task1205_para_border/`에 보관했다.

## 4. rhwp-studio 실행

`rhwp-studio`에서 현재 Rust 변경을 사용하도록 WASM 패키지를 빌드했다.

```text
/Users/melee/.cargo/bin/wasm-pack build --target web
```

결과:

```text
pkg 생성 완료: /private/tmp/rhwp-issue-1205/pkg
```

이후 Vite dev server를 실행했다.

```text
npm run dev -- --host 127.0.0.1 --port 7700
```

7700 포트가 이미 사용 중이어서 Vite가 자동으로 7701 포트를 선택했다.

```text
Local: http://127.0.0.1:7701/
```

권한 승격 curl 확인:

```text
HTTP/1.1 200 OK
Content-Type: text/html
```

## 5. 판단

Stage 4 기준 자동 테스트와 실제 샘플 export가 통과했다.

문단 borderFill의 `NONE` side는 더 이상 수직 border로 강제 렌더링되지 않고, 기존 4면 동일 visible stroke 문단 border는 `RectangleNode` stroke 경로를 유지한다. partial/cross-column 회귀 테스트도 통과했다.
