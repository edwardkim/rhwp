---
kind: report
status: active
last_verified: 2026-11-23
---

# #7766–#7776 조판 복원 구현 기록

작업 브랜치는 `task/7766-inline-table-page-origin`이며 검증 기준 `upstream/devel`은
`321ab8f97791ecf58b56f178f49b7795a836fb79`다.

## #7766 명시적 쪽 나눔의 인라인 표와 본표 이어받기

#1855가 캡션과 본표의 같은 쪽 소유를 복원한 뒤에도 44쪽의 캡션이 한컴 Print보다
약 6.6px 위에 있었다. 원본 HWPX는 명시적 쪽 나눔 직후 캡션 표를 저장 줄 원점
500 HU에 두고, 같은 문단의 부동 본표는 독립 앵커의 4327 HU 위치를 사용한다.
저장 인라인 줄에 원점을 전달하고, 첫 부동 조각을 확정할 때는 분할 예산에 사용한
문단 상대 원점에서 끝을 계산한다.

첫 번째 위치 보정 빌드에서 독립 한컴 Print와 96 DPI Native Skia 52쪽을 비교했다.
2px raw 실루엣 최저는 94.85%에서 98.40%로, 44쪽은 94.85%에서 100.00%로 올랐다.
다른 51쪽의 Native PNG 바이트와 44→45쪽의 이어받기 컷은 같았다. 44쪽 캡션의
윗변은 105.8px에서 112.5px로 이동했고, 본표는 161.6px에서 시작했다.
한컴 Print의 캡션 윗변은 112.357px였다.

위치 보정 시점의 원본·Print·TSV SHA-256을 기록한다.

| 자료 | SHA-256 |
| --- | --- |
| `samples/issue1853_caption_precedes_body_split.hwpx` | `46bd0142f7fe2f68dffb9768e08d65488feeb19f5fd6916f9ad90ae5d72a96a9` |
| `pdf/issue1853_caption_precedes_body_split-2024.pdf` | `c3befae95a2135761066c44435a331bb55d0d27bac2f97d8487358f1b9f2bd9b` |
| 기준 Native 전쪽 TSV | `5f936e1dbe5539523e9d3f1b37789e9e4ccfceda6e9763191c7e2bd42845b18f` |
| 위치 보정 Native 전쪽 TSV | `85e39b5f3008fd7f51373a8053c2adeebf3dcdd38c6f365b1b4ee491f58d8095` |

첫 조각의 확정 단계는 원래 분할 예산이 사용한 독립 문단 원점 대신 인라인 줄을 소비한
흐름 커서에서 표 높이를 더해, 44쪽의 `used_height`를 915.2px에서 974.0px로 늘렸다.
`FragmentBudget`의 원점을 방출 결과에도 연결해 두 개체의 실제 점유 끝을 사용하도록
보정했다. 이 끝점 보정이 포함된 Native 및 fresh WASM 전쪽 검증과 회귀 복원은 진행 중이다.
