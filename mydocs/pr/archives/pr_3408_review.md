# PR #3408 검토 기록 — 271건 문서 아카이브 대장화 사례

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3408](https://github.com/edwardkim/rhwp/pull/3408) — `docs(report): 문서 아카이브 대장화 CLI 작동 사례 — 파일 더미 → batch → 대장` |
| 작성자·검토자 | `@kevin9327` (external contributor) · `@jangster77` (collaborator) |
| base / source head | `devel` / `db6ef6b0fa9503092424202d4a14a67b087b2e43` (작성 시점 참고값) |
| 작성 시점 상태 | `MERGEABLE`, `BEHIND`, draft 아님. merge 전 최신 상태 재확인 필요 |
| 원 변경 규모 | 4 files, +327 / -0; README, PNG 2개, TSV 271행 |
| 관련 이슈 | [#3407](https://github.com/edwardkim/rhwp/issues/3407) 참고. 이 PR이 close하는 이슈는 없음 |
| 통합 기준 | `review/kevin9327-20260726-v2`; 최초 `upstream/devel` `732147a30c`, 최신 동기화 `7f8fcfef0`; 원 commit `db6ef6b0` → 통합 commit `4cc1c80d1` |
| 메인터너 보정 | 통합 commit `a1fe4ce760899f4ad0b12bc5fbddf808611e9dd5` 중 #3408 관련 README·TSV 보정 |
| 라우팅 | base route: `collaborator_external_pr.md`; modifiers: `intake_and_review.md`, `local_validation.md`, `multi_pr_update_branch.md`, `review_only_fast_pass.md` |

Loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 라우팅 문서.

## 원 변경과 메인터너 보정 구분

Contributor 원 변경은 `rhwp batch info --json`과 개별 `rhwp info`를 비교한 대량 문서 대장화 사례,
대표 화면 2개, 271행 TSV를 추가한다. 코드·renderer·fixture는 바꾸지 않는다.

원본 TSV에는 `파일명·포맷·쪽수·문단수·용량·폰트수`만 있고, README와 이미지가 핵심 산출물로 소개한
“문서 제목” 열은 실제 271행 TSV에 없었다. 또한 제목 추출 예시가 첫 페이지만 읽어 표지 이미지 문서를
놓칠 수 있었다. collaborator 메인터너 보정 `a1fe4ce76`은 다음만 추가·정합화했다.

- 각 문서 전체 페이지의 첫 비어 있지 않은 줄을 `export-text --json`과 명시된 `jq` 규칙으로 추출해
  제목을 포함한 실제 8열 TSV 계약에 반영했다.
- README를 `batch info` 메타데이터와 `export-text` 제목을 결합하는 **2-pass** 흐름으로 고쳤다.
- 이미지의 사람용 파일명 `문서대장_271.tsv`가 저장소의 `document_register_271.tsv`를 가리킨다고
  명시했다.

보정 뒤 TSV는 header 포함 272줄, data 271행이고 모든 행이 8필드다. 재집계 결과는
`rows=271`, `pages=5706`, `sizeKB=188717`, `hwp5=255`, `hwp3=15`, `hwpx=1`, 검증기의
`badTitles=0`이다. 추출 실패나 빈 필드를 조용히 숨기지 않으며, 텍스트가 없는 문서는
`(제목 없음)`이라는 명시적 fallback으로 남긴다.

## 증적자료

불투명한 파일명에서 내용 제목을 얻는 전후 흐름(`1030×518`, SHA-256
`e85a1d2d61956ee5560cc3ad307e626da38dee7975adbd9590defdc6544d5495`):

![PR #3408 파일 더미에서 batch 대장으로 전환한 흐름](../../report/archive_register_demo/daejang-before-after.png)

제목 열을 포함한 대표 16행 대장(`754×616`, SHA-256
`98755ef102f0f4af2a011ab5c9ccd568a1c319743f60c258e630ff6d4059e821`):

![PR #3408 제목 열이 포함된 문서 대장](../../report/archive_register_demo/register-table.png)

두 이미지는 보정 뒤 실제 TSV의 열 계약과 일치한다. 코드·renderer·fixture 변경이 아니므로 별도 visual
sweep은 수행하지 않았다.

## 검증과 CI

- source head `db6ef6b0`의 GitHub Actions는 docs-only fast-pass로 CI preflight와 `Build & Test`가
  통과했다. heavy worker skipped는 정상이다.
- TSV 구조 검증: 271 data rows, 모든 행 8 fields, 집계값 `5706 pages / 188717KB / 255·15·1`,
  제목 검증 오류 0.
- README의 명령, TSV 링크, 이미지가 가리키는 실제 산출물을 대조했고 `git diff --check`가 통과했다.
- 원 PR만 보면 mydocs-only이므로 Cargo는 생략 대상이다. 다만 코드 PR과 함께 만든 통합 후보의 공통 전체
  게이트는 release build, release lib `2943/0/7 ignored`, release-test 전체와 IR field sweep 2/2,
  Native Skia 공식 3종 `57/0`, `2/0`, `4/0`, fmt, diff check, clippy, doc test `4/0/2 ignored`,
  전용 경로 wasm-pack web build까지 모두 통과했다.
- 보정 commit이 포함된 통합 PR 최신 head의 GitHub Actions와 mergeable 상태는 merge 전에 다시 확인한다.

## Risk와 최종 권고

`batch info` 자체에는 아직 title이 없으므로 1-pass 기능은 구현된 것이 아니다. 이는 #3407의 후속 기능 범위며,
이번 문서가 #3407을 close하지 않는다. TSV와 재현 설명의 불일치는 maintainer 보정으로 해소됐으므로
**보정 후 기술적 수용 가능**하다.

owner의 [#3445 범위 지시](https://github.com/edwardkim/rhwp/issues/3445#issuecomment-5083833363)는 당시
열린 PR을 **v0.8.2 핫픽스 기준선**에서 제외한 것이었다. 이후
[v0.8.2 릴리즈가 완료](../../report/task_m100_3445_report.md)됐으므로 현재 `devel` merge 보류로
확장하지 않는다. **최신 통합 head CI와 mergeable 상태가 성공하면 merge 권고**한다.
