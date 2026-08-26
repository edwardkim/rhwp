# 자기 라운드트립 ≠ 한컴 호환

이 명제는 스킬 전역 불변식이다. 테스트가 golden 을 통과하고,
`hwp5-roundtrip` / `hwpx-roundtrip` / `render-diff` 자기 비교가 PASS 여도
한컴이 같은 화면을 보여주거나 같은 파일을 연다는 뜻이 아니다.

## 세 층

| 층 | 명령 | 무엇을 말하나 | 말하지 않는 것 |
|---|---|---|---|
| 구조 보존 | hwpx-roundtrip, hwp5-roundtrip | 우리 IR↔직렬화가 닫힘 | 한컴 파서 호환 |
| 자기 시각 | render-diff (한 파일) | 직렬화 전후 bbox 변위 | 한컴 PDF 충실 |
| 한컴 계약 | hwp5-inventory-diff + 한컴 수동 | oracle record / 한컴 화면 | 우리 테스트 초록불 |

## 에이전트가 하면 안 되는 보고

- "라운드트립 통과했으니 한컴에서 열립니다"
- "render-diff PASS 이니 간격이 맞습니다"
- "IR identical 이니 저장본이 같습니다"

올바른 보고:

- "자기 직렬화는 닫혔다. 한컴 검증은 남아 있다."
- "IR 카테고리 X 가 N 건. 화면은 overlay 로 따로 봤다."
- "oracle/generated inventory 힌트는 TABLE 축. 한컴 열기 여부는 미확인."

## convert --verify

`--verify` 는 저장 후 재파싱 IR 과 어댑터 적용 후 IR 을 비교한다.
차이 시 산출물은 남기고 exit 3. 이것도 **자기** 검증이다.
`--verify-pages` 는 쪽수 비교, 불일치 시 exit 4. 한컴 쪽수가 아니다.

## render-diff 주의

매뉴얼 문구: 자기 roundtrip 통과 ≠ 한컴 충실. 내부 회귀 방지용.
한컴 PDF 기준은 `tools/fidelity_compare` 등 별도 경로이며 이 스킬의 기본 축이 아니다.

## 최종 게이트

저장·렌더 결함의 최종 게이트는 한컴 수동 검증이다.
이 스킬은 그 전에 기계로 좁힌다. 좁힌 결과를 최종 합격으로 승격하지 않는다.
