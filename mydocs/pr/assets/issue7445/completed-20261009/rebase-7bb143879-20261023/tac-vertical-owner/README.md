# 원본 TAC의 가로·세로 소유 검증

[72단계 보고서](../../../../../../working/task_m100_7445_rebase_repair_stage72.md)의 같은 입력·독립 PDF다.
`before-native`는 부모0c94eff7b7, `after`는 보고서의 source/producer로 직접 다시 출력했다.
`devel-native`는 고정7bb143879와 runtime source가 같은 보존 Native 출력이며 fresh WASM base 대조를 대신하지 않는다.

논문 전 7쪽 Native 최저 97.75440%, fresh WASM 최저 97.70269%다. 5쪽은98.40204%/98.36191%다.
기존 캡션 10조건은 양쪽 모두100% raw이며 RGB 일치는 아니다. 전쪽 TSV를 원본 바이트로 보존했다.
기존 task1725 전 242쪽은201쪽 미달이 남아 있어 전체 PR 수용 증거가 아니다.

| 출력 | review | overlay |
| --- | --- | --- |
| 보정 전 논문5쪽 | [review](treatise-before-native-p005-review.png) | [overlay](treatise-before-native-p005-overlay.png) |
| 보정 후 Native | [review](treatise-after-native-p005-review.png) | [overlay](treatise-after-native-p005-overlay.png) |
| 보정 후 fresh WASM | [review](treatise-after-wasm-p005-review.png) | [overlay](treatise-after-wasm-p005-overlay.png) |
| 왼쪽 캡션 Native | [review](side-caption-native-p001-review.png) | [overlay](side-caption-native-p001-overlay.png) |
| 왼쪽 캡션 fresh WASM | [review](side-caption-wasm-p001-review.png) | [overlay](side-caption-wasm-p001-overlay.png) |

보정 후 캡션이 직전 본문과 겹치지 않고 소유 표 위에 놓인다. 글자·머리말 괘선의 미세한 차이는 남는다.
좌측 캡션의 번호1·3, 세 번째 표의 저장 위치를 보존한다. 원본/PDF/글꼴 및 완료 이미지·로그는 삭제하지 않았다.
