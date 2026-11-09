# PR #8649 통합 적용·메인터너 보정 기록

## 최종 source와 재사용 범위

제출 code head `2e262d0d4b2885b130e2db917b86e894710425e4`, base `e00f9b7e26e581896fa17eaff3c8eb3f4889f9d4`입니다. 아래 전체 회귀·전쪽 수치는 원 실측 HEAD의 증거를 재사용한 기록입니다. 최신 집중 검사와 페이지별 producer, 엄격 0.1%p 잔여 한 건은 [재사용·최종 판정](../assets/planet6897_20261106/final-b768bee1-20261109/validation-reuse.md)을 따릅니다. 보호 TSV 표시값 하락은 0입니다.

## 최종 구현·검증 결과

검증 code head `eb876d1416fb558c874f9efec65c5722a6769751`, base `b768bee1c4c0023087687f79a0a95f12108743e9`에서 원 기여와 메인터너 보정을 함께 검증했습니다. 최소 글꼴 전체 회귀 13,105 PASS / 0 FAIL / 45 skip, Native 전용 73 PASS이며, Native/fresh WASM 각각 94조건·2,995쪽의 미달·누락·보호 표시값 하락은 없습니다. 원 PR의 개별 입력·페이지 판독과 보정 내용은 [최종 review](pr_8649_review.md)에 연결합니다. 원격 통합 PR의 최종 CI와 병합은 별도 단계입니다.

## 단계별 기록의 시점

아래 R 단계와 과거 실행의 진행 중·실패·추가 검증 표현은 해당 시점의 원인 추적 기록입니다. 과거 결과를 현재 code head의 통과로 합산하지 않으며, 현재 상태는 위 최종 결과와 연결된 review를 기준으로 판단합니다.

## 대상과 단계

[원 PR #8649](https://github.com/edwardkim/rhwp/pull/8649)의 원 head는 `c546460f55cd9cf94d8106cde271750e0a131c0c`다.
[공통 계획](../assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 적용/보존 결과 | 제목 |
| --- | --- | --- |
| `5fdbd2b17faef122ada2f7aecedda5e3894678f3` | `f326ce55e6298802c2a819f7edbbd9ed1f8d5390` | fix(layout): 칸 안의 쪽 기준 글 앞으로 표를 칸 흐름에 쌓지 않는다 (#8648) |
| `c546460f55cd9cf94d8106cde271750e0a131c0c` | `a32c708529618d66a4e574c55865fcb5921a72c2` | test(#8648): 축소본·코퍼스 Native·fresh WASM TSV와 대표 PNG |

## 최종 리베이스 후 적용 상태

검증 base `b768bee1c4c0023087687f79a0a95f12108743e9`, 검증 제품 head
`eb876d1416fb558c874f9efec65c5722a6769751`의 실제 Git 이력을 대조해 위 적용 SHA를 갱신했다.
[고유 56개 최종 적용표](../assets/planet6897_20261106/contributor-commit-map-final.tsv)는 원 기여 commit의
cherry-picked footer와 현재 적용 commit을 일대일로 확인한 결과다. 중복 patch·bridge의 기존 처리는 유지한다.
이번 b768bee1 리베이스에는 실제 파일 충돌이 없었습니다. index.lock 경합이 끝나고 lock이 없는 상태에서 계속했으며 lock을 삭제하거나 todo를 바꾸지 않았습니다. 제품 src/crates/tests/scripts와 지침의 바이트는 이전 후보와 같고 의존성 변경에 대한 새 검증을 수행합니다.
이력 확인과 현재 제품의 기능·회귀·시각 검증 판정은 구분하며, 필수 검증 완료 전에는 승인하지 않는다.

## 완료 조건

충돌·컴파일 보정 → 원인별 전체/집중 회귀 → 같은 원본·독립 Print Native/fresh WASM 전쪽 TSV와 직접 확대 판독 → 필수 lint/실제 backend → 개별 최종 리뷰 순서다.
실행하지 않은 항목은 통과로 쓰지 않고, 보정 코드는 통합 브랜치의 일반 커밋으로 남긴다.
