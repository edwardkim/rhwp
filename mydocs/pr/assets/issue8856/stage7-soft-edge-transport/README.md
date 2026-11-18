# 그림 효과 반경의 조판·그리기 전달

`PictureEffects.soft_edge_radius`를 `ImageNode.soft_edge_radius_hu`에 전달한다.
원본 HWPUNIT 값이며 여기서 화면 DPI나 bitmap 픽셀로 바꾸지 않는다. 본문·도형 그룹,
표 셀·inline·각주의 6개 실제 그림 생성 경로에 적용하고 그림 채움의 기본값은 None이다.
효과 없는 노드의 JSON은 새 null 필드를 추가하지 않도록 None을 생략한다.

새 관계 검사는 원본 모델의 반경 목록과 전쪽 조판 트리·Print PaintOp의 반경 목록을 비교한다.
실제 수정 전에는 기존 3 PASS·새 1 FAIL이었고, 보정 후 4 PASS다. 효과 없는 실제 그림의
노드 JSON에 필드가 생기지 않는 것도 확인했다. 최소 글꼴 macOS Native Skia에서 기존
#8593 크기·자르기 2건과 앞 단계 기호 1건을 함께 실행해 총 7 PASS·0 FAIL을 확인했다.
이는 반경의 전달 검증이며 실제 alpha 마스크 생성·Native/fresh WASM 시각 수용 완료가 아니다.

## 독립 좌표 대조군

[이전 물리 대조군](../stage5-soft-edge-source/independent-controls.json)의 반경 1000 HU,
출력 틀 30000×20000 HU, 단색 그림을 유지하고 다음 세 조건을 각각 2020·2024로 Print했다.
기존 원본·기준 PDF를 대체하지 않는다.

| 조건 | 원래 크기 | 개체 확대 행렬 | 원본 자르기 |
| --- | --- | --- | --- |
| scale2x2 | 15000×10000 HU | 2×2 | 전체 |
| scale2x1 | 15000×20000 HU | 2×1 | 전체 |
| crop | 30000×20000 HU | 1×1 | 각 축 양끝 10% |

실제 6개 PDF는 모두 1쪽이다. 확대 두 조건은 두 엔진 모두 같은 출력 프레임의 mask 표본이
기존 대조군과 완전히 같다. 잘린 대조군은 새 잘림 경계에도 감쇠가 생기며, 원래 마스크와
2020에서 156표본, 2024에서 2표본 차이가 있다. 잘림 대조군까지 완전히 같다고 표현하지 않는다.
원 bitmap을 먼저 흐리게 한 뒤 자르거나 원래 크기 비율을 반경에 곱하는 구현으로 대신하지 않는다.
이 대조만으로 Hancom 내부의 blur kernel이나 API를 특정하지 않는다.

- 입력·Print·엔진·작업 ID: [좌표 대조군 원장](independent-coordinate-controls.json)
- 실제 프레임·mask 비교: [비교 기록](coordinate-mask-comparison.json)
- 소스·로그 해시와 실제 검사: [검증 원장](validation.json)
- 입력: [scale2x2](../../../../../pdf/issue8856/soft-edge-scale2x2.hwpx), [scale2x1](../../../../../pdf/issue8856/soft-edge-scale2x1.hwpx), [crop](../../../../../pdf/issue8856/soft-edge-crop.hwpx)
- 2020 Print: [scale2x2](../../../../../pdf/issue8856/soft-edge-scale2x2-2020.pdf), [scale2x1](../../../../../pdf/issue8856/soft-edge-scale2x1-2020.pdf), [crop](../../../../../pdf/issue8856/soft-edge-crop-2020.pdf)
- 2024 Print: [scale2x2](../../../../../pdf/issue8856/soft-edge-scale2x2-2024.pdf), [scale2x1](../../../../../pdf/issue8856/soft-edge-scale2x1-2024.pdf), [crop](../../../../../pdf/issue8856/soft-edge-crop-2024.pdf)

형식별 실제 한컴 버전·Print method·전처리 여부와 입력/출력 SHA-256을 원장에 보존한다.
글꼴 바이너리·임시 Python·로그는 Git에 넣지 않는다. 반경 전달 뒤의 실제 도색과 최종
Native/fresh WASM 전쪽·전체 회귀·필수 게이트는 계속 검증한다.
