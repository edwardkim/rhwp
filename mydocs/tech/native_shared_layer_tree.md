---
kind: reference
status: active
canonical: mydocs/tech/native_shared_layer_tree.md
last_verified: 2026-11-06
---

# Native layer tree 공유 조회

`DocumentCore::build_page_layer_tree_shared_with_profile(page, profile)`는
기존 페이지 layer tree 캐시의 `Arc<PageLayerTree>`를 반환한다. 같은 페이지·출력 옵션·
profile을 다시 요청하면 node, text, image, SVG payload를 깊게 복제하지 않는다.
Native의 전체 페이지 snapshot 준비와 고정 타일 렌더링은 이 공유 조회를 사용한다.
별도 캐시나 새로운 레이아웃 세대를 만들지 않는다.

기존 `build_page_layer_tree`, `build_page_layer_tree_with_profile` 및 Canvas 전용
소유값 API는 독립적으로 수정할 수 있는 `PageLayerTree`를 계속 반환한다. portable
metrics 검사, 출력 옵션 지문, 페이지당 최근 변형 4개 제한과 기존 무효화 지점은 유지한다.
브라우저 Canvas 전용 입구가 허용한 metrics 상태를 일반 공유 입구에 전파하지 않는다.

호출자가 보유한 공유 tree는 문서 편집·교체 후에도 과거 snapshot으로 유효하다.
내부 캐시에서 제거돼도 외부 `Arc`를 강제로 회수하지 않으므로, 호스트가 장기간 보유한
tree의 메모리는 호스트 책임이다. `Arc::make_mut`은 공유 중인 tree를 복제한 뒤 수정한다.

`tests/cases/issue_536_shared_layer_tree.rs`는 warm 조회 재사용, 소유값 API 수정의
독립성, Screen/Print 분리, 문서 교체 시 과거 snapshot 보존과 새 기하, 잘못된 페이지
요청의 캐시 보존을 검사한다. 실제 앱, wall-clock 개선률, 최종 한컴·실물 OS/GPU
일치 검증은 이 계약의 범위에 포함하지 않는다.
