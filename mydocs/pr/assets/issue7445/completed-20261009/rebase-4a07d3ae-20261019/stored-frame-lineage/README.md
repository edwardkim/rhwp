# 저장 프레임 계보 증적

#3930 원PR4058→후속4763의 저장 전후 표/바탕쪽 보존 계약을 복구한다.
294쪽 Native/fresh WASM의 전후 TSV와 review/overlay를 보존한다. 전체383쪽 시각 수용 자료가 아니다.
논리 출력383쪽 대조에서는 원본 변화0, 저장본294쪽만 변화했으며 기존33쪽 형식 차이는 유지된다.

원본·저장본·독립 Print·Native/fresh WASM 생산 hash와 실제 초기화는 `validation.json`을 따른다.
`fresh-wasm-producer.json`의 초기화 pending은 빌드 시점이며 이후 실제 응답 결과는 validation에 있다.
Native294쪽은98% 미만이라 새 절대 px 핀을 추가하지 않는다. 정확한 기존 관계 검사를 유지한다.

[Native review](after-native-p294-review.png) · [Native overlay](after-native-p294-overlay.png)

[WASM review](after-wasm-p294-review.png) · [WASM overlay](after-wasm-p294-overlay.png)
