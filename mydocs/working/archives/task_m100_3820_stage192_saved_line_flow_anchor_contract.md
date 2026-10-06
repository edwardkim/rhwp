# #3820 Stage 192: 각주 저장 줄 흐름 앵커 계약

## 관찰

전체 `release-test` 첫 관문에서
`saved_line_clears_footnote_area_requires_every_boundary`가 실패했다.
저장 줄의 각주 안전마진 예외가 일반 fragment의 넓은 겹침 범위를 그대로 사용해,
흐름 커서가 저장 줄 상단보다 16px 이상 아래인 경우까지 허용했다.

## 원인

일반 fragment 겹침은 줄 전체 범위 안의 커서를 찾는 목적이라 넓게 판정해야 한다.
반면 각주 안전마진 예외는 현재 흐름이 바로 그 저장 줄을 소비하는 경우에만
적용할 수 있다. 두 의미를 같은 helper로 처리해 저장 앵커 계약이 사라졌다.

## 변경

`saved_line_is_anchored_to_current_flow`를 분리했다. 저장 줄의 상단이 현재 흐름보다
앞서고, 흐름 커서가 그 상단에서 16px 이내일 때만 각주 안전마진 예외를 허용한다.
쪽 밖 좌표, 본문·각주 경계, 다단 여부의 기존 조건은 유지한다.

## 검증

`cargo test --profile release-test --lib renderer::typeset::tests::saved_line_clears_footnote_area_requires_every_boundary -- --exact`
가 통과해야 한다.
