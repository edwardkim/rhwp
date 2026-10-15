# #7899 편집 저장본과 독립 Print

원본은 저장소의 `samples/exam_math_8.hwp`다. `exam_math_8-edited.hwp`는 통합 source `97616c5223cde8985b431f60faba811f6d0c6b47`의 Native와 fresh Docker WASM에서 첫 문단 시작에 `A`를 삽입하고 저장한 동일 파일이다. 모든 바탕쪽 목록을 보존하고, 큰 수식의 저장 줄 시작은 원시 제어문자 위치를 포함한다.

| 자료 | SHA-256 |
| --- | --- |
| 편집 저장본 | `e00edfc63c35d5753e1369bad72de5187a485e9a6d0e9406f5746cb7c7cebf52` |
| 독립 한컴 Print | `ab34d0ffd3647f417d75657c1f75d3f5ac7c19aa1e98a4e4edc6e8ee44796173` |

PDF는 같은 편집 저장본을 한컴 2020 11.0.0.9136에서 preprocessing 없이 one-up Print한 1쪽 기준이다. MCP job은 `b1dce92f-ae09-4af7-ba97-e415be16e951`이며 PDF Creator `Hwp 2020 0.0.0.0`, Producer `Hancom PDF 1.3.0.550`을 실제 실행 제품 버전과 구분한다.

이 자료는 재현·독립 비교를 위한 reference다. 조판 baseline/golden 또는 새 렌더링 assertion을 등록하지 않는다. 메인터너는 `{` 기호 크기와 다중 행 수식의 약 2px 세로 위치 차이를 이번 PR의 예외로 수용했다. Native 77.01188%, fresh WASM SVG 진단 77.20043%로 자동 90% gate 통과를 주장하지 않는다. WASM Canvas2D 캡처에는 글리프 누락이 있어 정상 시각 증거로 사용하지 않는다. 위치·기호 차이와 캡처 경로 문제는 후속 검증 범위다.
