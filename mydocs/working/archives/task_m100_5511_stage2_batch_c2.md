# #5511 Stage 2 기능군 배치 C2 — object·shape·media command

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 최종 통합 기준: `upstream/devel` `e555f759a00fc12df234e2b4c4ed9dfa9d40400d`
- 최종 코드 HEAD: `f5c58979baafd670030d01ddf18c633fafe7ee22`
- 수행일: 2026-08-20
- 상태: 완료 — C2 종료, C3 진입 승인 대기

## 1. 결과

chart·number·form·page border, shape lifecycle, image·picture command와 전용 helper를 책임별
소유 모듈로 분리했다.

```text
src/cli/commands/edit/
├── document_objects.rs # chart·number·form·page border
├── shapes.rs           # shape lifecycle와 target parser
└── media.rs            # image·picture와 binary size·page anchor
```

| 책임 | 최종 파일 | 줄 수 |
|---|---|---:|
| document object data | `src/cli/commands/edit/document_objects.rs` | 650 |
| shape target·lifecycle | `src/cli/commands/edit/shapes.rs` | 549 |
| binary media | `src/cli/commands/edit/media.rs` | 1,010 |

세 파일은 모두 1,200줄 상한 이하다. `src/main.rs`는 C2 시작의 13,736줄에서 11,601줄로
2,135줄 줄었다. 마스터 계획의 두 구현 행은 2,065줄·15함수였지만 root에 떨어져 있던
insert-image 전용 상수·size/page-anchor helper 65줄·2함수를 포함해야 binary·anchor 정본이
새 모듈을 따라간다. 이를 약 2,130줄·17함수로 보정해 수행했다. 구현 중 `insert-image`의
argument parser 1개를 새 private 함수로 분리했으므로 최종 세 모듈에는 함수 18개가 있다.
root wrapper, helper 복제와 core 알고리즘 변경은 없다.

## 2. 소유권과 복잡도 결정

`document_objects.rs`는 chart data, number control, form value와 page border command를 소유한다.
`shapes.rs`는 insert/delete/group/ungroup과 `para,ctrl` target parser를 함께 소유해 lifecycle과
주소 해석의 정본을 나누지 않는다. `media.rs`는 image magic-byte·natural size, page anchor와
image/picture command를 함께 소유한다. command handler만 edit parent에 `pub(super)`로 노출하고
전용 parser·helper는 private로 유지했다.

공개 schema, binary decoder, page layout, object mutation, parser·serializer·renderer는 바꾸지
않았다. 세 모듈은 C0의 edit runtime과 기존 document load seam만 소비한다. `DocumentService`,
typed error와 전역 인증 제거는 계획대로 Stage 3 입력으로 남겼다.

이동 전 C2 함수 중 `edit_insert_image`만 cognitive complexity 27로 상한 25를 넘었다. 기존
진단 순서, exit code, stdout/stderr와 파일 부작용을 유지한 채 option·길이 parsing을
`parse_insert_image_args`로 분리했다. 최종 C2 세 모듈의 CC 25 초과 경고는 0건이다.

## 3. 최신 기준선과 커밋 계보

C2 시작 직전 `devel`이 `e555f759a`까지 2개 커밋 전진했고 #5647이 C2 chart 계약을 직접
확장했다. merge-tree 무충돌을 확인한 뒤 정상 merge하고 결합 HEAD에서 inventory와 기준 계약을
다시 고정했다.

| 커밋 | 역할 |
|---|---|
| `e06299cde` | 최신 `upstream/devel` 정상 merge |
| `64ae3384c` | C2 실범위·146개 보호 계약·소유권 inventory |
| `8d69d9d5a` | document object command 5개 이동 |
| `ea497853f` | shape lifecycle 4개와 target parser 이동 |
| `f5c58979b` | media command 4개·전용 helper 이동과 insert-image parsing 분리 |

## 4. 직접 계약

이동 전과 최종 코드 HEAD에서 C2 직접 계약 146/146을 통과했다.

| 계약 축 | 모듈 | 건수 |
|---|---|---:|
| chart edit·B2 판정 | `set_chart_data_contract`, `issue_4100_chart_data_edit` | 42 |
| number·shape lifecycle | `insert_number_contract`, `insert_shape_contract`, `delete_shape_contract`, `group_shapes_contract`, `ungroup_shape_contract` | 20 |
| form·page border | `set_form_value_contract`, `set_form_value_in_cell_contract`, `set_page_border_fill_contract` | 12 |
| image·picture | `insert_image_contract`, `insert_picture_contract`, `delete_picture_contract`, `set_picture_contract` | 31 |
| JSON·provenance 봉투 | `cli_json_contract`, `provenance_contract` | 41 |

구현 절편별로 document object 58/58, shape 16/16, media 31/31도 통과했다. 최종 계약은
chart 두 representation과 B2 구조 편집, shape group lifecycle, form·page 속성, dry-run 무쓰기,
binary magic-byte·natural aspect ratio, page anchor, HWP/HWPX 형식 보존, 저장 후 verify,
MCP 등록과 JSON·provenance를 보호한다. 파일 생성용 ignored chart generator 2개는 상시 모집단에서
제외했다.

## 5. 최종 검증

| 검증 | 결과 |
|---|---|
| C2 직접 focused 계약 | 이동 전·최종 146/146 통과 |
| 최종 release-test 전체 nextest | 8,008/8,008 통과, 3 slow, 39 skipped, 155.693초 |
| `cargo fmt --all -- --check`·`git diff --check` | 통과 |
| `cargo check --locked --all-targets` | 통과 |
| `cargo clippy --locked --all-targets -- -D warnings` | 통과 |
| C2 모듈 CC 25 상한 | 초과 경고 0건 |
| `cargo doc --locked --no-deps` | 성공, 기존 rustdoc 경고만 존재 |
| `cargo test --locked --doc` | 8/8 통과, 3 ignored |
| integration manifest 정책·현재 상태 | 18/18, 803 sources / 3,960 attrs / 41/48 targets 통과 |
| unit-tier 정책·현재 상태 | 12/12, 4,225 tests / 299 modules 통과 |
| CI impact Node·Python workflow 계약 | 62/62, 163/163 통과 |

전체 회귀는 매뉴얼의 고정 target 명령인 `--cargo-profile release-test --target-dir
target/pr-review --tests --test-threads 8 --no-fail-fast`로 실행했다. 로컬 nextest 0.9.137이 저장소
권고 0.9.140보다 낮다는 경고가 있었지만 8,008개 모집단은 정상 실행되어 전건 통과했다.

`rust-test-suite-manifest --prepare`의 파생 harness와 Cargo target은 추적 변경에 포함하지 않았고
최종 worktree에 남지 않았다. C2는 CLI adapter 이동과 내부 argument parser 분리이므로
renderer·layout·WASM·native-skia·시각 검증 발생 조건에 해당하지 않는다. binary와 page-anchor
동작은 직접 image 계약으로 검증했다.

## 6. 최신 devel과 열린 PR

최종 검증 뒤 `upstream/devel`을 다시 fetch했으며 `e555f759a00fc12df234e2b4c4ed9dfa9d40400d`로
C2 통합 이후 전진하지 않았다. 최종 코드 HEAD는 최신 `upstream/devel`을 조상으로 포함하며
원격보다 46개 커밋 앞서고 뒤처진 커밋은 없다.

열린 devel 대상 PR은 #5689, #5691, #5695, #5707, #5709, #5710, #5718, #5719, #5733이다.
각 최신 head의 변경 경로를 다시 조회했으며 C2의 root, edit command 세 모듈, 직접 계약과
#5511 C2 문서 경로에 겹침이 없다. 새 #5733은 renderer 통합 검토 경로이고 C2 경계를 건드리지
않는다.

이 판정은 시점 증거다. 향후 push 직전에 exact base SHA, 열린 PR head와 merge 가능성을 다시
확인한다. 이 보고서 작성 시점에는 remote push를 수행하지 않았다.

## 7. 다음 승인 단위

C2 완료로 object data, shape lifecycle과 binary media command의 물리 경계를 확정했다. 다음
기능군은 C3 cell·row·column·table·equation이며 table 좌표, nested cell path와
`finish_edit_write` 수명주기를 먼저 inventory한다. C3는 메인테이너의 C2 완료 승인과 별도 진입
승인 전 시작하지 않는다.
