# PR #6216 구현 검토 기록

## 구현 단계

1. `trusted-postmerge-ci-reuse.yml`은 merge commit의 pre-PR `devel` parent만 checkout한다.
2. base parent에 존재하는 verifier가 merge SHA, parent 관계, PR 연관성, tree, changed paths, source workflow run을 판정한다.
3. 판정 결과가 `reuse=true`일 때만 각 workflow가 source PR의 green 결과를 재사용한다.
4. CI duration 갱신은 source PR의 B/C artifact와 provenance가 모두 유효할 때만 수행한다.

## 불변 조건

- PR head의 verifier 또는 workflow를 checkout하거나 실행하지 않는다.
- 검증 정보가 누락되거나 모순되면 성공으로 추정하지 않고 전체 CI를 수행한다.
- B/C target-duration 측정은 빈 map, 서로 다른 run, ref, SHA를 허용하지 않는다.
- 새 workflow 계약 테스트는 `ci.yml`의 `Validate workflow contracts` 단계에 명시적으로 배선한다.

## 롤백 경로

- 재사용 판정에 결함이 확인되면 네 호출 workflow에서 `trusted_postmerge_reuse` 의존성과 fast-pass 연결을 제거하면 이전의 post-merge 전체 CI 동작으로 되돌아간다.
- duration 수집 또는 승격 결함은 새 collector/refresh 검증을 제거하거나 정책 파일 갱신을 중단해 기존 고정 정책으로 복귀할 수 있다.

## 배포 관찰 항목

- #6216 자체는 verifier helper가 base에 없으므로 전체 CI fallback이어야 한다.
- merge 뒤 다음 적격 PR의 `devel` post-merge 실행에서 source run ID가 기록되고, 변경 경로가 안전할 때만 재사용되어야 한다.
- 재사용 불가 조건에서는 workflow가 성공을 위장하지 않고 정상 전체 검증 경로를 실행해야 한다.
