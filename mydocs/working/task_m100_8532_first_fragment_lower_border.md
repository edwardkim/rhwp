# 이슈 #8532 — 앞 조각의 표 아래 외곽 높이 대조

2026-11-05 KST. PR #8547을 포함한 기존 39개 PR 통합 검토 범위다. 새 PR을 접수하거나
새 이슈를 등록하지 않았다. 현재는 원인 분석 단계이며 구현·기대값을 바꾸지 않았다.

## 독립 관측과 원 해결 PR

[원 PR #8547](https://github.com/edwardkim/rhwp/pull/8547)의 원 head는
`802078a02454a66ef3dc917d6ba2c6461709dcd5`, 제품 커밋은
`38ed5b5c4a4fc8e1a11dbdad2071c71302c7f531`이다. 현재 적용 제품 커밋은
`a6f332bf20a5baaaae5e2b430bd7a5414f7ddafb`다. 원 PR 전체 본문·diff를 다시 읽었다.
원 작성자는 3쪽의 남은 차이가 표 아래 경계임을 이미 공개했다. 현재 통합에서 새로
발생한 차이라고 단정하지 않는다. 4~6쪽이 뒤로 밀리는 문제를 해결한 범위와 구분한다.

같은 HWP·독립 한컴 Print의 기존 Native/fresh WASM 3·4쪽 review와 overlay를 직접
대조했다. 3쪽 위생 앞 조각과 4쪽 이어받는 내용의 소유·순서는 맞지만 3쪽 마지막 행의
아래 외곽은 다르다. 기존 96 DPI 원본 PNG의 마지막 두 긴 가로선은 Native y=996/1013,
Print y=996/1025다. 아래 선만 12 CSSpx 차이가 난다. 이 값은 진단 관측이며 98% 미만인
3쪽에 새 절대 px 회귀를 추가한 것이 아니다.

[원본 레코드·페이지 항목·PNG 관측](../pr/assets/planet6897_20261103/pr_8547/boundary-diagnostic/observations.json)을
같이 보존한다. 입력 SHA는 `797268d10fc7605ae5e0e299e9fb386ea6c204f35ef4c4879bb611e66ef4545a`다.
원본 CFB를 직접 해제해 읽었으며 엔진의 HWPX 재저장본을 근거로 사용하지 않았다.

## 원본 저장값과 현재 소비 경로

원본의 0기준 38행은 col 0이 rowspan 2·높이 6126HU, 나머지 네 칸은 rowspan 1·높이
3063HU다. col 3/4는 위생사·영양사 두 문단의 저장 vpos가 모두 0이며, 각 줄 높이는
1000HU·줄간격 500HU다. col 1은 위생 한 줄·간격 400HU다. 패딩 원값은 네 방향
141HU이고, 개별 여백 지정 여부와 표 기본 패딩의 적용을 구분해야 한다.

최신 제품의 `dump-pages --json --compat 2022 -p 2`는 실제 3쪽을 반환했다. partialTable은
startRow=21, endRow=39, startCut=[1,1,1,1], endCut=[1,1,1,1,1,0,0,0,0]다.
페이지 usedHeight는 920.5733333333335px, 본문 높이는 933.5333333333334px다.
기존 캡처 render tree에서 38행의 다섯 Cell은 y=998.0·height=17.1px로 기록돼 있다.

생산·소비 연결은 다음 순서로 계속 대조한다.

1. `BlockCutQuery::allows_split`의 저장 되감김 수용: 내용 컷을 받는 조건이다. 물리 밴드의
   최종 높이를 이 조건 하나에서 정하지 않는다.
2. `RowBlockQuery::row_cut_base_height` → `row_block_cut_physical_height` 또는
   `row_block_cut_row_content_height` → `fragment_extent`의 요구·예약 높이.
3. `layout/table_partial.rs`의 끝 행 override·행별 높이와 최종 셀/외곽 배치. 같은 helper
   호출 뒤 다른 분기가 값을 덮어쓰는지도 추적해야 한다.
4. 4쪽의 이어받는 내용·기존 100% 출력 보존. 앞 조각의 비어 있는 물리 밴드를 내용 소비로
   차감하면 다음 조각이 줄거나 소유가 달라질 수 있으므로 확인 없이 보정하지 않는다.

원본 높이 3063HU를 그대로 강제하거나 남은 본문 높이까지 무조건 늘리는 방식은 아직
독립 계약으로 입증되지 않았다. 실제 물리 프레임 규칙과 정상 대조군을 확인한 뒤 공통
생산 결과로 수정하고, 측정·예약·배치가 같은 결과를 소비하도록 해야 한다. 점수 통과나
작성자가 남은 차이를 공개했다는 사실만으로 해결 완료를 선언하지 않는다.
