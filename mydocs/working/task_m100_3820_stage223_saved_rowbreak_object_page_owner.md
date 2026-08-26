# Stage 223: 저장 RowBreak object의 physical page 소유권

## 목적

`issue2006/1790387_prep_final_report.hwpx` section 3 `pi=336`의 마지막 54.8px 행이
별도 tail page를 만드는 문제를 저장 object-frame 계약으로 해결한다.

## 관찰

- `pi=336`은 20행×6열 일반 `TopAndBottom` RowBreak 표다. inline 표가 아니다.
- host LineSeg의 source top은 149.08px, common object bottom은 925.99px이고 body는
  926.01px이다. HWP 2020 PDF에서는 이 표 전체가 source 105쪽에 있다.
- 다음 host `pi=337`의 비합성 LineSeg는 `vpos=0`으로 새 physical page를 명시한다.
- `pi=304`는 비슷한 RowBreak 표지만 셀 내부에 저장 vpos reset이 있어 common height가
  첫 fragment만 소유한다. 따라서 `pi=336`의 object contract를 일반 적용하면 안 된다.

## 수정

다행 non-inline RowBreak 표가 빈 단일-control host, 표 자체 각주 없음, 음수 아닌 vertical
offset, 다음 host의 source 새-page reset, cell 내부 저장 reset 없음, 단일 비합성 host LineSeg,
source top보다 뒤의 cursor, source object bottom의 현재 body 내 포함을 모두 만족할 때만 object
frame을 current page owner로 채택한다. 조건이 성립하면 cursor와 placement 시작을 source top으로
복원하고 object frame 높이만큼 advance한다.

## 안전 경계

- cell 내부 reset 또는 다음 host가 새 page를 명시하지 않은 표는 기존 row scanner를 사용한다.
- inline 표, 다중 control/text host, 각주 표, 음수 offset, 합성 LineSeg, body 밖 frame은 제외한다.
- px allowance나 페이지 수 핀 변경은 없다.

## 검증 계획

1. `pi=336` 20행이 HWP 2020 source 105쪽 한 object frame으로 남고 continuation이 없어지는지 확인한다.
2. `pi=304` continuation과 `pi=330` tail은 별도 source owner stage에서 계속 분석한다.

## 검증 결과

`rhwp dump-pages`에서 문서 전체는 143쪽에서 142쪽으로 줄었고, `pi=336`은
`Table 20x6 642.5x776.9px` 하나로 source 105쪽 대응 frame에 남았다. 기존
143쪽 regression pin은 최종 MCP 2020 기준 140쪽을 도달한 뒤 한 번만 갱신한다.
