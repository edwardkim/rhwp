# PR #6615 검토 - EMF 경로 레코드 변환 기반

- 원 PR head: `72af78546e90222175961199925765aaa753b46c`
- 통합 기준: `upstream/devel` `2edbe62e5dc74db58c33df2c006ae587f86a1a71`
- 후속 관계: #6616, #6622, #6625와 하나의 #6577 EMF 변환 스택으로 적용했다.
- 검토자: `@jangster77` review request를 검토 시작 전에 등록하고 API로 확인함.

## 판정: 승인

`POLYLINE_TO16`과 `POLYBEZIER_TO16` 파싱은 후속 DC 상태, 펜, clip, world-transform 출력의 전제다. 아직 내장 EMF 자동 배선은 켜지지 않아 범위를 변환기 층으로 유지한다.

## 검증

- `issue_6577_emf_path_to_records`, `wmf_emf_goldens` 통과.
- 통합 후보에서 rustfmt, workspace clippy, release-test nextest 전체 종료 코드 `0`, Native Skia lib `3,959`건, WASM web build를 통과했다.

## 시각 증적

- 스택 최종 후보의 원문 2쪽 도해: [page 002](../assets/pr_6586_6649_planet6897_integration_20260902/review_6615_6625_issue6577_page_002.png)
- 이 이미지는 #6625까지 적용한 최종 변환 결과이며 #6615 단독 출력이라고 주장하지 않는다. 해시는 [manifest](../assets/pr_6586_6649_planet6897_integration_20260902/manifest.sha256)에 있다.

원 PR은 직접 merge하지 않는다. 별도 승인 뒤 후속 EMF 스택과 함께 통합 PR에서 수용한다.
