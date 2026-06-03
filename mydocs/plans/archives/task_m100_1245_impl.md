# Task M100 #1245 구현 계획 — 7쪽 하단 그림 배치 보정

## 1. 목표

`3-09월_교육_통합_2022.hwp` 7쪽 하단 `문25)`/`문26)` 주변의 그림/도형 객체와 문단 흐름을 한컴오피스 기준에 맞춘다.

## 2. 초기 가설

현재 증상은 다음 중 하나일 가능성이 높다.

1. 하단 그림/도형이 현재 단의 가로 영역과 교차하지 않는데도 문단 흐름을 밀거나, 반대로 교차하는데 충분히 반영하지 않는 경우
2. `Square`/`TopAndBottom` 객체의 `vertical_pos` 되감김 또는 하단 overflow 판정이 문단 경계 근처에서 과하거나 부족한 경우
3. `LINE_SEG.segment_width`가 좁은 wrap 줄을 표현하지만, 페이지 항목 배치에서 그림 높이와 문단 높이의 결합 방식이 한컴과 다른 경우
4. 그림/도형 컨트롤의 anchor paragraph와 실제 영향 paragraph가 분리되어 있는데 현재 로직이 anchor paragraph만 기준으로 판단하는 경우

## 3. 분석 명령

```bash
cargo run -- dump-pages samples/3-09월_교육_통합_2022.hwp -p 6
cargo run -- dump samples/3-09월_교육_통합_2022.hwp -s 0 -p <문제 문단>
cargo run -- export-svg samples/3-09월_교육_통합_2022.hwp -p 6 -o output/task1245_stage0 --show-control-codes --debug-overlay
rsvg-convert output/task1245_stage0/page_007.svg -o output/task1245_stage0/page_007.png
```

PDF 비교는 `pdf/3-09월_교육_통합_2022.pdf`의 7쪽을 사용한다.

## 4. 수정 후보

분석 결과에 따라 다음 중 하나를 선택한다.

- `src/renderer/layout.rs`: 페이지 항목 배치, 그림/도형 흐름 영향, 하단 overflow 판정
- `src/renderer/layout/paragraph_layout.rs`: 문단 내부 line segment와 wrap 줄 재배치
- `src/renderer/height_cursor.rs`: `LINE_SEG.vertical_pos` 기반 위치 되감김/보정
- `src/renderer/typeset.rs`: 그림/도형 선택 rect 또는 render tree 좌표가 배치 결과와 불일치할 때만 확인

## 5. 회귀 가드

우선 `tests/issue_1139_inline_picture_duplicate.rs`에 다음 검증을 추가한다.

- `3-09월_교육_통합_2022.hwp` 7쪽 page count 유지
- 7쪽 하단 `문25)`/`문26)` 관련 문단이 같은 페이지에 유지되는지
- 문제 그림/도형의 page item 위치와 문단 흐름 영향이 한컴/PDF 기준 범위에 들어오는지

## 6. 검증 계획

수정 직후:

```bash
cargo test --test issue_1139_inline_picture_duplicate -- --nocapture
```

PR 직전:

```bash
cargo test --tests
wasm-pack build --target web --out-dir pkg
```

필요 시 Studio 시각 확인은 작업지시자의 한컴 비교 판단을 받는다.

## 7. 승인 요청 범위

Stage0 분석 후, 위 수정 후보 중 실제로 건드릴 파일과 회귀 가드 내용을 확정해 작업지시자 승인 후 소스 수정한다.
