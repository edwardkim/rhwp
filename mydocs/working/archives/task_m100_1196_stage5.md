# Stage 5 보고 — Task M100-1196

## 범위

- 이슈: [#1196](https://github.com/edwardkim/rhwp/issues/1196)
- 단계: SVG/debug overlay export, 좌표 수동 검증, 전체 회귀 검증
- 브랜치: `local/task1196`

## 산출물

SVG/debug overlay export:

```text
output/poc/task1196/[2027] 온새미로 1 본교재_004.svg
output/poc/task1196/[2027] 온새미로 1 본교재_005.svg
output/poc/task1196/[2027] 온새미로 1 본교재_006.svg
```

비교용 HTML:

```text
output/poc/task1196/index.html
```

`output/`은 `.gitignore` 대상이므로 PR에는 포함되지 않는다.

## 실행 명령

```text
cargo run --quiet --bin rhwp -- export-svg "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -o output/poc/task1196 --debug-overlay -p 3
cargo run --quiet --bin rhwp -- export-svg "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -o output/poc/task1196 --debug-overlay -p 4
cargo run --quiet --bin rhwp -- export-svg "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -o output/poc/task1196 --debug-overlay -p 5
```

결과:

```text
문서 로드 완료: samples/hwpx/[2027] 온새미로 1 본교재.hwpx (46페이지)
→ output/poc/task1196/[2027] 온새미로 1 본교재_004.svg
→ output/poc/task1196/[2027] 온새미로 1 본교재_005.svg
→ output/poc/task1196/[2027] 온새미로 1 본교재_006.svg
```

## SVG 좌표 확인

SVG에서 `body-clip`과 첫 debug overlay 좌표를 추출했다.

```text
[2027] 온새미로 1 본교재_004.svg
  body-clip x=188.973 w=510.240
  first-overlay x=188.973 w=510.240 label=s1:pi=0 y=113.4
  footer-first-text x=60.453 text=4
  first-background-cell x=37.787 w=5.960

[2027] 온새미로 1 본교재_005.svg
  body-clip x=94.480 w=515.067
  first-overlay x=105.813 w=487.573 label=s1:pi=9 y=113.4
  footer-first-text x=642.240 text=독
  first-background-cell x=0.053 w=560.933

[2027] 온새미로 1 본교재_006.svg
  body-clip x=188.973 w=516.067
  first-overlay x=188.973 w=510.240 label=s1:pi=19 y=113.4
  footer-first-text x=60.453 text=6
  first-background-cell x=37.787 w=5.960
```

판정:

- page 4 `body-clip.x=188.973`은 page 5 `body-clip.x=94.480`보다 오른쪽이다.
- page 6 `body-clip.x=188.973`도 page 5보다 오른쪽이다.
- debug overlay의 첫 본문 경계도 page 4/6은 `x=188.973`, page 5는 `x=105.813`로 같은 방향성을 보인다.
- page 4/6의 쪽번호 첫 글자는 `x=60.453` 쪽에 있고, page 5의 하단 바탕쪽 텍스트는 오른쪽 영역에서 시작한다. #1276의 홀짝 바탕쪽 방향이 유지된다.
- page 4/6의 첫 배경성 cell clip은 `x=37.787`로 동일하다. body 여백 교대 보정이 같은 parity의 paper 기준 배경성 개체를 불필요하게 이동시키지 않는다.

SVG `body-clip` width는 페이지별 실제 clip/content 구성 차이가 섞이므로, width 보존 여부는 `dump-pages`의 `body_area` 값을 기준으로 함께 확인했다.

## dump-pages 교차 확인

```text
page 4: body_area: x=189.0 y=113.4 w=510.2 h=895.8
page 5: body_area: x=94.5  y=113.4 w=510.2 h=895.8
page 6: body_area: x=189.0 y=113.4 w=510.2 h=895.8
```

판정:

- 최종 `page_num` 기준으로 짝수쪽 page 4/6의 body x가 홀수쪽 page 5보다 오른쪽으로 교대된다.
- body width는 dump 기준 모두 `510.2`로 유지된다.

## 회귀 검증

명령:

```text
cargo fmt --check
cargo test --lib
cargo test --tests
```

결과:

```text
cargo fmt --check
통과

cargo test --lib
test result: ok. 1565 passed; 0 failed; 6 ignored

cargo test --tests
종료 코드 0. 전체 integration test 통과
```

`cargo test --tests` 실행 중 #1196 전용 테스트와 #1271 회귀 테스트도 통과했다.

```text
tests/issue_1196_hwpx_gutter_left_right.rs
test result: ok. 1 passed; 0 failed

tests/issue_1271_hwpx_behind_text_table.rs
test result: ok. 3 passed; 0 failed
```

## 직접 시각 검증 시점

작업지시자의 직접 시각 검증은 이 단계 직후 진행하면 된다.

권장 확인 파일:

```text
output/poc/task1196/index.html
```

확인 포인트:

- page 4와 page 6의 debug overlay 본문 시작선이 page 5보다 오른쪽에 있는지 확인한다.
- page 4의 `"강의 01."` 시작 위치가 오른쪽 여백 교대 방향에 맞는지 확인한다.
- page 4/6의 하단 쪽번호와 바탕쪽 텍스트가 #1276 후속 상태를 유지하는지 확인한다.
- page 4/6의 상단 배경성 회색 표/텍스트가 body 여백 보정 때문에 좌우로 흔들리지 않는지 확인한다.

## Stage 5 결론

- SVG/debug overlay 산출물에서 page 4/6과 page 5의 본문 시작 x 방향성이 기대와 맞다.
- dump-pages 기준 body width는 유지된다.
- 전체 lib/integration 회귀 테스트가 통과했다.
- Stage 6에서는 최종 보고서를 작성하고 오늘 할일 상태를 완료로 갱신한다.
