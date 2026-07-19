---
kind: guide
status: active
canonical: mydocs/tech/README.md
last_verified: 2026-07-17
---

# webhwp 역분석 문서 지도

이 디렉터리는 2026-02의 minified webhwp 번들을 관찰한 historical investigation이다. 난독화 식별자,
번들 크기, 기능 수치는 현재 제품 계약이 아니며 rhwp 구현의 권위 자료로 직접 사용하지 않는다.

- 기능별 기록: [텍스트](01_text_rendering.md), [표](02_table.md), [도형·이미지](03_shape_image.md),
  [페이지 레이아웃](04_page_layout.md), [기타 컨트롤](05_other_controls.md),
  [이벤트](06_events.md), [툴바](07_toolbar.md)
- 보조 분석: [표 함수](table_functions.md), [텍스트 측정](text_measurement.md),
  [파싱 구조 비교](parsing_comparison.md)

현재 rhwp 계약은 [렌더링 엔진 설계](../rendering_engine_design.md),
[표 레이아웃 규칙](../table_layout_rules.md), [폰트 fallback 전략](../font_fallback_strategy.md)을 우선한다.
