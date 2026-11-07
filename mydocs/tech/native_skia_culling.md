---
kind: reference
status: active
canonical: mydocs/tech/native_skia_culling.md
last_verified: 2026-11-06
---

# Native Skia의 보수적 viewport culling

Native raster replay는 실제 op 기하를 증명할 수 있는 단색 사각형·타원과 그 하위 트리만
현재 canvas clip 밖에서 건너뛴다. cold 고정 타일에도 같은 replay가 사용된다.
기존 `visual_bounds`는 추정값이므로 이 판정에 사용하지 않는다. 논리 node bounds가
잘못되거나 작아도 실제 도형 op에서 bounds를 다시 구한다.

회전·대칭 도형, picture pen, gradient, pattern, shadow, text, SVG, image 및
알 수 없는 op는 증명 범위 밖이다. 알 수 없는 자식이 하나라도 있으면 부모 전체를
건너뛰지 않는다. 그 부모 안에서 독립적으로 증명된 다른 자식은 판정할 수 있다.
그룹 자체의 affine transform은 부모 proof를 닫으며 자식은 현재 local clip을 사용한다.
skew·rotation·perspective·퇴화 CTM에서는 판정을 생략한다. clip은 bounds를 줄이는
근거로 사용하지 않아 효과나 글리프의 논리 bbox 밖 잉크를 추측해 버리지 않는다.

f32 draw 기하와 현재 장치 배율을 기준으로 획의 한 장치 픽셀 하한, join, AA 및
인쇄 fill snapping 여유를 넉넉히 포함한다. 비유한 값이나 축약된 f32 기하는 재생한다.
증명 index는 재생마다 반복형 순회로 만들며 node/op 합산 65,536개를 넘으면 그 호출의
culling을 생략한다. 영구 캐시나 새 font/resource 소유권은 추가하지 않는다.

`SkiaLayerRenderer::with_viewport_culling(false)`는 픽셀·재생 수 대조용 탈출구다.
기본 raster 경로는 켜져 있고 Direct PDF 등 strict replay는 항상 전체를 검사·재생한다.
`LayerRenderDiagnostics::layer_nodes_culled`는 건너뛴 subtree root의 호출 수이며
후손을 방문하거나 합산하지 않는다. 기존 replay 수와 마찬가지로 paint plane별 호출이다.

기존 Native region cache case에 off-clip subtree, 작은 배율의 최소 획과 AA,
알 수 없는 shadow, cold tile 출력 대조를 추가한다. 이 계약은 최종 한컴·실물 OS/GPU
검증이나 wall-clock 개선률을 뜻하지 않는다. 실제 앱 UI는 포함하지 않는다.
