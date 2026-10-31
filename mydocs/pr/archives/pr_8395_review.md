# PR #8395 리뷰 — chrisryugj 탭·그룹 데이터 보존 통합

검토 head: `298d1ecc520014c2d18cf8c9016471f586967c03`.
비교 base: `55f40ecd99485a1c9b1f7cdc709cb6efcb9e9b29`.

## 최종 판정

**승인.** 동일 head의 로컬 필수 검증·GitHub CI·self-review를 완료하고 작업지시자 승인에 따라 머지했다. 추가 차단 결함은 발견하지 않았다.

## 확인한 내용

- 두 원 PR head가 통합 head의 실제 조상이며 원 기여 커밋을 재작성하지 않았다. 최신 base와 merge-tree가 충돌 없이 일치한다.
- #8224: 실제 탭 순서와 확장 word의 대응을 삽입·삭제·분할·병합에서 보존한다. 기존 NBSP 복사 helper는 유지한다. 병합 배열 성장은 공통 Storage를 통하고 작은 예산 거부·정상 예산 보존을 실제 실행했다.
- #8233: 자식 SHAPE_COMPONENT에 속한 CTRL_DATA를 같은 레벨에 보존하고 최상위 수집 범위에서 제외한다. 불투명 payload를 실행·조판 해석하지 않는다. 세 원본에서 재직렬화·재저장과 실제 WASM 편집 후 저장을 확인했다.
- 조판 소비 경로: 탭 메타데이터의 기존 소비 경로를 유지하며 저장 줄 수용 조건이나 좌표 보정을 추가하지 않는다. 원본·편집본의 독립 Print, Native와 fresh WASM 전쪽 TSV·직접 확대 판독을 대조했다. same-input base 대비 하락 없음.
- 검증 입력·기준 PDF·테스트 소스가 검증 시 SHA-256과 최종 Git blob에서 일치한다. 그룹 자식 데이터의 비시각 검사와 탭의 시각 검사를 구분한다.
- 실제 DocumentCore 저장 왕복 회귀와 기존 400px 폭 핀을 대체한 문자 포함 관계가 전체 회귀에서 실행됐다. #6520의 기존 SVG 경로 관계 검사는 유지했다.
- 로컬 전체 12,207개·Native 4,064개·명시한 신규 sample 보안 20개와 전체 Rust lint가 통과했다. 0건 선택·중첩 worktree fmt 실패는 통과 근거에서 제외하고 올바른 경로 재실행 결과만 사용했다.
- #8225의 기존 최상위 CTRL_DATA 중복 표현 차이는 남는다. 자식 payload는 모두 보존되지만 이 이슈 전체 해결로 확대하지 않는다.

## 최종 통합

- [PR #8395](https://github.com/edwardkim/rhwp/pull/8395), merge `21ef5020f763e474ff652df5c2c9adf94efededc`.
- [Self-review 댓글](https://github.com/edwardkim/rhwp/pull/8395#issuecomment-6536743274).
- [Full CI](https://github.com/edwardkim/rhwp/actions/runs/41104868865)·CodeQL·Render Diff·Adapter·Proptest 성공. 실제 CodeQL 선택 언어는 Rust다. 다른 언어의 성공 job은 선택 제외 경로이며 해당 언어 분석을 실행했다는 뜻이 아니다.
- GitHub 단계별 API 기록 지연 후 승인받은 [정책 감사 attempt 2](https://github.com/edwardkim/rhwp/actions/runs/41108714192/attempts/2)가 성공했다. 원 CI를 재실행하거나 정책을 우회하지 않았다.
- 최신 base `55f40ecd99`와 충돌 없는 merge-tree `9cfc1e37e5b1d112e905fd9fb39ab6deb5ca56f3`를 확인하고 `--match-head-commit`으로 머지했다.
- [원 PR #8224 검토](pr_8224_review.md)·[원 PR #8233 검토](pr_8233_review.md), [정식 증거](../assets/chrisryugj_20261031/README.md).
- 관련 #8222는 완료, #8225는 OPEN이며 전체 해결로 확대하지 않는다.

후속 기록은 메인터너 운영 문서로 devel에 직접 반영한다. 검증 CI를 병합 뒤 다시 실행하지 않고
[duration 갱신](https://github.com/edwardkim/rhwp/actions/runs/41152214941)은 성공했다. 검증된 PR worker 측정으로 46개 target을 갱신하고 메타데이터 브랜치에 `fe3d55b99`를 게시했다. 원 PR별 댓글·작업 전용 산출물 정리를 이어서 수행한다.
