# PR #6616 검토 - EMF stock object 선택

- 원 PR head: `5dd71047d9dd6518cfc4787312d385f4f3e8b548`
- 통합 기준: `upstream/devel` `2edbe62e5dc74db58c33df2c006ae587f86a1a71`
- 후속 관계: #6615의 경로 파싱 뒤 #6622, #6625와 같은 #6577 스택으로 적용했다.
- 검토자: `@jangster77` review request를 검토 시작 전에 등록하고 API로 확인함.

## 판정: 승인

stock pen/brush object를 DC에 명시적으로 매핑한다. 일반 object-id와 혼동하지 않도록 stock-object 분기를 한정했고, 후속 도형 방출에서만 효과가 난다.

## 검증

- `issue_6577_emf_path_to_records`, `wmf_emf_goldens` 통과.
- 통합 후보에서 rustfmt, workspace clippy, release-test nextest 전체 종료 코드 `0`, Native Skia lib `3,959`건, WASM web build를 통과했다.

## 시각 증적

- 독립 문서 배선은 아직 없으므로 #6616 단독 화면 결과를 주장하지 않는다.
- #6615--#6625 스택의 최종 원문 2쪽 도해는 [이 직접 출력](../assets/pr_6586_6649_planet6897_integration_20260902/review_6615_6625_issue6577_page_002.png)으로 보존했으며, 파일 해시는 [manifest](../assets/pr_6586_6649_planet6897_integration_20260902/manifest.sha256)에 있다.

원 PR은 직접 merge하지 않는다. 별도 승인 뒤 EMF 스택과 함께 통합 PR에서 수용한다.
