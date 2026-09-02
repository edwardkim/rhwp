# Stage 241: 전방 저장 TAC frame의 physical page owner

## 목표

HWPX TAC 표의 저장 frame을 무조건 현재 흐름 anchor로 제한해 생긴 `issue_2243`
6쪽 회귀를 해결하면서, Stage 238의 `aift.hwpx` overflow-cell 보호를 유지한다.

## 비교 증거

Stage 239 상태에서 두 HWPX 문서를 같은 진단으로 비교했다.

| 문서 | 현재 흐름 | 저장 bounds | 선언 frame | 본문 높이 | 판단 |
| --- | ---: | --- | ---: | ---: | --- |
| `36386907_gyeoljae_sewoon.hwpx` pi=39 | 135.4 | 170.36..926.44 | 752.32 | 930.5 | 현재 흐름보다 전방이며 겹치고 frame 하단이 본문 안이다. |
| `aift.hwpx` pi=911 | 816.3 | 0..904.75 | 901.01 | 971.4 | 저장 frame이 현재 흐름보다 뒤처진 쪽 상단 anchor다. |

upstream은 첫 형상을 4쪽에 유지해 전체 5쪽을 만들며, 두 번째 형상은 새 쪽으로
이월해 74쪽을 유지한다.

## 변경 계약

- 표 선언 frame과 저장 line 하단이 같은 본문 안에 있어야 한다.
- 기존처럼 저장 line이 현재 흐름 anchor와 일치하면 frame을 허용한다.
- 추가로 저장 frame의 시작이 현재 흐름보다 전방이면서 그 line이 현재 흐름과 겹치면,
  그 frame은 같은 physical page의 owner로 허용한다.
- 저장 frame이 현재 흐름보다 뒤처졌으면 stale page-top anchor이므로 허용하지 않는다.

이 계약은 source bounds와 object frame의 기하 관계만 사용하며 문서명, 페이지 번호,
임의 px allowance를 사용하지 않는다.

## 검증 대상

```sh
cargo test --profile release-test --lib saved_tac_table_flow_tail_contract
cargo test --profile release-test --test issue_2243
cargo test --profile release-test --test issue_2020
cargo test --profile release-test --test overflow_cell_baseline
cargo test --profile release-test --test issue_3820_rowbreak_rowspan_band
```

## 검증 결과

- `saved_tac_table_flow_tail_contract`: 2개 통과
- `issue_2243`: 1개 통과, 기대 페이지 수 5쪽 유지
- `issue_2020`: 4개 통과, 여권 문서 2쪽 유지
- `overflow_cell_baseline`: 1개 통과
- `issue_3820_rowbreak_rowspan_band`: 4개 통과
