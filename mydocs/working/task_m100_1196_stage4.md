# Stage 4 보고 — Task M100-1196

## 범위

- 이슈: [#1196](https://github.com/edwardkim/rhwp/issues/1196)
- 단계: #1196 전용 통합 회귀 테스트 추가
- 브랜치: `local/task1196`

## 변경 파일

```text
tests/issue_1196_hwpx_gutter_left_right.rs
```

## 구현 내용

대상 샘플 문서:

```text
samples/hwpx/[2027] 온새미로 1 본교재.hwpx
```

추가 테스트:

```text
onsaemiro_left_right_gutter_alternates_body_area_by_page_parity
```

검증 항목:

- page 4가 `section=1, page_num=4`를 유지한다.
- page 4가 `"강의 01."` 본문 시작 위치를 유지한다.
- page 5/6이 각각 최종 `page_num=5`, `page_num=6`을 유지한다.
- `gutterType="LEFT_RIGHT"` 맞쪽 편집에 따라 짝수쪽 page 4/6의 `body_area.x`가 홀수쪽 page 5보다 오른쪽에 있다.
- page 4와 page 6의 `body_area.x`가 동일 계열이다.
- 좌우 여백 교대는 body width를 변경하지 않는다.

테스트는 `wasm_api::HwpDocument::dump_page_items()` 출력에서 `body_area` 라인을 파싱해 x 좌표와 width를 직접 비교한다. 숫자 비교는 dump 출력의 소수점 반올림을 고려해 0.1pt 허용 오차를 둔다.

## 검증

명령:

```text
cargo fmt --check
cargo test --test issue_1196_hwpx_gutter_left_right -- --nocapture
cargo test --test issue_1271_hwpx_behind_text_table -- --nocapture
```

결과:

```text
cargo fmt --check
통과

cargo test --test issue_1196_hwpx_gutter_left_right -- --nocapture
test result: ok. 1 passed; 0 failed

cargo test --test issue_1271_hwpx_behind_text_table -- --nocapture
test result: ok. 3 passed; 0 failed
```

## Stage 4 결론

- #1196 재현 샘플을 직접 사용하는 통합 회귀 테스트를 추가했다.
- 테스트는 #1271의 page 4 본문 시작 보정과 #1196의 좌우 여백 교대를 함께 고정한다.
- 기존 #1271 회귀 테스트도 계속 통과한다.
- Stage 5에서는 SVG/debug overlay export로 시각 검증 산출물을 만들고, 작업지시자가 직접 확인할 수 있는 시점을 정한다.
