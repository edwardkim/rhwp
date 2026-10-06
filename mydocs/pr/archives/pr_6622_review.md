# PR #6622 검토 - EMF ExtCreatePen 지원

- 원 PR head: `be2535ab913becdd6f63eb70d4f188f310544479`
- 통합 기준: `upstream/devel` `2edbe62e5dc74db58c33df2c006ae587f86a1a71`
- 중복 처리: #6625의 선행 커밋 `4c6e7ce`는 #6622 head와 동일 patch-id라 한 번만 적용했다.
- 검토자: `@jangster77` review request를 검토 시작 전에 등록하고 API로 확인함.

## 판정: 승인

`ExtCreatePen`을 parser/DC object로 연결한다. #6615--#6625 변환 스택에서 하나의 동일 패치를 이중 적용하지 않아 provenance와 실제 변경을 일치시켰다.

## 검증

- `issue_6577_emf_path_to_records`, `wmf_emf_goldens` 통과.
- 통합 후보에서 rustfmt, workspace clippy, release-test nextest 전체 종료 코드 `0`, Native Skia lib `3,959`건, WASM web build를 통과했다.

## 시각 증적

- 최종 EMF 스택 결과는 [원문 2쪽 도해](../assets/pr_6586_6649_planet6897_integration_20260902/review_6615_6625_issue6577_page_002.png)로 보존했다. 이 이미지는 #6622 단독 효과가 아니라 #6625까지의 누적 결과다.
- 해시는 [manifest](../assets/pr_6586_6649_planet6897_integration_20260902/manifest.sha256)에 있다.

원 PR은 직접 merge하지 않는다. 별도 승인 뒤 EMF 스택과 함께 통합 PR에서 수용한다.
