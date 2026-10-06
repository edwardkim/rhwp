---
kind: working
status: active
issue: 6928
---

# 글자처럼 취급 개체 줄의 저장 걸음이 바닥 없이 통과한다 (#6928)

작업 브랜치: `fix/6928-tac-picture-line-advance`
대상: `src/renderer/layout/paragraph_layout.rs`

## 한 줄

저장 사다리가 주는 줄 걸음(`stored_line_advance`)의 바닥 가드가 **글자 높이 하나**뿐이라,
글리프가 없는 **글자처럼 취급 개체 줄**에서는 가드가 풀려 2.7px 걸음이 107.1px 줄을 덮는다.

## 이슈가 요구한 것

- 1쪽 상단 겹침을 없앤다(배너 그림 뒤 내용이 −106px 위로 올라온다).
- 배너 자신의 위치(−2.5px)는 별개 축이므로 건드리지 않는다.

## 원인

```rust
// stored_line_advance
let step = hwpunit_to_px(next.vertical_pos - seg.vertical_pos, self.dpi) - line_spacing_px;
(step > 0.0 && step < line_height && (max_fs <= 0.0 || step + 0.5 >= max_fs)).then_some(step)
```

`flow_step = stored_line_advance.unwrap_or(line_height)` 이므로 이 걸음이 줄 높이를 덮는다.
바닥은 `max_fs`(그 줄 글리프 높이) 하나인데, **개체가 줄 높이를 정하는 줄에는 글리프가
없어 `max_fs` 가 0** 이다. `max_fs <= 0.0` 가 참이 되어 가드가 통째로 풀린다.

계측(1쪽 문단 0, 배너 그림 호스트):

```text
PARA in  pi=0 y_start=94.5 lines=2 line_h_px=[107.1, 107.1] spacing_px=[0.0, 0.0]
  LINE i=0 y=94.5 flow_h=2.7 branch=normal      ← 줄 높이 107.1 인데 흐름 걸음이 2.7
  LINE i=1 y=97.1 branch=skip_empty
PARA out pi=0 y=97.1 (전진 2.7)
```

대조군은 정상이다.

```text
  pi=2  20.0px 줄 → 36.0 전진      pi=3  26.7px 줄 → 45.3 전진
```

곧 이슈가 지적한 "줄 높이는 맞는데 다음 문단이 그 높이를 안 넘겨받는다" 의 기계다.

## 수정

그런 줄의 바닥은 **개체가 정한 줄 높이 자체**다. `ComposedParagraph::inline_controls` 로
그 줄이 `Table`·`Shape` 를 품는지 보고, 품으면 바닥을 `max_fs.max(line_height)` 로 올린다.
`step < line_height` 와 함께 걸리므로 그 줄은 저장 걸음을 쓰지 않고 `line_height` 로
전진한다. 글자 줄은 `max_fs` 그대로라 종전 동작이다.

## 검증 실측 — 한컴 2024 정본 대조

| 요소 | 정본 | 수정 전 | 수정 후 | 잔차 |
|---|---:|---:|---:|---:|
| 표 `특허심사2국` | 217.5 | 113.3 | **217.7** | +0.2 |
| 표 `문 의` | 231.1 | 124.6 | **229.0** | −2.1 |
| 표 `사무관 이귀남` | 247.3 | 140.7 | **245.1** | −2.2 |
| 엠바고 1행 | 276.7 | 172.4 | **276.8** | +0.1 |
| `OPEN` 로고 | 283.2 | 176.5 | **280.9** | −2.3 |

−106px 가 **±2.3px 이내**로 들어온다. 배너 자신(94.5, 정본 97.0)은 이 축과 별개다.

## 게이트

| 게이트 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | OK |
| `rust-unit-test-tiers --check` | 4205 (기준선 유지) |
| clippy 3단 · workspace build | OK |
| `rust-test-suite-manifest --check` | 48/48 targets |
| 전 타깃 순차(28 suite + lib) | **9206 passed / 0 failed / 38 ignored** |
| 코퍼스 래칫(겹침·off-canvas·overflow-cell) | **증가·신규 발생 0건** |

겹침 래칫은 이 축의 관문이다 — `#6924` 에서 잘못된 구현을 정확히 잡아낸 그 게이트다.

## 남긴 것

- 배너 그림 자신의 −2.5px 는 다른 축이다(이슈도 그렇게 적었다).
- 표·엠바고·로고에 남는 −2.1~−2.3px 잔차는 저장 걸음이 아니라 그 아래 줄간격 회계
  쪽으로 보인다. 겹침은 해소됐고 이 이슈의 요구(−106px)는 닫혔다.
