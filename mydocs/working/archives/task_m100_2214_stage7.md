# Task M100 #2214 Stage 7 완료보고 — normalized-state coherence

## 1. 확정 원인

#2195가 비-TAC 중첩 표 stretch를 `render_normalized` 복사본에 적용하면서 #2214 픽스처도
원본 IR과 별도의 렌더 문단을 사용하게 됐다. deferred 셀 편집은 원본 문단과 원본
`cell_units_cache`만 갱신해 normalized 복사본의 text와 pointer-key cache가 편집 전 상태로
남았다.

```text
원본 model/LINE_SEG = 174
render_normalized page tree = 130
explicit flush 뒤 normalized 재생성 = 174
```

## 2. 구현

- deferred 셀 편집 뒤 해당 normalized 상위 문단의 동일 cell paragraph만 최신 원본 문단으로
  교체한다.
- 상위 문단·소유 표·셀 주소는 보존해 unrelated/sibling cache identity를 유지한다.
- normalized edited cell에도 기존 #2214 owner-flag scoped invalidation 규칙을 적용한다.
- 새 문단 내부의 비-TAC 중첩 표에는 #2195 stretch를 다시 적용한다.
- immediate pagination 경로는 기존 pagination 재생성을 사용하며 별도 mirror를 수행하지 않는다.
- 전역 layout-cache clear와 매 입력 full pagination은 추가하지 않았다.

## 3. #2195와 통합된 경계 계약

44번째 입력은 상대 paragraph flow advance가 바뀌므로 `cellFlowChanged=true`와 pre-cursor
flush 1회를 유지한다. 다만 #2195의 선언 셀 높이가 첫 페이지 증가분을 흡수한다.

| 상태 | tree max | page 0 cut | bounds h |
|------|---------:|-----------:|---------:|
| deferred transient | 174 | 37 | 945.9 |
| explicit flush | 174 | 37 | 945.9 |

첫 페이지가 같다고 flush가 불필요한 것은 아니다. 전체 115개 fragment 비교에서 page 0~1은
같고 page 2~114의 continuation cut 113개가 flush 뒤 재정렬됐다. 따라서 첫 페이지 cut
`37→38` 고정값만 제거하고 downstream cut-chain 변화와 1회 flush 계약은 보존했다.

## 4. 집중 검증

| 검증 | 결과 |
|------|------|
| CI 실패 단일 테스트 | 1 passed / 0 failed, HWP/HWPX 각각 tree 174 |
| `cargo test --lib issue2214` | 9 passed / 0 failed |
| `cargo test --test issue_2214_page_local_repaint` | 3 passed / 0 failed |
| normalized cut-chain | HWP/HWPX 각각 115 fragment 연속, downstream 113개 재정렬 |

최신 `devel` 통합 뒤 release-test 전체, Clippy, fmt, Studio 게이트는 다음 통합 검증 단계에서
수행한다.
