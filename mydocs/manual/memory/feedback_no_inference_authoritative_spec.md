---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-17
name: ""
description: 워터마크 등 시각 의미를 brightness/contrast 패턴 매칭으로 추정하지 말 것. 한컴 스펙 + 대비 샘플 + 편집기 UI 교차검증으로 비트/속성 확정 후 구현
metadata: 
  node_type: memory
  type: feedback
  originSessionId: da1865ca-614e-44a5-8c3e-ce3fe8956096
---

워터마크 같은 시각 속성의 적용 여부를 "effect=RealPic이면 항상" 또는 "특정값(70/-50) 매칭" 같은 **추정**으로 구현하지 말 것. 작업지시자: "자의적 해석하면 안됩니다. 한컴의 hwp, hwpx에는 명확하게 적용 비트/속성이 존재합니다."

**Why**: Issue #1156 _v2 (2026-05-29). 워터마크 회귀를 brightness/contrast 패턴 매칭으로 정정하려다 CI characterization 테스트를 깨뜨림(native-skia `renders_page_background_fill_border_and_image`). 추정 기반 정정은 다른 케이스 회귀를 부른다.

**확정된 사실 (이 건 한정)**: HWP/HWPX에 워터마크 적용 **비트는 존재하지 않음**. 워터마크 = `밝기≠0 && 대비≠0` (AND, effect 무관). 한컴 편집기는 "워터마크 효과" 해제 시 밝기·대비를 0/0으로 되돌림. 코드 `is_watermark()`, 문서 `한글문서파일구조3.0.md` 표 45 직후에 명문화.

**How to apply**: 시각 의미(워터마크/효과/플래그)를 구현·정정하기 전에 3종 교차검증 — (1) 한컴 공식 파일구조 스펙(3.0/5.0 표), (2) 적용/미적용 **대비 샘플** 두 개의 IR diff로 유일 차이 식별, (3) 한컴 편집기 UI. 셋이 일치할 때만 확정. 작업지시자가 대비 샘플(`samples/water-mark.hwp/.hwpx` 같은)을 제공하면 그게 결정적 근거. [[feedback_self_verification_not_hancom]] [[feedback_hancom_compat_specific_over_general]] [[feedback_image_renderer_paths_separate]]
