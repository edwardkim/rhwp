# task_m100_1023 완료 보고서

## 1. 이슈

`samples/table-vpos-01.hwp` 5페이지의 표 안에서 10, 11, 12 등 두 자리 글자겹침 번호가
사각형 내부에 합쳐져 출력되지 않고, 첫 글자만 사각형 안에 남고 나머지 숫자가 오른쪽으로 밀려 출력되는 문제를 수정했다.

추가 요구:

```text
글자겹침으로 처리된 두 글자 합침은 사용자가 한 글자로 캐럿 이동할 수 있어야 한다.
text flow 에서도 한 글자 폭으로 계산되어야 한다.
```

## 2. 재현 파일

```text
samples/table-vpos-01.hwp
```

디버그 SVG:

```text
output/poc/task1023_table_vpos_debug/table-vpos-01_005.svg
```

수정 후 SVG:

```text
output/poc/task1023_char_overlap_fixed/table-vpos-01_005.svg
```

## 3. 원인

문제 셀의 10/11/12 마커는 일반 텍스트가 아니라 `CharOverlap` 컨트롤이다.

해당 컨트롤 payload는 두 개의 HWP PUA 구성 글자를 가진다.

```text
10: U+F02BA + U+F02C3
11: U+F02BA + U+F02C4
12: U+F02BA + U+F02C5
```

기존 구현은 디코딩 가능한 PUA 글자겹침만 한 글자 폭으로 처리했다.
이번 샘플의 PUA 조합은 숫자 문자열로 디코딩되지 않아 다음 문제가 동시에 발생했다.

```text
1. 렌더러가 두 구성 글자를 나란히 배치했다.
2. text flow 폭이 두 글자 폭으로 계산되었다.
3. 중첩 표 셀 커서 경로(getCursorRectByPath)가 여전히 두 글자 기준으로 cursor x를 계산했다.
```

## 4. 수정 내용

`CharOverlap`은 payload 글자 수와 무관하게 하나의 편집 단위로 보도록 공통 helper를 추가했다.

```text
renderer::composer::char_overlap_advance_units()
```

적용 범위:

```text
1. paragraph layout text flow 폭 계산
2. table required padding 폭 계산
3. SVG 렌더링
4. Web Canvas 렌더링
5. Skia text replay
6. 본문/셀/경로 기반 cursor rect 계산
```

렌더링은 디코딩되지 않는 다중 PUA CharOverlap의 구성 글자를 같은 중심 좌표에 겹쳐 그리도록 변경했다.

경로 기반 중첩 표 커서 계산도 `effective_char_count()`를 사용하도록 수정했다.
이로써 10번 사각형 앞 offset 0에서 offset 1로 이동할 때 커서가 사각형 중앙이 아니라 사각형 뒤쪽 경계로 이동한다.

## 5. 회귀 테스트

`tests/issue_table_vpos_01_page5_cell_hit_test.rs`에 다음 테스트를 추가했다.

```text
page5_inner_11x3_char_overlap_marker_advances_one_box
```

검증 내용:

```text
중첩 표 셀의 10번 CharOverlap 마커에서
offset 0 -> offset 1 커서 이동이 한 글자 박스 전체 폭으로 계산되는지 확인
```

## 6. 검증

명령:

```text
cargo fmt --all -- --check
cargo check
cargo test --test issue_table_vpos_01_page5_cell_hit_test
cargo test --test issue_1071_tac_cursor_nav
docker compose --env-file .env.docker run --rm wasm
```

결과:

```text
success
```

WASM 산출물은 `pkg/` 생성 후 `rhwp-studio/public/`에 동기화했다.

## 7. 메인테이너 판정

```text
SVG 시각 판정: 통과
rhwp-studio 웹 동작 테스트: 통과
```

## 8. 결론

`CharOverlap`의 다중 PUA 구성 글자 케이스를 렌더링, text flow, cursor rect 모두에서
하나의 편집 단위로 처리하도록 정리했다.

이번 수정으로 `table-vpos-01.hwp` 5페이지의 10/11/12 사각형 번호 출력과 커서 이동이 정상화되었다.
