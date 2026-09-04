# Task #1585 Stage 3 완료보고서 — 구현 및 검증

## 범위

- #1551 merge 상태 확인
- 최신 `upstream/devel` 기준 rebase
- 캡션 내부 `TopAndBottom` 그림 렌더링 구현
- depth 1 중첩 표 caption의 플로팅 이미지 방출 보강
- 신규 회귀 테스트와 기존 관련 회귀 테스트 실행
- #1270 첨부 실샘플 SVG 보조 검증

## 기준 정렬

#1551은 Stage 3 시작 시점에 merge 완료 상태였다.

```text
state=MERGED
mergedAt=2026-06-27T04:12:17Z
mergeCommit=a94e20518b486caabd82de824784bdf25a0e1c87
base=devel
```

이후 `upstream/devel`을 fetch하고 `local/task1585`를 rebase했다.

```text
upstream/devel HEAD: a94e2051 Merge pull request #1551 from postmelee/local/task1270
작업 브랜치: local/task1585...upstream/devel
```

따라서 #1585 변경은 #1551의 인라인 caption image 스레딩 변경을 중복 포함하지 않고, merge된 기준 위에 얹힌다.

## 구현 요약

### `src/renderer/layout/picture_footnote.rs`

`layout_caption()`에서 caption paragraph 조판 직후 caption 전용 helper를 호출하도록 했다.

핵심 처리:

- caption paragraph의 `Control::Picture` 중 `text_wrap == TextWrap::TopAndBottom`인 그림을 탐지한다.
- #1551 인라인 경로에서 이미 같은 control 위치가 등록된 경우 중복 방출하지 않는다.
- caption content box와 paragraph line segment를 기준으로 배치 좌표를 계산한다.
- 기존 `layout_picture()` 경로를 재사용해 `ImageNode`와 payload 처리를 유지한다.
- HWPX 샘플처럼 `treat_as_char=true`가 함께 있어도, `TopAndBottom` caption floating 배치용 clone은 non-TAC로 정규화해 렌더링한다.

### `src/renderer/layout/table_layout.rs`

#1270 첨부 실샘플의 `image2`는 top-level 표가 아니라 내부 표의 TOP caption 안에 있었다. 기존 구현은 표 caption 렌더링을 `depth == 0`으로 제한해, helper를 추가해도 해당 caption 자체가 렌더링되지 않았다.

이번 변경은 범위를 좁혀 다음 경우만 추가 허용했다.

- `depth == 0`: 기존처럼 caption 렌더링
- `depth == 1`이고 caption 안에 `TopAndBottom` picture가 있는 경우: caption 렌더링

중첩 caption은 기존 표 caption 센티널인 `cell_index=65534`를 유지하도록 `CellContext` 마지막 path entry를 caption context로 치환한다.

## 신규 테스트

추가 파일:

```text
tests/issue_1585_caption_floating_image.rs
```

테스트 케이스:

1. top-level table caption 내부 `TopAndBottom` picture가 payload 포함 `ImageNode`로 1회 방출되는지 확인
2. depth 1 nested table caption 내부 `TopAndBottom` picture가 caption context와 함께 1회 방출되는지 확인

## 실샘플 보조 검증

대상:

```text
/private/tmp/issue1270/서울 문화예술단체 지원사업.hwpx
```

#1551 merge 직후, #1585 변경 전 page 0 SVG의 `<image>` 개수는 1개였다.

```text
/private/tmp/rhwp-task1585-after1551-svg
<image> count: 1
```

#1585 변경 후 page 0 SVG의 `<image>` 개수는 2개로 증가했다.

```text
/private/tmp/rhwp-task1585-stage3-svg
<image> count: 2
```

추가 방출된 image는 `image2` / `SEOUL MY SOUL` 로고에 해당한다.

```xml
<image x="499.47333333333336"
       y="238.9333333333333"
       width="214.66666666666666"
       height="35.266666666666666"
       .../>
```

기존 `image1`도 유지된다.

```xml
<image x="69.90666666666667"
       y="109.79999999999998"
       width="200"
       height="118.4"
       .../>
```

## 검증 결과

다음 검증을 통과했다.

```text
cargo fmt --check
cargo test --test issue_1585_caption_floating_image
cargo test --test issue_1270_caption_inline_image
cargo test --test issue_530
cargo test --test issue_1459_topbottom_picture_reflow
cargo test --test issue_1139_inline_picture_duplicate
cargo test --test issue_1352_table_cell_tac_picture_text
cargo test --test issue_1486_hwpx_partial_tac_table
cargo test --lib
cargo clippy --lib -- -D warnings
```

`cargo test --lib` 결과:

```text
1959 passed; 0 failed; 6 ignored
```

## 판정

Stage 3 구현 및 검증은 완료됐다.

- #1551 merge 기준으로 재기준화했다.
- #1270 첨부 실샘플의 `image2`가 SVG에 방출된다.
- 신규 #1585 회귀 테스트와 관련 기존 회귀 테스트가 통과한다.
- 변경 범위는 caption 내부 `TopAndBottom` picture와 depth 1 nested table caption 보강으로 제한했다.

## 다음 단계

작업지시자 승인 후 Stage 4 최종 보고 및 PR 준비 단계로 진행한다.
