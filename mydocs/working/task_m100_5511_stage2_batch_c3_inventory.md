# #5511 Stage 2 기능군 배치 C3 inventory — cell·table·equation command

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 조사 HEAD: `05cae2eb75ca6a7aa284ae2260b2ad1e9d00412d`
- 통합 기준선: `upstream/devel` `73939045ef8e806519a64ed0f55a663bf08a2a45`
- 작성일: 2026-08-20
- 상태: C3 진입 승인 — 최신 기준선·이동 범위·좌표 seam 소유권 고정

## 1. 최신 기준선 통합

C3 시작 직전 `upstream/devel`이 `e555f759a`에서 `73939045e`로 1개 커밋 전진했다. 이 변경은
Studio 스킨과 관련 매뉴얼·검토 문서만 수정해 C3 source·계약과 직접 겹치지 않았다.
merge-tree 무충돌을 확인한 뒤 `05cae2eb7`에서 최신 `devel`을 정상 merge했고, 아래 기준선은
결합 HEAD에서 다시 계측·실행했다.

## 2. 범위 판정

마스터 계획은 C3를 3,639줄·33함수로 계측했다. 현재 함수 경계와 호출 계보를 다시 대조하면
실제 C3 소유권은 약 3,525줄·31함수다. 명령 handler 25개와 셀 내용·table 좌표 전용 helper
6개로 구성된다.

기존 33함수 계측에 포함됐던 `hu_to_mm`·`hu_to_mm_i`는 현재 table edit가 아니라 Q2 vector
output과 Q5 diagnostics·control dump가 공동 소비하는 query/output 단위 변환 seam이다. Q5에서
root 공유 seam으로 보존하기로 이미 판정했으므로 C3에 끌어오지 않는다. 이 두 함수를 제외하고
현재 함수 시작점에서 다음 최상위 함수 시작점까지 합산하면 약 3,525줄이며, 주석 경계에 따른
차이는 이동 후 실제 파일 줄 수로 다시 정산한다.

| 책임 | 명령·helper | 현재 규모 |
|---|---|---:|
| cell content·properties | set-cell, insert/delete-text-in-cell, set-cell-props, text/overflow helper 4종 | 약 990줄·8함수 |
| table coordinates | top-table·cell resolver | 약 100줄·2함수 |
| table structure | insert/delete table·row·column, merge/split/transpose | 약 1,520줄·12함수 |
| table layout·properties | fit/resize/move, widths, cell/table properties | 약 600줄·6함수 |
| equation lifecycle | insert/delete/set-properties | 약 350줄·3함수 |

`resolve_table_cell`은 C3 명령뿐 아니라 아직 root에 남은 C5의 cell paragraph·formatting handler도
소비한다. 따라서 table 좌표 모듈이 정본을 소유하고 `edit` parent가 crate 내부에 좁게 다시
노출한다. `resolve_top_table`은 C3 table 명령 전용이므로 table 하위 모듈 밖으로 노출하지 않는다.
C4 `set-column-def`, C5 cell paragraph·style·formatting과 `cell_para_lens`는 이번 범위가 아니다.

core table mutation, parser·serializer·renderer 알고리즘도 C3 범위가 아니다. Stage 2에서는 기존
CLI adapter와 전용 helper를 물리적으로 이동하며 관찰 가능한 동작을 바꾸지 않는다.

## 3. 보호 계약 기준선

최신 결합 HEAD에서 다음 직접 계약 137/137을 통과했다.

| 계약 축 | 모듈 | 건수 |
|---|---|---:|
| cell content·properties | `edit_set_cell_contract`, `insert_text_in_cell_contract`, `delete_text_in_cell_contract`, `set_cell_props_contract` | 17 |
| table structure·layout | table insert/row/column/merge/split/fit/resize/property/move/delete/transpose 계약 18개 | 65 |
| equation lifecycle·golden | `insert_equation_contract`, `delete_equation_contract`, `set_equation_properties_contract`, `equation_command_goldens` | 14 |
| JSON·provenance 봉투 | `cli_json_contract`, `provenance_contract` | 41 |

계약들은 happy path, dry-run 무쓰기, 잘못된 option·좌표 거부, HWP/HWPX 형식 보존, output·verify,
table 구조·크기·속성, equation 생성·삭제·속성 변경, MCP 경로, JSON·provenance를 보호한다. 명령별
직접 계약과 공통 봉투가 이미 있으므로 move-only C3를 위한 신규 characterization은 추가하지
않는다. 관찰 가능한 공백이나 계약 차이가 발견되면 다음 이동을 멈춘다.

## 4. 복잡도 중단 조건과 처리

이동 전 `cargo clippy --locked --bin rhwp -- -W clippy::cognitive_complexity` 계측에서 C3 대상
함수의 CC 25 초과 경고는 0건이다. 저장소 library의 기존 경고 100건은 이번 CLI 이동 범위가
아니며 C3 새 경로로 옮겨 숨기지 않는다.

이동 후에도 새 C3 경로에 CC 25 초과 함수가 없는지 다시 확인한다. 파일 크기는 1,200줄 상한을
지키기 위해 cell, table coordinate, table structure, table layout/property, equation 책임으로
나눈다.

## 5. 목표 소유권

```text
src/cli/commands/edit/
├── cells.rs
├── equations.rs
└── tables/
    ├── mod.rs
    ├── coordinates.rs
    ├── structure.rs
    └── layout.rs
```

`cells.rs`는 cell text·property command와 CLI·MCP·protocol이 공유하는 글자색·폭·overflow·
control-character helper를 소유한다. `tables/coordinates.rs`가 주소 해석의 단일 원천이고,
`structure.rs`와 `layout.rs`는
구조 변경과 치수·속성 변경을 분리한다. `equations.rs`는 equation lifecycle을 소유한다.
handler만 edit parent에 `pub(super)`로 노출하고, C5가 임시로 필요한 cell resolver만
`pub(crate)` seam으로 유지한다. root wrapper나 helper 복제를 만들지 않는다.

`DocumentService`, typed error와 전역 인증 제거는 Stage 3 범위다. C3에서 광범위한
`EditContext`를 만들거나 C4 문서 구조, C5 formatting 책임을 끌어오지 않는다.

## 6. 구현·커밋 순서

1. 이 inventory와 최신 `devel` 결합 기준을 독립 커밋으로 고정한다.
2. table coordinate 정본과 cell content·property command를 이동한다.
3. table structure와 layout·property command를 책임별 하위 모듈로 이동한다.
4. equation lifecycle command를 이동한다.
5. 137개 직접 계약과 전체·정적·정책 관문을 실행하고 완료 보고서를 커밋한다.

각 절편은 stdout/stderr, JSON, exit code, table 주소, output 이름·형식, 원본 보호와 파일 부작용을
보존한다. 구현 커밋마다 해당 focused 계약과 format·diff 검사를 실행한다.

## 7. 원격 위험과 중단 기준

조사 시점 열린 devel 대상 PR #5689, #5691, #5695, #5707의 최신 변경 경로에는 `src/main.rs`,
edit command, C3 직접 계약과 #5511 C3 문서가 없다. 기준선 통합 뒤 현재 브랜치는
`upstream/devel`보다 48개 커밋 앞, 0개 뒤다.

다음 경우에는 같은 승인 배치 안에서도 이동을 멈추고 메인테이너에게 보고한다.

- 137개 기준 계약의 stdout/stderr·exit·좌표·구조·속성·저장 방어가 달라지는 경우
- 새 파일 1,200줄 또는 CC 25 상한을 지킬 수 없는 경우
- root와 새 모듈의 양방향 참조나 cell·table·equation 사이 helper 복제가 생기는 경우
- 최신 devel·열린 PR이 같은 함수, 테스트, 모듈 경계를 변경한 경우
- move-only 범위를 넘어 core mutation, parser·serializer·renderer 알고리즘 변경이 필요한 경우
