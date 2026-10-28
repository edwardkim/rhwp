# task_m100_3695 처리결과 보고서 — export-structure auto 선택 정책

- **Issue**: [#3695](https://github.com/edwardkim/rhwp/issues/3695)
- **상위 이슈**: [#1528](https://github.com/edwardkim/rhwp/issues/1528)
- **선행 작업**: [#3693](https://github.com/edwardkim/rhwp/issues/3693), [PR #3715](https://github.com/edwardkim/rhwp/pull/3715), merge commit `fe9749d542f46643e408c23878229c326e341363`
- **브랜치**: `codex/issue-3695-export-structure-auto`
- **Draft PR**: [#3749](https://github.com/edwardkim/rhwp/pull/3749)
- **상태**: 최신 `devel` `3d4863a0d` 통합, PR 리뷰 confidence 보정·전체 검증 완료, 보정 head CI 대기

## 0. 절차 복구 결과

초기 파일은 #3693의 승인된 단계 완료와 #3695 계획 승인 전에 구현 커밋과 함께 작성돼 WIP
스냅샷으로 재분류했다. 이후 #3693 선행 게이트를 완료하고 #3695 정정 계획 승인 체크포인트
`3de8b1709`을 고정한 뒤, `task_m100_3695_stage2.md`에서 기존 결론을 전제로 삼지 않은 독립 대조와
focused 재검증을 수행했다. 작업지시자가 그 결과를 검토해 `8343c98c6` 채택을 승인했으므로 이 문서를
#3695 최종 결과보고서로 확정한다.

## 1. 결과

`export-structure --mode auto`의 문서 단위 증거 우선순위를 명시적으로 정의했다.

- 명시적 `HeadType::Outline`은 authoritative outline이다.
- Outline이 없고 Number와 충돌할 때 confidence를 통과한 `조` 제목은 clause를 선택한다.
- 편·장·절·관, 목차 쪽번호, 조사형 상호참조는 Number를 뒤집는 독립 증거로 쓰지 않는다.
- strong clause marker가 없으면 단일·복수 Number 문서 모두 기존처럼 outline을 선택한다.
- 증거가 없으면 기존과 같이 clause로 폴백한다.

따라서 일반 자동번호 문단 하나 때문에 조문 구조가 전부 사라지는 결함을 막으면서, Number 하나만 쓰는
실제 개요 문서도 보존한다.

PR review `4838218628`에서 초기 정책이 정부 연구보고서의 목차·본문 절을 clause로 오분류하고,
`제3조의 규정에 따라` 같은 본문 상호참조도 문서 전체를 뒤집는다는 High 두 건이 확인됐다. 보정 뒤
Number와 충돌하는 편·장·절·관은 보고서 container일 수 있으므로 독립 증거에서 제외했고, 제목형 `조`도
쪽번호 tail과 조사형 상호참조 confidence를 통과해야 한다.

## 2. 호환성

- explicit `--mode outline|clause` 경로는 불변이다.
- `StructureDoc`/`StructureNode`와 CLI JSON 봉투의 필드·shape는 불변이다.
- effective `mode` 값은 계속 `outline|clause` 중 하나이며 동일 입력에 결정적이다.
- #3693의 clause marker·stack 문맥 정책은 그대로 사용한다.
- #3744의 explicit clause 문맥 만료·날짜·목 confidence 범위는 그대로 남긴다.
- 파서·렌더·레이아웃·직렬화 변경은 없어 시각 검증 대상이 아니다.

## 3. 변경 파일

- `src/document_core/queries/structure.rs`
  - `select_auto_mode()`와 증거 우선순위
- `tests/issue_3695_structure_auto_policy.rs`
  - synthetic 혼합/pure/explicit 경계
  - 실제 Outline·단일/복수 Number sample 회귀
- `mydocs/manual/cli_commands.md`
  - auto 선택 정책 설명
- `mydocs/plans/task_m100_3695*.md`, `mydocs/working/task_m100_3695_stage1.md`
- `mydocs/working/task_m100_3695_stage2.md`, `mydocs/working/task_m100_3695_stage3.md`
- `mydocs/orders/20260801.md`, `mydocs/orders/20260802.md`

## 4. 검증 결과

초기 구현·Stage 3 검증 결과는 다음과 같다.

- structure 단위: 6 passed
- #3695 auto 정책 통합: 8 passed
- #3693 clause 실문서 회귀: 3 passed
- export-structure CLI JSON 계약: 4 passed
- 전체 release-test(`fe9749d54` 통합 트리): 406 test binaries, 4,480 passed / 0 failed / 26 ignored
- fmt, clippy `-D warnings`, diff check: 통과

Stage 5 리뷰 보정 red는 9 passed / 3 failed(시장구조조사, 조사형 상호참조, 쪽번호 tail)였고, 구현 뒤
#3695 13건 전체가 통과했다. 최신 `devel` `3d4863a0d` 결합 트리에서 structure 6건, #3693 3건,
CLI JSON 4건, 전체 `cargo test --profile release-test --tests` 최종 exit 0, fmt, diff check,
clippy `-D warnings`를 순차 통과했다.

기존 devel auto 정책과 보정 결과를 같은 parse 결과에서 비교한 영향표는 다음과 같다. 암호 sample 3건은
password 없이 parse할 수 없어 분모와 별도로 적었다.

| 범위 | 후보 | parse 성공 | parse 실패 | mode 변화 | node_count 변화 |
| --- | ---: | ---: | ---: | ---: | ---: |
| `samples/` top-level | 351 | 348 | 3 | 0 | 0 |
| `samples/` 재귀 | 668 | 665 | 3 | 0 | 0 |

보정 전 PR에서 직접 재현된 `outline 3 → clause 51` 시장구조조사 회귀는 다시 outline 3으로 복구됐다.

모든 Cargo 실행은 `CARGO_INCREMENTAL=0`으로 순차 수행했다. 초기 Stage 2 신규 테스트의 red 기준은
5 passed / 1 failed였고, 당시 selector 적용 뒤 8건 전체가 통과했다.

Stage 2에서 같은 focused 게이트를 다시 실행해 모두 통과했고, `8343c98c6..HEAD`의 구현 파일 차이가
0임을 확인했다. 계획 대비 12개 항목도 모두 충족으로 판정했다.

Stage 3에서 `upstream/devel` `fe9749d54`를 WIP 감사 이력을 유지하는 merge 방식으로 통합했다. 소스
충돌은 없었고, 오늘할일 문서 한 곳만 양쪽의 2026-08-01 종료 시점 기록을 보존해 해결했다. 최신 devel
대비 net diff는 기존 #3695의 9개 파일·593 insertions/19 deletions 범위와 일치하며, #3715의 clause
marker·문맥 테스트와 #3695 selector 테스트가 focused·전체 프로필에서 함께 통과했다.

draft PR #3749 생성 직후 `devel`이 PR #3742 merge로 `cc3829116`까지 전진해 GitHub가 conflict를
보고했다. 최신 base를 다시 merge한 결과 소스 충돌은 없었고, add/add 오늘할일 문서만 양쪽 내용을
보존해 해결했다. 이 최신 결합 트리에서 structure 6건, #3695 8건, #3693 3건, CLI JSON 4건과 fmt,
diff check, clippy를 다시 통과했다. 전체 release-test는 직전 `fe9749d54` 통합 트리의 결과이고,
`cc3829116` 자체는 PR #3742의 full CI를 통과했으며 최종 결합은 PR #3749 CI에서 다시 검증한다.

## 5. 호환성과 남은 작업

- `8343c98c6`은 작업지시자 승인으로 #3695 구현에 채택됐다.
- PR 전 full release-test는 최신 `devel` 통합 트리에서 완료했다.
- draft PR #3749를 생성했고 최신 base 동기화 head의 GitHub CI·리뷰를 기다린다.
- PR review High 2건, 실문서·코퍼스·review 문서 누락을 보정했고 최신 head CI를 다시 확인한다.
- #3695를 merge해도 후속 #3744와 최종 통합 검증 전에는 상위 #1528을 close하지 않는다.
