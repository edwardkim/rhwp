---
kind: review-implementation
status: completed
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4445 humdrum00001010 누적 통합 구현·검토 기록

기준은 `upstream/devel`의 `e48fe86947fbf9a44b1b98c7037150751af541ab`이다. 원 PR
20건은 모두 이 commit에서 직접 분기했고 merge commit이 없다. #4315는 의도적인 red 테스트
Draft이므로 이 후보에서 제외한다. 누적 브랜치는 검증 전용이며 원 PR branch나 devel에 push하지 않는다.

## 적용 그룹과 순서

1. renderer/document_core: #4313, #4316, #4360, #4363, #4365, #4374, #4380, #4382, #4394, #4420
2. HWPX: #4409, #4411, #4415, #4417, #4421, #4425
3. clipboard: #4416, #4426
4. 독립 serializer: #4419, #4434

그룹 안에서는 오래된 PR 번호 순서로 기능·문서 commit을 원 순서대로 체리픽한다. 각 PR 적용 뒤
`git diff --check`와 상태를 확인하고 원 head와 누적 commit 대응표를 이 문서에 추가한다.

## 검증 게이트

1. PR별 focused test를 변경 축별로 순차 실행한다.
2. renderer/layout 후보는 WASM build와 대표 fixture 시각 검증을 수행한다.
3. 최종 누적 후보에서 고정 `target/pr-review`를 사용해 전체 release-test, Native Skia 3종,
   fmt, diff check, clippy를 순차 실행한다.
4. 원 PR별 최신 head, mergeable, required checks를 다시 확인하고 작업지시자에게 merge 승인을 요청한다.

## 중단과 rollback

체리픽 충돌은 임의로 해소하지 않는다. 해당 PR 직전 checkpoint에서 중단해 충돌 파일과 양쪽 의도를
보고한다. 원 PR head·기존 local ref·사용자 작업은 재작성하거나 삭제하지 않는다.

## 적용 결과

20개 원 PR의 commit을 아래 순서로 누적 적용했다. 모든 local SHA는
`review/humdrum00001010-20260810`에서 생성됐으며 원 author를 보존한다.

| PR | 누적 local SHA |
| --- | --- |
| #4313 | `2eb9770dc`, `68dfdb354` |
| #4316 | `c36e9bf64`, `cd00b5a3d`, `c0d7889e0` |
| #4360 | `b85bbeb79`, `a1fa9fc79`, `7fe879fd7` |
| #4363 | `3ed12d7b1`, `d202b282f` |
| #4365 | `17d2b2309`, `5c8606436`, `1bc4694b1` |
| #4374 | `3aab4748d`, `15384c6cd` |
| #4380 | `ef9601553`, `a86b33e95`, `60ae5fea4`, `7e2c3d535` |
| #4382 | `b3b0044e3`, `d39c7a6c9` |
| #4394 | `f0bed9704`, `edf012976` |
| #4420 | `46453689a`, `dc89e6561` |
| #4409 | `29a468ebd` |
| #4411 | `2d32438ba`, `75897ea92`, `a78aa77b2` |
| #4415 | `7553410cd`, `86771c681` |
| #4417 | `caa3ac50c`, `6c6fdc97e` |
| #4421 | `1d9383e83`, `232d4d866`, `ec9fe2f4a` |
| #4425 | `f6dbf96b4`, `f1e7c5706`, `bb973bd88` |
| #4416 | `4237bc8a1`, `afcf6f608` |
| #4426 | `97822085d`, `6f33ccb73`, `69e5edc90`, `022171390` |
| #4419 | `6e63c823d`, `76188de80` |
| #4434 | `1e7554227`, `98252f1e9` |

## 기능 충돌 해소

작업지시자의 건별 승인을 받아 다음 네 곳을 의미 기준으로 통합했다.

1. #4316이 제거한 `Scope::requires_include_fields()`를 #4365의 FieldMemo 소유자
   순회가 다시 요구했다. 중복 술어가 아니라 갱신된 단일 메서드로 유지했다.
2. #4313의 `first_text_line()`과 #4382의 `caption_height_px()`가 같은 삽입 위치를
   사용했다. 서로 다른 책임이므로 두 함수를 모두 유지했다.
3. #4316의 `render_tree_from_layer_tree()` 명명 정리와 #4394의 `RenderProfile`
   전달을 결합했다. 옛 bool과 옛 함수 호출은 남기지 않았다.
4. #4416의 셀 BorderFill 보정과 #4426의 중첩 셀 컨트롤 내보내기가
   `table_to_html` 계약에서 충돌했다. 테스트 호출부를 바꾸지 않고 최상위
   `table_to_html(table)` 계약과 깊이 인자를 받는 내부
   `table_to_html_at_depth(table, depth)`를 분리했다. 메인테이너 보정 commit은
   `1f5c4f3e4`다.

## 로컬 검증 결과

- 충돌 지점 focused: FieldMemo 1/1, rowbreak chart overlap 20/20,
  RenderProfile 경로 동등성 1/1, #4384 4/4.
- HWPX 묶음 focused: 30/30.
- clipboard 묶음 focused: 기존 BorderFill 1/1, 셀 컨트롤 7/7,
  중첩 표 import 왕복 1/1. 테스트 코드는 변경하지 않았다.
- HML/HWP5 serializer focused: #4386 2/2, #4402 7/7.
- 누적 전체: `cargo nextest run --cargo-profile release-test --target-dir
  target/pr-review --tests --test-threads 12 --no-fail-fast` — **5,567/5,567
  통과, 정책 skip 35**. 실행 nextest는 0.9.137이며 저장소 권고 0.9.140 경고가 있었다.
- Native Skia 공식 3종: **58/58 + 2/2 + 4/4**.
- `cargo fmt --check`, `git diff --check`,
  `cargo clippy --all-targets -- -D warnings` 통과.
- 표준 Docker WASM: `docker compose --env-file .env.docker run --rm wasm`
  통과(5분 01초). `pkg/rhwp_bg.wasm` 7,810,792 bytes, SHA-256
  `197c9ed11dca577a55b3e3463bbc43539999e33b6930ccd77ec4790373a34f32`,
  소유권 `edward:edward`(1002:1002).

## 시각 검증 자료

모든 임시 산출물은 ignore 대상인 `output/4313/batch-cumulative/`에 남겼다.

- 2024-09 문서 17·22쪽: 한컴 PDF와의 기존 fidelity 후보가 남지만, 누적 후보 SVG는
  기존 #4313 검토본과 두 페이지 모두 SHA-256·byte가 동일하다. 17쪽 하단 overflow는
  이번 PR 범위 밖으로 이미 분리한 기존 결함이다.
- `76076_regulatory_analysis.hwp` 5·6쪽: visual sweep 구조 후보 0건.
- `복학원서.hwp` 1쪽: visual sweep 구조 후보 0건.
- #4420 발동 조건: 스크래치 HWP5에서 제목을 `접 수 표` / `Filing Notice`로
  바꾸고 재저장·재로드해도 PUA 날인선과 `(인)` 표시가 유지되고
  `overflowCellLines=0`이다.

## 최종 재확인과 권고

- 작업지시자가 2026-08-10 누적 후보의 시각 판정을 통과시켰다.
- 같은 날 원 PR 20건을 다시 조회했다. 검토 시작 뒤 contributor의 추가 push가 없었고,
  모든 원 head가 기록값과 일치하며 `OPEN` / `CLEAN` / `MERGEABLE` /
  required checks `SUCCESS`다.
- `upstream/devel`도 누적 검증 기준
  `e48fe86947fbf9a44b1b98c7037150751af541ab`에서 변하지 않았다.

**통합 PR 수용 권고.** 승인된 네 건의 의미 충돌 해소와 메인테이너 보정
`1f5c4f3e4`는 원 PR head에 없으므로 원 PR들을 개별 merge하면 검증한 누적 트리를
재현하지 못한다. 별도 승인을 받아 현재 누적 branch를 integration PR로 게시하고 그
최신 head의 required checks를 통과시킨 뒤 merge한다. 통합 뒤에는 원 PR별 기여 안내와
close, 관련 issue 상태 확인을 후속 처리한다. 의도적인 red-test Draft #4315는 이
통합과 후속 close 대상에서 제외한다.

## PR 생성 뒤 상태

- 작업지시자 승인으로 [통합 PR #4445](https://github.com/edwardkim/rhwp/pull/4445)를
  `devel` 대상 Open PR로 생성했다.
- code·검증 기록 head `caceebd36cc72f4cb3771fdb249d786088658ac7`의 최초
  GitHub CI, CodeQL, Canvas visual diff, Native Skia와 Build & Test가 모두 성공했다.
- 최초 reviewer `jangster77` 지정 뒤 작업지시자가 maintainer self-review로 전환했다.
  작성자 본인에게 reviewer request나 `APPROVE`를 만들지 않고 최신 head에
  `COMMENTED` self-review를 남긴다. 1차 트리야지는 assignee `edwardkim`, milestone
  `v1.0.0`, 변경 범위 기반 기존 label로 유지한다.
- 이 번호 기반 review·오늘할일 trailing commit이 새 head를 만들므로, merge 조건은
  그 최신 head의 required checks 성공·mergeability 재확인·`COMMENTED` self-review·
  작업지시자 merge 승인이다.

## 통합 완료

- 최종 head `02caa3e03d7edd2716a598b54cbabfab0b833175`의 review-only
  fast-pass와 `COMMENTED` self-review를 확인했다.
- 작업지시자 승인 뒤 [PR #4445](https://github.com/edwardkim/rhwp/pull/4445)를
  `baf6ef7ff13ebbb782277b8391b81b81f1250a5f`로 `devel`에 merge했다.
- 원 PR·issue 후속 처리와 정확한 review branch 정리는 merge 뒤 절차로 분리한다.
