# PR #7027 검토: snapshot 예산의 단일 출처와 undo/redo

## 판정: 승인

- 원 PR: https://github.com/edwardkim/rhwp/pull/7027
- 기여자: `lpaiu-cs`; 관련 이슈: [#7020](https://github.com/edwardkim/rhwp/issues/7020).
- 최초 source head `1ca76c2f780da8b64cfd0054b180a5a427998495`, 현재 head `baa804c1e23f29b8ff0939965e60fedd44fc5fac`.
- 로컬 `1266ad650`. 현재 원격 커밋과 `git range-diff`에서 패치 동일(`=`); 중복 체리픽하지 않았다.

## 본문 계약과 초기 보류 사유

Rust `DocumentCore::MAX_SNAPSHOTS`를 저장소와 WASM `snapshotCapacity()`의 단일 출처로
사용하고, Studio 예산은 capacity에서 headroom 2를 뺀 값으로 계산한다. 이전 WASM에 API가
없으면 기존 capacity 100 fallback을 유지한다.

기존 source guard 제거와 상수 문자열 검사만으로는 새 예산을 적용한 실제 undo/redo, 축출,
오류 복원 동작을 입증하지 못했다. 이 검증 공백 때문에 최초 리뷰에서 머지를 보류했다.

## 메인터너 보완과 재검증

[동작 회귀 테스트](../../../rhwp-studio/tests/review-runtime-contracts.test.ts)를 추가해 실제
`CommandHistory`·`SnapshotCommand`·bridge getter를 실행했다. 입력 100·50·API 부재로 예산
98·48·98을 각각 적용하고, 예산을 넘는 편집, 오래된 snapshot 해제, 연속 undo/redo, 오류
rollback, undo 후 새 편집의 redo 폐기를 확인했다. 저장소 대역의 성공을 실제 WASM 성공으로
바꿔 적지 않았다. 별도 Chrome에서는 새로 빌드한 실제 WASM의 capacity 100과 텍스트 수정
전후 snapshot 복원을 확인했다.

메모리 안에서 예산 조회를 고정값 100으로 돌린 음성 대조가 실패함을 확인해, 같은 회귀가
재도입되면 테스트가 막도록 했다. 첫 undo의 after snapshot 확보로 오래된 항목 하나가 더
해제되는 기존 정책을 반영해 테스트 기대값을 정정했으며 제품 예산·baseline은 완화하지 않았다.

새 동작 테스트 묶음 6개, Studio 1,659개가 통과했다(Studio skip 2개). 통합 Rust 9,473개
통과·46개 skip, TypeScript·fmt·Clippy·workspace/WASM 빌드도 통과했다.
최신 source head CI rollup은 `SUCCESS`다. 초기 보류 사유는 해소됐고 별도 제품 코드 보정은
필요하지 않았다. 자세한 실행 결과는 [통합 기록](pr_7024_review_impl.md)에 있다.

## 원 PR/이슈 후속 기록 계획

merge 후 #7027·#7020에 merge SHA, 실제 CI, 이 리뷰와 runtime test를 연결한다. 화면 변화가
아닌 history 계약이므로 불필요한 PNG/PDF를 만들지 않는다. 기존 comment가 있으면 수정하고,
closing reference가 없는 #7020의 종료 여부는 실제 API로 확인한다. 통합 PR·merge·후속 원격
처리는 아직 수행하지 않았다.
