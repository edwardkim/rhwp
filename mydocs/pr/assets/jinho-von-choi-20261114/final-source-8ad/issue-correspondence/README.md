# 원 신고 쪽에 대응하는 후속 댓글 증적

최종 제품 `8ad1c5c08aa4cd68accbd2af6b9c60bc458d8992`에서 이미 완료한 Native/최적화 WASM 캡처로 생성했다. 제출 head `b6a91166761b212d8501faf70b4b13e19b26474e`의 입력·독립 PDF·기존 원본 TSV와 해시를 대조했다. 렌더러·파서·저장기·DocumentCore 및 시각 harness는 두 source 사이에 변경이 없다. 새 렌더·한컴 실행·입력 변경·기준 변경을 하지 않았다.

기존 `02-both` p2는 메인테이너가 보완한 공통 프레임의 증거다. 원 신고 #9046의 `05-even-odd`·`10-both-even-odd` 저장 후 p2 증거를 대신하지 않도록 여기에서 따로 보존한다. 색상 후속 본문 p3도 원 신고 #8318의 p1·2 검정 배경을 대신하지 않는다.

[정확한 입력·출력·PNG·TSV·생산자 해시](receipts.json) · [색상 10개 위치 대조](issue8318-sentinel-contract.json).

| 사례·쪽 | 경로 | raw | pixel match | visual_accuracy_proxy_percent | review / overlay |
| --- | --- | ---: | ---: | ---: | --- |
| 05-even-odd-hwp p2 | native | 100.00000% | 99.96576% | 75.83201% | [review](issue9046-even-odd-native-review.png) / [overlay](issue9046-even-odd-native-overlay.png) |
| 05-even-odd-hwp p2 | wasm | 100.00000% | 99.96565% | 75.75277% | [review](issue9046-even-odd-wasm-review.png) / [overlay](issue9046-even-odd-wasm-overlay.png) |
| 10-both-even-odd-hwp p2 | native | 100.00000% | 99.96576% | 75.83201% | [review](issue9046-both-even-odd-native-review.png) / [overlay](issue9046-both-even-odd-native-overlay.png) |
| 10-both-even-odd-hwp p2 | wasm | 100.00000% | 99.96565% | 75.75277% | [review](issue9046-both-even-odd-wasm-review.png) / [overlay](issue9046-both-even-odd-wasm-overlay.png) |
| color-saved p1 | native | 100.00000% | 99.46098% | 25.26070% | [review](issue8318-color-p1-native-review.png) / [overlay](issue8318-color-p1-native-overlay.png) |
| color-saved p1 | wasm | 100.00000% | 99.46098% | 25.27233% | [review](issue8318-color-p1-wasm-review.png) / [overlay](issue8318-color-p1-wasm-overlay.png) |
| color-saved p2 | native | 100.00000% | 99.45750% | 25.20891% | [review](issue8318-color-p2-native-review.png) / [overlay](issue8318-color-p2-native-overlay.png) |
| color-saved p2 | wasm | 100.00000% | 99.45750% | 25.20891% | [review](issue8318-color-p2-wasm-review.png) / [overlay](issue8318-color-p2-wasm-overlay.png) |

각 사례는 3쪽 전체의 Native/fresh WASM raw 최저가 100.00%다. raw는 색상 경계 보정 전 2px 실루엣, pixel match는 전체 RGB 픽셀, proxy는 내용 픽셀 중심 자동 보조값이다. 사람의 판정 정확도나 픽셀 완전 동일을 뜻하지 않는다. 기존 silhouette-only 실행은 flagged 후보 수를 산출하지 않았다.

직접 판독: 바탕쪽 저장본의 짝수 쪽 막대는 왼쪽 용지 끝에서 보존되고 MP-EVEN·본문이 같은 순서로 보인다. 색상 저장본 p1·2의 표 내부는 흰색이며 각각 서식 1·서식 2가 표시된다. 바탕쪽의 비적용 Both 내용 보존은 그림만으로 판단하지 않고 저장·재열기와 슬롯 회귀로 검증했다.

색상 원 신고의 한컴 2018 보정본과 이번 저장본에서 같은 10곳이 모두 `#FFFFFFFF`임을 XML로 확인했다. 새 이미지의 기준은 한컴 2020 Print이며 2018 프로그램을 새로 실행한 결과라고 주장하지 않는다.

이 디렉터리는 제품 merge 뒤 옵션 M 운영 기록에 추가하는 비교 asset이다. 댓글의 새 PNG와 이 기록은 실제 운영 기록 SHA에 고정하고, 입력·기존 TSV·독립 PDF는 실제 제품 merge SHA를 사용한다.
