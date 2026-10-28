# 실제 출력 소비와 WMF 경계 검사

렌더 producer는 `e01e7540fd4c8a67ce5d1d1e5802170973acb998`이며 최종 회귀 원본은
`949195dba653cdef8d8fa1506131f5db2f669e78`에 등록했다. 두 커밋 사이의 런타임 소스는 같다.
사용한 WASM은 상위 [표준 최적화 빌드](../wasm/build.json)의 실제 패키지다.

| 실제 실행 | 결과 | 증거 |
| --- | --- | --- |
| 현재 목표 문서 주요 쪽 | Native·fresh WASM·독립 Print 8개 triplet 직접 판독 | [이미지·해시·판독 기록](target-inspection.json) |
| Canvas 브라우저 계약 | JPEG 인쇄 격자·독립 Print 색상/축·HFT 공급 등 8 PASS | [렌더러 실행 기록](renderers.json) |
| CanvasKit crop | `none`, `zoom`, `fitToSize`, `total` 네 모드 PASS | [렌더러 실행 기록](renderers.json) |
| CanvasKit 실제 WMF | 18쪽 그림과 35쪽 도표의 4영역, 정수 미대응 모두 0/0 | [영역별 PNG·좌표·해시](renderers.json), [18쪽 초기화·readiness](canvaskit/p018-provenance.json), [35쪽 초기화·readiness](canvaskit/p035-provenance.json) |
| Studio 후보 | AI 설정·side panel·AI panel·암호 문서 열기·부분 글꼴 열거 5종 PASS | [실제 초기화 응답 SHA](studio-head.json) |
| Studio 새 base | AI 설정·side panel·AI panel 3종 PASS | [실제 초기화 응답 SHA](studio-base.json) |
| 최종 WMF 검사 본문 | 수정 전 결함 단언 FAIL, 수정 후 PASS | [같은 원본 SHA·라이브러리 SHA·실제 종료 코드](wmf-before-after.json) |

CanvasKit은 software backend에서 현재 Native가 준비한 WMF 자료를 최신 WASM의 공통 인쇄 API로
재생했다. 검증 범위는 두 그림 영역이며, 실제 readiness·image 실패 없음·fallback 미사용과 초기화 때
수신한 WASM SHA까지 확인했다. Studio도 초기화 응답 자체를 비교했으며 별도 재요청으로 대체하지 않았다.

BCP 문단의 2022 원본 두 줄과 수식의 분수·괄호는
[BCP 확대](target-review/bcp-two-lines-focus.png), [수식 확대](target-review/equation-lines-focus.png)에서
Native·최적화 WASM·독립 Print를 함께 확인했다. 각 원본의 줄 구성을 유지하며 2024 원본의 해당
문구는 독립 Print와 같은 한 줄이다.

CanvasKit 진단 초기에 누락한 Chrome 경로, 실제 `LayerImageOp.mime` 필드와 브라우저 inspector
응답 보관 설정은 검증 호출에서 바로잡았다. 그 실패 실행은 ignored output에 보존했고 통과 수에
합산하지 않았다. 렌더 제품 코드나 픽셀 판정은 바꾸지 않았다.

이 문서는 위 소비 범위의 증거다. 전체 Rust·Native Skia 및 새 devel 전쪽 WASM 보존의 최종 판정은
상위 검증 기록에서 별도로 확인한다.
