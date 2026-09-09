# PR #6759 등 CI 그린 5건 통합 검토·검증 기록

## 판정: 로컬 검토 완료, 통합 PR 생성 승인됨

2026-09-05 갱신. #6759·#6763은 승인, #6760·#6762·#6768은 메인터너 보정 됨, 수용 가능이다. #6763의 실제 치수 변경·재확인 CDP 검증이 완료되어 로컬 보류 사유를 해소했다. 개별 수용 판정은 GitHub approve나 merge 실행을 뜻하지 않으며 작업지시자 시각 확인과 최종 head CI는 별도다.

## 기준과 검증 대상

- 작업공간: `/Users/tsjang/rhwp`.
- 브랜치: `review/ci-green-6759-6768-20260905`.
- 기준 devel: `2c144b180dd776aa450c499778510199ae6cdf89`.
- 메인터너 보정 전 체리픽 HEAD: `d87b3037e5aeb6b662904b0182c361d5a2929108`.
- 검증 대상은 위 HEAD에 당시 미커밋 code·test·sample·baseline 보정을 더한 작업 트리였다. 해당 보정은 이후 `902a208b515e83024502f004a2adaf84c33f18de`로 커밋했다. 최종 통합 PR head 또는 원 PR head 자체가 검증됐다고 바꾸어 적지 않는다.
- 선정 당시 open·non-draft이며 CI가 그린인 #6759·#6760·#6762·#6763·#6768에 리뷰어 `jangster77`를 먼저 할당했다.
- 선정 당시 #6765는 CI 실패, #6767은 이미 병합 완료였다. #6761·#6764·#6766은 PR이 아니라 issue다.
- 원 head CI의 성공·중립·예상 skip 원값은 개별 리뷰에 보존했다. 현재 GitHub 상태를 재조회한 결과가 아니며 통합 PR CI는 아직 없다.

## 체리픽 출처

| 순서 | 원 PR / 원 SHA | 로컬 SHA | 제목 요약 |
| --- | --- | --- | --- |
| 1 | #6759 / `4071ae534bd380a18c41125284f96c1b5a6a7e15` | `86f79ec31` | 선택 복원 테스트의 주석 오탐 방지 |
| 2 | #6760 / `ac6c297c5aacf47e3358f401a5123f02ea8e7157` | `b07744075` | 컷 벡터 범위 초과 시 행 단위 인덱스 복원 |
| 3 | #6762 / `df03b740066dc96aa4ecb26665561d2d2f63d5a9` | `09862f8f1` | lazy 기준 역산의 트림 복원 |
| 4 | #6763 / `3fd1bb410c80b0358564c2f466fbeafae0af8fe0` | `9d161e12d` | 변경하지 않은 개체 치수 미전송 |
| 5 | #6763 / `00422f4683d47cda7a46f5e2a8fb47bf7bc51fdb` | `ba245487d` | 크기 표시 서식 공용화 |
| 6 | #6768 / `31fdc395021e7974dca74feaa6f9ed3b3a395bf8` | `d87b3037e` | 후속 블록 표의 float 배제 밴드 반영 |

여섯 커밋을 `git cherry-pick -x`로 누적 적용했고 충돌은 없었다. 아래 결과는 이 누적 적용 상태에 메인터너 보정을 더해 실행한 검증이다.


## PR별 판정

| 원 PR | 판정 | 근거와 남은 범위 |
| --- | --- | --- |
| [#6759](pr_6759_review.md) | 승인 | 소스 계약의 주석 오탐 방지, 제품 동작 변경 없음, Studio 테스트 통과 |
| [#6760](pr_6760_review.md) | 메인터너 보정 됨, 수용 가능 | 정식 sample·필수 읽기, 중복·용지 초과 회귀와 2·3쪽 대조 완료, 한컴과 쪽별 행 배분 차이는 남음 |
| [#6762](pr_6762_review.md) | 메인터너 보정 됨, 수용 가능 | lazy 기준 정규화 순서 수정, 11개 경계 입력과 실물 회귀 통과, 5·6쪽 직접 대조 |
| [#6763](pr_6763_review.md) | 승인 | CDP로 무변경 1→1, 명시적 변경 1→283, 재확인 283→283과 너비 17716 보존 확인 |
| [#6768](pr_6768_review.md) | 메인터너 보정 됨, 수용 가능 | 필수 sample·표 존재/행 보존 검증, 기준 devel A/B와 대조 완료, #6764 전체 종료는 불가 |

## 메인터너 보정 내용

- #6760·#6768: 비공개 경로를 찾지 못하면 검사 없이 성공하던 테스트를 정식 sample의 필수 읽기로 바꿨다. sample README·MANIFEST와 HWP의 binary 속성을 함께 등록했다.
- #6762: `src/renderer/height_cursor_lazy_base.rs`의 공용 helper에서 작은 음수를 먼저 정규화한 뒤 fallback을 선택한다. 진단용 `lazy_base_corrected` 값도 유지했다.
- #6762: 제품 helper와 integration test가 같은 함수를 사용하도록 한정된 module 정책을 추가했다. 새 source-side `cfg(test)` module은 만들지 않았고 integration target 예산 48은 올리지 않았다.
- #6768: `tests/cases/issue_6764_public_table_fragment_presence.rs`에서 실제 CLI render tree의 특정 표가 하나인지, source row 0~22와 본문 anchor가 있는지, 그 표가 용지 안인지 확인한다. 표 소실을 정상으로 오인하지 않는다.
- 새 #6764 sample에만 text-overlap 23, off-canvas 5를 등록했다. 기존 fixture 기준선을 올리거나 검사 전체를 skip하지 않았다.

## 최초 실패와 기준선 보정 근거

첫 전체 실행은 9,048건 중 9,046 통과, 2 실패, 46 skip, exit 100이었다. 실패는 새 #6764 sample의
`text_overlaps_do_not_grow_partition_3`와 `off_canvas_does_not_grow_partition_3`였다. 실패를 삭제하거나 최초부터 통과한 것으로 기록하지 않는다.

같은 입력 SHA-256 `8ef9de3f35690bf9d7994527f77cb02d4a4fcff447c219a78fbc2855d64be6e7`로
실제 기준 devel 바이너리와 보정 포함 후보를 각각 실행했다.

| 필드 | 기준 devel | 후보 | 판정 |
| --- | ---: | ---: | --- |
| 전체 rhwp 페이지 | 200 | 201 | 한컴 204쪽과 같다고 주장하지 않음 |
| text-overlap 탐지 건수 | 23 | 23 | 신규 경로에 기존 수준 23 등록 |
| off-canvas 탐지 건수 | 6 | 5 | 신규 경로에 개선된 5 등록 |
| 표 하단 최대 초과 | 885.5567 px | 13.3567 px | 큰 표 조각 초과 해소, 다른 작은 초과는 잔여 |

- 대상 경로: `issue6764/1613000-202200037-air-traffic-controller-cbta.hwp`.
- `tests/fixtures/text_overlap_baseline.tsv`에는 이 경로의 건수 23 한 행만 추가했다.
- `tests/fixtures/off_canvas_baseline.tsv`에는 이 경로의 건수 5 한 행만 추가했다.
- 기준 바이너리 SHA-256: `d34749fbff8d855ef6f019a6ea9b59b272e287399ea72c2adfa5f229c57ed5fc`.
- 후보 바이너리 SHA-256: `2edf5134fb14f1a3fafa1a28a1b0673d0206ca55642abaf0ee51b8a35919c58f`.
- 후보에서도 남는 제목·본문 초과와 페이지 매핑은 [시각 대조 기록](pr_6759_6768_visual_sweep.md)에 적었다. 모든 탐지를 오탐이라고 분류하지 않는다.

## 보정 후 실제 검증 결과

| 검증 | 실제 결과 |
| --- | --- |
| 전체 Rust integration·실물 회귀 | **9,049 통과, 0 실패, 46 skip**, 294.741초, exit 0 |
| #6753 focused | 2 통과, 경계 입력 11개와 실물 문서 포함 |
| #6756 실물 회귀 | 2 통과, 필수 sample 경로 사용 |
| #6764 실물·표 보존 회귀 | 기존 경계 검사와 신규 표 존재·행 보존 검사 모두 통과 |
| IR sweep·overflow-cell·security corpus | 최종 전체 회귀에 포함되어 통과 |
| Native Skia library | workspace 합계 4,112 통과, 13 ignored, 실패 0, exit 0 |
| Native Skia placeholder focused | 2 통과, exit 0 |
| Native Skia direct PDF focused | 4 통과, exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| native·WASM·workspace Clippy | 각 exit 0 |
| workspace build | exit 0 |
| suite manifest 정책 테스트 | 21 통과 |
| suite manifest 검사 | 1,168 source, 4,918 static test attr, 28 suite + 20 exception = 48/48 target |
| Studio 전체 테스트 | 총 1,404건 중 1,403 통과, 1 skip, 실패 0 |
| TypeScript `npx tsc --noEmit` | 새 WASM 생성 후에도 exit 0 |
| 로컬 WASM build | locked wrapper·wasm-opt 포함 성공, exit 0, Docker 사용 안 함 |

최종 전체 회귀 명령:

~~~bash
RHWP_IR_SWEEP_DUMP=/tmp/rhwp-pr6759-ir-final.tsv \
RHWP_OVERFLOW_CELL_DUMP=/tmp/rhwp-pr6759-overflow-final.tsv \
cargo nextest run --locked --cargo-profile release-test \
  --target-dir target/pr-review --tests --test-threads 6 --no-fail-fast
~~~

Native Skia library 및 WASM 빌드 명령:

~~~bash
cargo test --locked --profile release-test --target-dir target/pr-review \
  --features native-skia --lib -- --test-threads 6
CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg
~~~

Cargo 검증은 같은 checkout·target에서 동시에 실행하지 않았다. 생성된 `tests/generated/`와
`tests/suites/manifest.json`은 커밋 대상이 아니다. WASM SHA-256은
`f531f4d540839b4d2630f3ffb120d5704ef31659e17acd28236f3d7328743321`이다.

## 추가 Studio 직접 검증: #6763 완료, renderer 추가 캡처 미완료

#6763의 최초 스크립트는 두 번째 속성 창 열기에서 30초 timeout, exit 1이었다.
원인은 포커스를 받지 않는 canvas에 `focus()`를 호출해 `P` 키가 `BODY`로 전달된 검증 준비 문제였다.
숫자 부분 선택으로 `.00`이 남아 100.00 mm가 입력되는 추가 준비 오류도 구분해 기록했다.

편집 textarea와 초기 자동 포커스를 기다리고, CDP로 정확한 폼 입력값을 채운 뒤 실제 설정 버튼을 거치는
최종 검증은 **exit 0**이었다. 높이 `1→1→283→283`, 너비 `17716` 보존과 재열기 `62.50 / 1.00` mm를 확인했다.
모델 setter를 직접 호출하지 않았다. 제품 코드는 추가 보정하지 않았으며
기존 전체 회귀 9,049건은 이번 문서·임시 검증 스크립트 보정 때문에 재실행하지 않았다.

코멘트용으로 변경 후 재열기 확인 PNG 한 개만 추가했다. 최종 JSON·로그와 다른 임시 PNG는
`/tmp/rhwp-6763-size-validation/` 등 저장소 밖에 두고 커밋하지 않는다.

별도의 renderer Studio canvas 캡처는 첫 문서 페이지 준비 대기에서 30초 timeout, exit 1이었던 상태 그대로다.
이번 #6763 CDP 성공을 renderer 세 문서의 새 Studio canvas 검증 성공으로 바꾸어 기록하지 않는다.
renderer의 수용 근거는 앞서 완료된 native SVG/Chrome raster와 한컴 PDF 직접 대조다.

## 증적 보존과 댓글 준비

- [시각 대조 정본](pr_6759_6768_visual_sweep.md)에 입력·PDF·바이너리 출처, 실제 대조 페이지·지표·한계와 대표 PNG를 기록했다.
- 이번 검토의 `mydocs/pr/assets/pr_6759_6768_20260905/`에는 코멘트에 사용할 PNG **9개만** 남겼다.
- 중간 JSON 275개, SVG 218개, PNG 83개, 로그 6개, HTML 3개, 총 585개는 저장소 밖 임시 경로로 옮겨 커밋 대상에서 제외했다.
- 정식 sample의 MANIFEST와 두 최종 기준 PDF는 보존한다. 기존 #6753 기준 PDF도 재사용한다.
- 생성 로그·JSON을 첨부하지 않고 실제 실행 결과와 해시·수치를 Markdown에 기록한다.
- 개별 리뷰의 댓글 계획은 commit SHA 고정 raw PNG URL, 직접 이미지 표시, 실제 CI, UTF-8 body-file 및 API 재조회 조건을 포함한다.
- #6764는 제목·쪽 배분 등 잔여 문제가 있으므로 issue close 대상이 아니다. #3416도 기존 CLOSED 상태를 새 해결로 표현하지 않는다.

## 남은 단계와 승인 경계

1. 완료: #6763은 임시 검증 스크립트의 편집 입력 포커스·숫자 폼 값 설정을 보정하여 CDP 검증을 마쳤다. 제품 소스는 추가 변경하지 않았다.
2. 완료: 개별 리뷰·최종 시각 증적·2026-09-05 오늘할일과 통합 PR 본문 초안을 준비했다. 기존 오늘할일은 보존하고 이번 5건의 결과만 별도 항목으로 추가했다. commit·push·PR 생성은 작업지시자 승인을 받았으며 시각 확인과 merge 승인은 별도다.
3. 작업지시자의 PR 생성 승인에 따라 메인터너 보정·정식 sample을 `902a208b515e83024502f004a2adaf84c33f18de`로 커밋했다. review·오늘할일·최종 증적은 같은 branch의 후속 문서 커밋으로 묶어 upstream temporary head에 push한다.
4. 최종 head CI·mergeable·mergeStateStatus와 merge 승인을 확인한 뒤 일반 merge commit을 사용한다.
5. merge SHA의 devel CI 성공 뒤에만 `post_merge.md`에 따라 원 PR·관련 issue의 기존 댓글을 확인해 수정 또는 게시하고 필요한 close를 처리한다.
6. 종료된 이번 작업에 한해 실행 중 작업 없음·clean·merge SHA의 devel 포함을 확인한 뒤 승인된 branch·worktree·전용 target만 정리한다.

메인터너 보정 commit은 완료했다. 이 기록은 같은 branch의 후속 문서 커밋과 승인된 push·PR 생성을 위한 상태다. GitHub approve·merge·원 PR/issue comment·close는 실행하지 않았다.

## PR 준비와 검증 결과 재사용

- 통합 PR 대상은 #6759·#6760·#6762·#6763·#6768 다섯 건, 원본 출처를 보존한 체리픽 여섯 커밋이다.
- 2026-09-05 PR 준비 단계에서는 작업지시자의 지시에 따라 이미 통과한 테스트·lint·빌드를 재실행하지 않았다. 위 실행 결과를 재사용하며, 다시 실행한 것처럼 시각·건수·exit code를 갱신하지 않는다.
- 최종 제품·회귀 보정 이후 추가 변경은 검토 문서·오늘할일·증적 정책과 CDP 검증 산출물이다. 이번 준비 작업에서는 제품·테스트 코드를 수정하지 않았다.
- 검증 재사용은 로컬 중복 실행 생략이다. 새 통합 PR의 required CI를 비활성화하거나 성공으로 간주하지 않는다.
- 본문 초안: `/tmp/rhwp-pr6759-6768-body.md`. 제목 초안: `/tmp/rhwp-pr6759-6768-title.txt`. 커밋 범위 계획: `/tmp/rhwp-pr6759-6768-commit-plan.md`. 이 세 파일은 로컬 준비물이며 커밋 증적이 아니다.
- 종료 참조 초안은 #6753·#6756·#6758이다. #6764는 잔여 제목·쪽 배분 문제로 종료하지 않으며, 이미 닫힌 #3416과 원 PR 다섯 건에 자동 종료 참조를 붙이지 않는다.
- 작업지시자 승인에 따라 code/test/sample 보정과 review/오늘할일/최종 증적을 분리된 커밋으로 같은 branch에 반영한다. 원 contributor 커밋은 rewrite하지 않는다. 추가 문서 전용 PR은 만들지 않는다.
