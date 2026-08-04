---
kind: working-note
status: active
issue: 2635
stage: 1
---

# Task #2635 Stage 1: 순수 RawSvg 첫 화면 지연 분석

## 재현 범위

- `samples/chart/원형/쪼개진원형.hwp`
- `samples/chart/원형/쪼개진원형.hwpx`

첫 페이지에는 순수 벡터 `rawSvg` 차트가 하나 있다. layer tree의 SVG는
`<g class="hwp-ooxml-chart">`로 시작하며 내부 `data:image/...;base64`가 없다.

## 확정 원인

[f32a99856](https://github.com/edwardkim/rhwp/commit/f32a99856462b95208dd1b531a506ff7880f872d)가
이미지 재렌더를 `200ms / 600ms / 1500ms` 단계 재시도에서 1500ms 단일 fallback으로 바꿨다.
`PageRenderer.prefetchLayerImages`는 image base64와 rawSvg 내부 data URL만 decode 대상으로 찾는다.
따라서 순수 SVG는 완료 신호를 만들지 못하고 `flow-static` 재렌더를 fallback까지 기다린다.

## 구현 제약

- 일반 raster 이미지의 decode 완료 기반 단일 재렌더 최적화는 유지한다.
- 순수 `rawSvg`만 SVG decode 완료 또는 짧은 전용 재시도로 첫 화면 지연을 줄인다.
- 단순히 모든 `flow-static`을 반복 렌더하지 않는다.

## 다음 단계

`web_canvas`의 SVG data URL 생성 규칙과 브라우저 이미지 cache 키를 확인한 뒤, 동일 SVG의 decode 완료를
`scheduleReRender`에 연결할 수 있는지 검증한다. 동일 키를 만들 수 없으면 RawSvg 전용의 제한된 첫-frame 재시도를
추가하고 `쪼개진원형`을 포함한 회귀 테스트로 지연 상한을 고정한다.
