# #7688 대표 출력 증적

검증한 제품·회귀 코드 기록 commit은 `33991c311a81aef43392f9f9940f5ab805972d8d`다. 이 증적 복사는 제품 출력이나 PNG를 변경하지 않는다. Native 바이너리 및 최종 20개 재출력의 byte 동일성은 [7단계 기록](../../../working/task_m100_7688_stage7.md)에 연결했다. 2026-10-09 메인테이너가 최종 비교 자료의 시각 검증을 통과 판정했다.

SVG raster인 Visual Sweep과 실제 Native Skia PNG·Studio Canvas 캡처를 구별한다. Studio는 Chrome 154.0.8037.57, CanvasKit software, screen profile이며 GPU 검증을 대신하지 않는다. Native Skia 캡처는 print profile이다. 수치·미검증·재현 명령은 [최종 보고서](../../../report/task_m100_7688_report.md)에 있다.

## image-crop · p1

원문: [pic-crop-01.hwp](../../../../samples/pic-crop-01.hwp), 독립 Print: [image-crop-2020.pdf](../../../../samples/issue7688/reference/image-crop-2020.pdf)

| 경로 | review | overlay |
| --- | --- | --- |
| native | ![image-crop p1 native review](image-crop-p001-native-review.png) | ![image-crop p1 native overlay](image-crop-p001-native-overlay.png) |
| wasm | ![image-crop p1 wasm review](image-crop-p001-wasm-review.png) | ![image-crop p1 wasm overlay](image-crop-p001-wasm-overlay.png) |

| 실제 Native PNG | Studio Canvas2D | Studio CanvasKit software |
| --- | --- | --- |
| ![image-crop-p001-native-skia.png](image-crop-p001-native-skia.png) | ![image-crop-p001-studio-canvas2d.png](image-crop-p001-studio-canvas2d.png) | ![image-crop-p001-studio-canvaskit.png](image-crop-p001-studio-canvaskit.png) |

## shape-group · p1

원문: [group-drawing-02.hwp](../../../../samples/group-drawing-02.hwp), 독립 Print: [shape-group-2020.pdf](../../../../samples/issue7688/reference/shape-group-2020.pdf)

| 경로 | review | overlay |
| --- | --- | --- |
| native | ![shape-group p1 native review](shape-group-p001-native-review.png) | ![shape-group p1 native overlay](shape-group-p001-native-overlay.png) |
| wasm | ![shape-group p1 wasm review](shape-group-p001-wasm-review.png) | ![shape-group p1 wasm overlay](shape-group-p001-wasm-overlay.png) |

| 실제 Native PNG | Studio Canvas2D | Studio CanvasKit software |
| --- | --- | --- |
| ![shape-group-p001-native-skia.png](shape-group-p001-native-skia.png) | ![shape-group-p001-studio-canvas2d.png](shape-group-p001-studio-canvas2d.png) | ![shape-group-p001-studio-canvaskit.png](shape-group-p001-studio-canvaskit.png) |

## equation-inline · p1

원문: [eq-01.hwp](../../../../samples/eq-01.hwp), 독립 Print: [equation-inline-2020.pdf](../../../../samples/issue7688/reference/equation-inline-2020.pdf)

| 경로 | review | overlay |
| --- | --- | --- |
| native | ![equation-inline p1 native review](equation-inline-p001-native-review.png) | ![equation-inline p1 native overlay](equation-inline-p001-native-overlay.png) |
| wasm | ![equation-inline p1 wasm review](equation-inline-p001-wasm-review.png) | ![equation-inline p1 wasm overlay](equation-inline-p001-wasm-overlay.png) |

| 실제 Native PNG | Studio Canvas2D | Studio CanvasKit software |
| --- | --- | --- |
| ![equation-inline-p001-native-skia.png](equation-inline-p001-native-skia.png) | ![equation-inline-p001-studio-canvas2d.png](equation-inline-p001-studio-canvas2d.png) | ![equation-inline-p001-studio-canvaskit.png](equation-inline-p001-studio-canvaskit.png) |

## issue7333 · p31

원문: [aaaaaa.hwp](../../../../samples/issue7333/aaaaaa.hwp), 독립 Print: [aaaaaa-2020.pdf](../../../../pdf/issue7333/aaaaaa-2020.pdf)

| 경로 | review | overlay |
| --- | --- | --- |
| native | ![issue7333 p31 native review](issue7333-p031-native-review.png) | ![issue7333 p31 native overlay](issue7333-p031-native-overlay.png) |
| wasm | ![issue7333 p31 wasm review](issue7333-p031-wasm-review.png) | ![issue7333 p31 wasm overlay](issue7333-p031-wasm-overlay.png) |

| 실제 Native PNG | Studio Canvas2D | Studio CanvasKit software |
| --- | --- | --- |
| ![issue7333-p031-native-skia.png](issue7333-p031-native-skia.png) | ![issue7333-p031-canvas2d.png](issue7333-p031-canvas2d.png) | ![issue7333-p031-canvaskit.png](issue7333-p031-canvaskit.png) |
