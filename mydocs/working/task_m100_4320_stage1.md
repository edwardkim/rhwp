# task_m100_4320 Stage 1 — 캡션 높이 산식 통일

- **이슈**: [#4320](https://github.com/edwardkim/rhwp/issues/4320)
- **PR**: [#4382](https://github.com/edwardkim/rhwp/pull/4382)
- **브랜치**: `fix/issue-4320-caption-height-dedup`
- **분기 기준**: `upstream/devel` `e48fe8694`
- **상태**: 로컬 전체 검증 + 코퍼스 대조 통과, PR 게시
- **기록일**: 2026-08-09 KST

## 1. 결함 — 같은 값을 두 곳에서 다르게 계산했다

| | 함수 | 산식 |
|---|---|---|
| 렌더 | `LayoutEngine::calculate_caption_height`(`layout/picture_footnote.rs:661`) | `max(line_seg 기반, compose 재계산)` |
| 측정 | `HeightMeasurer::measure_caption`(`height_measurer.rs:2445`) | `line_segs` 단순 합산 |

측정 쪽에는 `compose_paragraph` 호출이 아예 없다(grep 0건).

## 2. 권위 판정 — 렌더 쪽

`git log` 로 렌더 쪽 compose 폴백이 들어온 커밋 `2d973021c`("fix: account for caption line segment
height")를 찾았다. `samples/rowbreak-problem-pages.hwpx` 에서 compose 재계산 높이가 실제 한컴
레이아웃(저장 `line_segs`)보다 작게 나와 캡션이 다음 요소와 겹치는 회귀를 고친 커밋이고, 그 가드가
`tests/issue_rowbreak_chart_overlap.rs::rowbreak_page2_chart_starts_below_title_line` 로 지금도
남아 있다.

즉 렌더 쪽은 **실제 한컴 출력(오버랩 없음)에 맞춰 검증된 산식**이고, 측정 쪽은 그 보정이 없다.

## 3. 구현

렌더 쪽 로직을 `composer::caption_height_px(caption, dpi)` 로 **그대로** 옮기고 양쪽이 호출하게
했다. 새 로직은 없다. `calculate_caption_height` 시그니처는 불변이라 7개 호출부 무영향.

## 4. 측정 경로 동작 변경 — 코퍼스 대조로 확인

이 변경은 **측정 경로의 값을 바꾼다**(`line_segs` 합산 → `max(line_seg, composed)`). 다문단 캡션에서
문단별 `max` 와 합의 차이가 페이지 수를 바꿀 수 있어, release-test 만으로는 부족하다.

`upstream/devel`(e48fe8694)과 이 head 로 `samples/` 681개 문서의 페이지 수를 대조했다:

```
same=678  diff=0  err=3
```

**차이 0건.** err 3건은 전부 암호 문서로 양쪽 동일하게 페이지 수를 얻지 못했다(회귀 아님).

## 5. 범위 밖

`line_seg_height` 는 문단별 `max` 이고 `composed_height` 는 문단별 합(`+=`)인 내부 비대칭이 원래
렌더 쪽에 있었고 공유 함수에도 그대로 남아 있다. `2d973021c` 검증 당시 이 형태로 회귀가 잡혔으므로
동작을 바꾸지 않고 옮겼다. 위 대조에서 페이지 수 차이가 0건이라 실제 영향은 관측되지 않는다.

두 산식이 실제로 다른 값을 내는 문서 사례는 찾지 못했다.

## 6. 검증 (완료)

- `issue_rowbreak_chart_overlap` 20/20, 캡션 통합 8개 파일 12/12, `--lib caption` 63/63,
  `--lib lineseg_compare` 9/9.
- `cargo test --profile release-test --tests` 전체 통과.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` 통과.
- 코퍼스 페이지 수 대조 681건(위 4절).

남은 미래 조건은 GitHub Actions 와 작업지시자 승인, merge 다.
