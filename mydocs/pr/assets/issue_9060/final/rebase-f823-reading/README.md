# 새 base PR #9083의 기능 보존 직접 판독

판독 기록: 2026-11-15T11:02:42.656394+09:00.

원 PR #9075의 개선 주장 증적은 [원 문서 5·6쪽](../main/README.md)이다. 이 자료는 새 base의 기능 보존을 확인한 별도 대조군이며 같은 증적으로 바꾸어 쓰지 않는다.

같은 실제 Linux·시스템 글꼴에서 source `833c22d0de3e6ec8cbe090a1358a6d457bac8039`의 Native와 최적화 fresh WASM PNG를 독립 한컴 Print와 대조했다. 기존 PDF를 재인쇄하거나 PNG를 다시 렌더하지 않고, 완료한 전쪽 출력에서 같은 쪽 review·overlay를 구성했다.

| 원본 식별자 | 쪽 | 경로 | raw | review / overlay |
| --- | ---: | --- | ---: | --- |
| `cbta` | 118 | native | 98.46% | [review](native/cbta-review-118.png) · [overlay](native/cbta-overlay-118.png) |
| `cbta` | 162 | native | 99.97% | [review](native/cbta-review-162.png) · [overlay](native/cbta-overlay-162.png) |
| `cbta` | 166 | native | 99.97% | [review](native/cbta-review-166.png) · [overlay](native/cbta-overlay-166.png) |
| `recommendation` | 4 | native | 99.96% | [review](native/recommendation-review-004.png) · [overlay](native/recommendation-overlay-004.png) |
| `exam` | 2 | native | 99.95% | [review](native/exam-review-002.png) · [overlay](native/exam-overlay-002.png) |
| `api` | 35 | native | 100.00% | [review](native/api-review-035.png) · [overlay](native/api-overlay-035.png) |
| `footnote` | 37 | native | 98.75% | [review](native/footnote-review-037.png) · [overlay](native/footnote-overlay-037.png) |
| `cbta` | 118 | wasm | 98.41% | [review](wasm/cbta-review-118.png) · [overlay](wasm/cbta-overlay-118.png) |
| `cbta` | 162 | wasm | 99.91% | [review](wasm/cbta-review-162.png) · [overlay](wasm/cbta-overlay-162.png) |
| `cbta` | 166 | wasm | 99.92% | [review](wasm/cbta-review-166.png) · [overlay](wasm/cbta-overlay-166.png) |
| `recommendation` | 4 | wasm | 99.96% | [review](wasm/recommendation-review-004.png) · [overlay](wasm/recommendation-overlay-004.png) |
| `exam` | 2 | wasm | 99.94% | [review](wasm/exam-review-002.png) · [overlay](wasm/exam-overlay-002.png) |
| `api` | 35 | wasm | 100.00% | [review](wasm/api-review-035.png) · [overlay](wasm/api-overlay-035.png) |
| `footnote` | 37 | wasm | 98.75% | [review](wasm/footnote-review-037.png) · [overlay](wasm/footnote-overlay-037.png) |

## 직접 확인한 내용

- `cbta`: 118쪽의 첫 그림 표와 뒤 본문, 162·166쪽의 병합 셀·마지막 행·외곽과 내용 순서를 대조했다. 118쪽 그림 외곽의 농도 차이를 RGB 동일성으로 표현하지 않는다.
- `recommendation`: 목차의 오른쪽 쪽 번호·점선과 실제 회색 방사형 채움의 위치·범위를 대조했다. Canvas2D 출력이며 새 CanvasKit 스크린샷으로 표기하지 않는다.
- `exam`: 두 열과 13~20번 문항, 밑줄의 시작·끝과 보기·창 모양 상자의 소유를 대조했다.
- `api`: Return Values·Remarks·Example·CurFieldState의 내용 순서와 예제·Syntax 상자, 밑줄 링크를 대조했다.
- `footnote`: 22~31번 행과 분담금 강조·주 번호·병합 셀·마지막 행·쪽표시를 대조했다. 일부 주 번호와 글자 농도의 차이는 점수와 별도로 남는다.

공식 raw 수치는 RGB 픽셀 완전 일치가 아니다. 새 base와 후보의 동일 환경 전쪽 TSV 대조 및 전체 Rust 검사는 별도 완료 기록을 따른다. 현재 이 직접 판독만으로 전체 PR 준비 완료를 선언하지 않는다.

## CanvasKit 기존 실행 근거의 재사용 경계

권고안 4쪽의 실제 WASM paint JSON 전체가 새 base와 후보에서 바이트로 같으며 SHA-256은 `7f55004481831b80ecc948e6e293f6c8d82457469323cbd3a50fc2803a56342d`이다. `radialCircle`을 생산하는 공통 코드와 CanvasKit 소비 코드, 실제 `wasm-bridge.ts`·`renderer-session.ts`가 새 base와 같다. 기존 PR #9083의 실제 CanvasKit 반경 전달 증거를 이 경계에서 재사용한다. 새 CanvasKit PNG나 새 측정값을 산출했다고 주장하지 않는다.

## 최종 후보의 동일 그림 확인

제품 source `c95f9ade4e8acf9d901c01136ef2b9bacd60c497`로 다시 출력한 위 14개 주요 쪽 PNG는 이전 직접 판독 PNG와 각각 바이트가 같다. 따라서 같은 원본·쪽의 기존 review·overlay와 판독을 재사용했다. 실제 전쪽 TSV는 [최종 같은 환경 대조](../hft-style-coverage-tsv/README.md)에 연결한다. 점수만으로 그림 동일성을 추정하지 않았다.

CBTA 47쪽은 새 서식 범위 보정 뒤 다시 판독했다. Native raw 99.18%의 [review](en-dash-p047/final-native-review-047.png)·[overlay](en-dash-p047/final-native-overlay-047.png), fresh WASM raw 98.54%의 [review](en-dash-p047/final-wasm-review-047.png)·[overlay](en-dash-p047/final-wasm-overlay-047.png)를 최종 그림으로 사용한다. 기존 `candidate-*` 그림은 source833의 보정 전 기록이다.
