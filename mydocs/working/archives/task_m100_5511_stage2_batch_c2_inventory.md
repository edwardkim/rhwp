# #5511 Stage 2 기능군 배치 C2 inventory — object·shape·media command

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 조사 HEAD: `e06299cdeaadfda193069333c92e58c4bcc18501`
- 통합 기준선: `upstream/devel` `e555f759a00fc12df234e2b4c4ed9dfa9d40400d`
- 작성일: 2026-08-20
- 상태: C2 진입 승인 — 최신 chart 계약·이동 범위·소유권 고정

## 1. 최신 기준선 통합

C2 시작 직전 `upstream/devel`과 `origin/devel`이 `cfe2c351e`에서 `e555f759a`로 2개 커밋
전진했다. #5732는 검토 문서이고 #5647은 chart 구조 편집 판정 bundle과
`tests/issue_4100_chart_data_edit.rs`를 확장했다. 후자는 C2의 chart 계약과 직접 겹치므로 옛
기준에서 inventory를 고정하지 않았다. merge-tree 무충돌을 확인한 뒤 `e06299cde`에서 최신
`devel`을 정상 merge했고, 아래 기준선은 결합 HEAD에서 다시 계측·실행했다.

## 2. 범위 판정

마스터 계획은 C2를 root의 두 구현 블록 2,065줄·15함수로 계측했다. 첫 블록 1,087줄·10함수는
chart·shape·form과 같은 object edit 수명주기를 소유하고, 그 안의 `insert-number`와
`set-page-border-fill`도 승인된 C2 책임 행에 포함된다. 두 번째 블록 978줄·5함수는
image·picture와 `ungroup-shape`다.

현재 root에는 이 두 블록과 떨어진 insert-image 전용 상수 2개와 helper 2개가 65줄 더 있다.
`insert_image_dimensions`는 insert-image와 insert-picture가 함께 쓰는 magic-byte·natural-size
규약이고 `insert_image_page_anchor`는 page를 본문 anchor로 바꾸는 insert-image 전용 규약이다.
C2 뒤에 이들만 root에 남기면 binary 자산과 anchor 정책의 정본 소유자가 사라진다. 따라서 C2
실범위를 약 2,130줄·17함수(명령 13, 전용 helper 4)로 보정한다.

| 책임 | 명령·helper | 현재 규모 |
|---|---|---:|
| document object data | set-chart-data, insert-number, form value 2종, page-border-fill | 약 640줄·5함수 |
| shape target·lifecycle | insert/delete/group/ungroup shape, target parser 2종 | 약 550줄·6함수 |
| binary media | insert-image, insert/delete/set-picture, size·page-anchor helper | 약 940줄·6함수 |

core의 object mutation, parser·serializer·renderer와 image decoding 알고리즘은 C2 범위가 아니다.
Stage 2에서는 기존 CLI adapter와 전용 helper만 물리적으로 이동한다.

## 3. 보호 계약 기준선

최신 결합 HEAD에서 다음 직접 계약 146/146을 통과했다.

| 계약 축 | 모듈 | 건수 |
|---|---|---:|
| chart edit·B2 판정 | `set_chart_data_contract`, `issue_4100_chart_data_edit` | 42 |
| number·shape lifecycle | `insert_number_contract`, `insert_shape_contract`, `delete_shape_contract`, `group_shapes_contract`, `ungroup_shape_contract` | 20 |
| form·page border | `set_form_value_contract`, `set_form_value_in_cell_contract`, `set_page_border_fill_contract` | 12 |
| image·picture | `insert_image_contract`, `insert_picture_contract`, `delete_picture_contract`, `set_picture_contract` | 31 |
| JSON·provenance 봉투 | `cli_json_contract`, `provenance_contract` | 41 |

`issue_4100_chart_data_edit`의 파일 생성용 ignored generator 2개는 모집단에서 제외하고 상시 38개를
실행했다. 계약들은 chart의 두 representation 동시성·불일치 거부·무변경 byte identity·B2 구조
수술, shape target과 group lifecycle, dry-run 무쓰기, form·page 속성 적용, binary magic-byte와
natural size, page anchor, HWP/HWPX 형식 보존, verify, MCP 등록, JSON·provenance를 보호한다.

명령별 happy path, dry-run, 잘못된 option과 MCP 경로가 이미 있고 insert-image와 chart에는
자산·anchor·저장 세부 계약이 충분하므로 move-only C2를 위한 신규 characterization은 추가하지
않는다. 관찰 가능한 공백이나 계약 차이가 발견되면 다음 이동을 멈춘다.

## 4. 복잡도 중단 조건과 처리

이동 전 cognitive-complexity 계측에서 C2 함수 중 `edit_insert_image`만 상한 25를 넘었다.

| 함수 | CC | 분해 기준 |
|---|---:|---|
| `edit_insert_image` | 27 | option·길이 parsing과 binary load·anchor·mutation·write 실행 분리 |

이 함수를 그대로 새 파일에 숨기지 않는다. 기존 진단 순서와 exit code를 보존하는 private argument
구조를 만들고 실행 함수는 magic-byte 판정, natural aspect ratio, page anchor와 저장 수명주기만
소유한다. C2 새 경로의 CC 25 초과 경고가 0건인지 확인한다.

## 5. 목표 소유권

```text
src/cli/commands/edit/
├── document_objects.rs # chart·number·form·page border command
├── shapes.rs           # shape lifecycle와 target parser
└── media.rs            # image·picture command와 binary size·page anchor
```

세 command module은 C0 `edit::runtime::finish_edit_write`와 기존 root load seam을 소비한다.
`shapes.rs`의 target parser와 `media.rs`의 binary helper는 모듈 내부 전용으로 유지한다. command
handler만 edit parent에 `pub(super)`로 노출하고 root wrapper나 helper 복제를 두지 않는다.
세 파일은 모두 1,200줄 이하를 유지한다.

`DocumentService`, typed error와 전역 인증 제거는 Stage 3 범위다. C2에서 광범위한
`EditContext`를 만들거나 C3 table 좌표, C4 document structure, C5 formatting 책임을 끌어오지
않는다.

## 6. 구현·커밋 순서

1. 이 inventory와 최신 `devel` 결합 기준을 독립 커밋으로 고정한다.
2. chart·number·form·page-border command를 `document_objects.rs`로 이동한다.
3. shape lifecycle과 target parser를 `shapes.rs`로 이동한다.
4. image·picture command와 전용 helper를 `media.rs`로 이동하고 insert-image parsing·실행을 나눈다.
5. 146개 직접 계약과 전체·정적·정책 관문을 실행하고 완료 보고서를 커밋한다.

각 절편은 stdout/stderr, JSON, exit code, target 주소, output 이름·형식, 원본 보호와 파일 부작용을
보존한다. 구현 커밋마다 해당 focused 계약과 format·diff 검사를 실행한다.

## 7. 원격 위험과 중단 기준

조사 시점 `origin/devel`과 `upstream/devel`은 모두 `e555f759a`이고 현재 HEAD의 조상이다. 열린
devel 대상 PR #5689, #5691, #5695, #5707, #5709, #5710, #5718, #5719의 최신 변경 경로에는
root, edit command, C2 직접 계약과 #5511 C2 문서가 없다. #5647의 chart 계약 중첩은 이미 최신
`devel` 병합으로 흡수했다.

다음 경우에는 같은 승인 배치 안에서도 이동을 멈추고 메인테이너에게 보고한다.

- 146개 기준 계약의 stdout/stderr·exit·target·binary·anchor·저장 방어가 달라지는 경우
- 새 파일 1,200줄 또는 CC 25 상한을 지킬 수 없는 경우
- root와 새 모듈의 양방향 참조나 object·shape·media 사이 helper 복제가 생기는 경우
- 최신 devel·열린 PR이 같은 함수, 테스트, 모듈 경계를 변경한 경우
- move-only 범위를 넘어 core mutation, parser·serializer·renderer나 image 알고리즘 변경이 필요한 경우
