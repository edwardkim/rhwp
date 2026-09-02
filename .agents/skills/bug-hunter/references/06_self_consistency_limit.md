# 06 — 독립 기준이 없을 때

playbook: "비교 기준이 없으면 render-diff 같은 자기 일관성 검사까지만
수행하고 그 한계를 기록한다."

이것은 실패가 아니다. 범위를 정직하게 줄이는 것이다. 한계를 적지
않고 "레이아웃이 맞다"고 쓰면 거짓이다.

## 허용되는 검사

```bash
rhwp render-diff <파일> --via hwpx
rhwp render-diff <파일> --via hwp
rhwp render-diff <파일> <파일>
rhwp ir-diff <파일> <파일> --json
```

`render-diff A A` 는 항상 PASS 여야 한다. 아니면 도구 비결정성이고
문서 결함보다 먼저다. 그래도 **한컴과 같다**는 뜻이 아니다.

## 금지되는 승격

- 자기 라운드트립 PASS → "한컴 충실"
- 자기 라운드트립 OVER → "한컴과 어긋남"
- 정답지 없는 STRUCT_MISMATCH → 충실도 이슈

자기 일관성 실패는 "rhwp 가 rhwp 와 다르다"는 내부 회귀 후보다.
이웃 `rhwp-visual-regression` 으로 인계할 수 있다. 이 스킬의 한컴
대조 이슈 템플릿으로는 올리지 않는다 (F04).

## 기록 문장 (그대로 쓴다)

```
독립 비교 기준을 확보하지 못했다.
수행: rhwp render-diff <경로> --via hwpx
결과: <status>
한계: 자기 일관성만. 한컴 공식 출력·법정 서식·제출 요건과
대조하지 않았다. 충실도 결함으로 이슈화하지 않는다.
```

예제: [14_no_baseline.md](../examples/14_no_baseline.md)
전사: `fixtures/transcripts/self_only_limit.txt`

## 기준이 나중에 생기면

같은 입력에 한컴 PDF 가 도착하면 provenance 를 채우고
`fidelity_compare` 를 **처음부터** 돈다. 예전 render-diff 로그를
첨부 근거로 재사용하지 않는다.
