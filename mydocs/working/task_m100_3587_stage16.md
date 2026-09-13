# #3587 Stage 16 — D0 통합 및 D1 원본/대상 분리 선검증

- 승인: 2026-09-13 「다음 절차 진행을 승인합니다」, [D 상세 계획](../plans/task_m100_3587_impl_d.md).
- 계획 보존: `7149d5455`. 최신 원격 `897c6a3d8`을 작업 브랜치에 통합한 커밋: `0ba817ee6`.
- 상태: **D1의 읽기 전용 선검증 구현 및 집중 검증 완료**. D1 전체 완료는 아니며 실제 자원 이식·블록 삽입은 아직 구현하지 않았다.

## D0 통합

`merge-tree`에서 `document_core/commands/mod.rs`, `document_core/mod.rs`의 같은 위치에 추가된
공개 선언 2곳이 충돌했다. 우리 `paragraph_block` 선언과 원격 `TableColumnWidths`·`TableCreationOptions`
선언을 모두 보존했다. 기능 중 하나를 선택하거나 제거하지 않았다.
`object_ops/table.rs`, `typeset.rs`, passthrough guard와 문서는 자동 병합됐으며 이후 검증 대상으로 둔다.
로컬 devel 자체는 이동하지 않고 승인된 작업 브랜치에만 merge했다. push는 수행하지 않았다.

## D1 선검증의 목적과 구현

기존 B 요청에는 원형 범위와 삽입 위치가 같은 문서라는 전제가 있다. 외부 문서를 가져올 때 이를
그대로 대상 문서에서 검사하면 원본의 범위가 정상인데도 거부하거나 대상의 같은 번호 서식을 검사하게 된다.

- `ImportParagraphBlockRequest`는 source/target 구역과 원본 범위·대상 경계를 분리한다.
- `inspect_paragraph_block_import_native`는 source를 불변 Document로 받는다. 검사하려고 별도 core나
  전체 source 문서를 복제하지 않는다.
- B의 예산·지원 컨트롤·내부 참조·도달 서식 검사 본체를 Document 기준 함수로 추출해 재사용한다.
  기존 B 공개 함수는 같은 본체를 호출한다. 다른 복제/조판 정책은 변경하지 않는다.
- 원본 문서와 대상 문서 노드의 합계가 요청 상한 안인지 확인한다. 대상 경계 및 최종 문단 수의
  덧셈은 checked arithmetic으로 검사한다. `count=0`은 주소·옵션 검사 후 구조와 자원 조회를 생략한다.
- 승인된 자원 상한 설정(준비 metadata 32MiB, binary 64MiB)은 낮출 수만 있게 타입에 명시했다.
  **이 절편은 실제 자원 바이트 양을 계측하지 않는다.** 자원 준비 완료를 뜻하는 결과는 반환하지 않으며
  결과의 `resourcesPrepared=false`를 명시한다. 실제 가져오기 허가증/완료된 dry-run으로 사용해서는 안 된다.

## 검증 설계

신규 integration source: `tests/cases/issue_3587_block_import_preflight.rs`.
다른 길이의 원본/대상, 숫자로 겹치는 범위, 잘못된 주소·상한, count=0, 원본 서식 공간 조회,
양쪽 합산 예산, lazy resolver 비호출, JSON 잘못된 키를 실제 공개 query로 검사한다.
기존 A/B/C·외부 붙여넣기·원격 표 생성·passthrough guard도 통합 SHA에서 확인한다.

## 검증 결과

- D0 통합 커밋 `0ba817ee6`: 기존 집중 검사 **142 PASS**.
- 선검증 구현 `4782a4ac0`, 테스트 필드명 정정 후 최종 제품·테스트 기준
  `1b3648aac803787f8dfb683c05c6376ee9fa670f`: 집중 검사 **156 PASS, 실패 0**.
  신규 선검증 9건과 원격 #5819 표 생성 5건을 포함한다.
- 동일 SHA의 review worktree에서 `--prepare` 후 manifest `--check` PASS,
  `cargo fmt --all -- --check` PASS.
- 집중 선택식은 `test(issue_3587) | test(foreign_paste) | test(issue_5819) | test(passthrough_invalidation)`이다.
  nextest의 `9573 skipped`는 이번 선택식에서 제외된 테스트 수이지 전체 회귀 통과 증적이 아니다.
- 신규 테스트의 최초 컴파일에서 `CharShapeRef.position`이라는 잘못된 필드명을 사용해 실패했다.
  실제 필드 `start_pos`로 정정했다. 정정 후 테스트는 통과했으나 파생 목록 재준비가 누락되어
  manifest 불일치가 검출됐다. 같은 최종 SHA에서 목록을 다시 준비하고 manifest·포맷·156건을
  모두 재실행해 통과했다. 제품 로직이나 기대값을 완화하지 않았다.
- nextest 설치 버전 0.9.137과 저장소 권장 0.9.140의 차이는 경고로 출력됐으며 실행은 성공했다.

로컬 증적: `output/3587/d0-focused.log`, `output/3587/d1-focused-r3.log`,
`output/3587/d1-fmt-r2.log`, `output/3587/d1-manifest-r2.log`.
review worktree는 `/home/edward/mygithub/rhwp-review-3587`, 공용 target은
`/home/edward/mygithub/rhwp/target/pr-review`를 사용했다. 파생 suite·manifest는 커밋하지 않았다.

**이번 SHA의 전체 회귀·세 Clippy 단계·Native Skia·WASM 빌드는 수행하지 않았다.**
이전 C 결과를 이번 코드의 전체 검증 결과로 재사용하지 않는다. D 통합 검증과 push/PR 직전
필수 게이트는 별도로 남아 있다.

## 남은 D1

선택 블록의 자원 집합 준비·번호 충돌 매핑·제한 바이너리 읽기·중복 재사용·신원 발급·단일 commit 삽입 및
저장 재열기는 아직 남았다. 그 뒤 D2 공개 실행 경로, D3 전체 통합 검증, 선택적 Gym으로 진행한다.
#7065·#7084·#7090은 이 변경에서 수정하지 않았다. Studio가 제공하는 WASM도 변경하지 않았다.
