# #5511 Stage 2 기능군 배치 C4 inventory — document structure command

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 조사 HEAD: `4b2268487b6c4ea16bcf31e029218578cedcb9b7`
- 통합 기준선: `upstream/devel` `1139f28d17d55b499f553354f8711ecc60b110dd`
- 작성일: 2026-08-20
- 상태: C4 진입 승인 — 실범위·보호 공백·책임 경계 고정

## 1. 최신 기준선과 원격 위험

C4 진입 직전 `upstream/devel`과 `origin/devel`은 모두 `1139f28d1`이며 C3 완료 뒤 전진하지
않았다. 현재 브랜치는 원격보다 55개 커밋 앞, 0개 뒤다.

열린 devel 대상 PR #5689, #5691, #5695, #5707, #5735의 최신 head와 변경 경로를 조회했다.
Studio, q-more, agent skill·review 통합 범위로서 C4의 `src/main.rs`, edit document-structure 경로,
직접 계약과 #5511 C4 문서에 겹침이 없다. 이 판정은 시점 증거이므로 완료와 push 직전에 다시
조회한다.

## 2. 실범위 판정

마스터 계획은 C4를 3,181줄·26함수로 계측했다. C0~C3 이동 뒤 현재 함수 경계와 소유권을 다시
대조하면 C4의 실제 범위는 2,306줄·20 handler다. 기존 계측에 함께 잡힌 header/footer·footnote
body tail은 C6, cell paragraph·formatting은 C5에 남겨야 한다.

| 책임 | handler | 현재 규모 |
|---|---|---:|
| 본문 text·paragraph·break | insert/delete text, insert/delete/merge/split paragraph, page/column break, numbering restart | 1,082줄·9함수 |
| page·section·column structure | set page/section/column def, page hide | 470줄·4함수 |
| note lifecycle | insert footnote/endnote, delete footnote | 289줄·3함수 |
| bookmark lifecycle | add/delete/rename bookmark | 368줄·3함수 |
| generic structural control | delete control | 97줄·1함수 |

`edit_delete_text_in_footnote`, footnote body text·paragraph command, header/footer lifecycle·body command는
C6 story 경계에서 함께 다룬다. cell paragraph split/merge와 format command는 C5 범위다. C4는
본문 구조 좌표와 page/section/note/bookmark/control lifecycle만 이동하며 core mutation,
parser·serializer·renderer 알고리즘은 바꾸지 않는다.

## 3. 보호 계약 기준선과 공백

최신 HEAD에서 C4 전용 계약 19개와 공통 JSON·provenance 계약을 합친 110/110이 통과했다.

| 계약 축 | 모듈 | 건수 |
|---|---|---:|
| 본문 text·paragraph·break | insert/delete text, insert/delete/merge/split paragraph, page/column break, numbering restart 계약 9개 | 34 |
| page·section | `set_page_def_contract`, `set_section_def_contract`, `set_page_hide_contract` | 12 |
| note lifecycle | `insert_footnote_contract`, `insert_endnote_contract`, `delete_footnote_contract` | 9 |
| bookmark·control lifecycle | add/delete/rename bookmark, delete-control 계약 4개 | 14 |
| JSON·provenance 봉투 | `cli_json_contract`, `provenance_contract` | 41 |

`set-column-def`는 capabilities·MCP·provenance field 등록은 검증하지만 실제 CLI option, dry-run,
저장 후 ColumnDef 값과 실패 stdout 계약을 검증하는 integration source가 없다. 이는 move-only
이동 전에 닫아야 할 characterization 공백이다. 현재 public `getColumnDef`가 count, type,
sameWidth, spacing을 결정론적으로 노출하므로 다음 네 계약을 독립 커밋으로 먼저 추가한다.

1. count/type/mixed-width/spacing 변경이 저장본과 JSON 봉투에 보인다.
2. dry-run은 출력 파일을 만들지 않고 계획 값을 보고한다.
3. 잘못된 type과 알 수 없는 option은 usage exit와 빈 stdout을 유지한다.
4. MCP tool 등록을 유지한다.

이 계약은 현재 동작을 관찰해 고정할 뿐 구현을 바꾸지 않는다. characterization이 현재 HEAD에서
통과하지 않거나 저장 후 값이 CLI 봉투와 다르면 C4 이동을 중단한다.

## 4. 복잡도와 목표 소유권

이동 전 `cargo clippy --locked --bin rhwp -- -W clippy::cognitive_complexity` 계측에서 C4 대상
함수의 CC 25 초과 경고는 0건이다. 저장소 library의 기존 경고는 이번 adapter 이동 범위가
아니며 새 경로로 옮겨 숨기지 않는다.

```text
src/cli/commands/edit/
├── document_text.rs # 본문 text·paragraph·break
├── page.rs          # page·section·column 구조
├── notes.rs         # footnote/endnote lifecycle
├── bookmarks.rs     # bookmark lifecycle
└── controls.rs      # generic structural control lifecycle
```

`document_text.rs`는 1,082줄 예상으로 1,200줄 상한 안에 있다. 나머지 파일도 책임별로 분리하고
handler만 edit parent에 `pub(super)`로 노출한다. root wrapper, helper 복제와 양방향 참조는
만들지 않는다. `DocumentService`, typed error와 전역 인증 제거는 Stage 3 입력으로 남긴다.

## 5. 구현·커밋 순서

1. 이 inventory를 독립 커밋으로 고정한다.
2. `set-column-def` characterization 4건을 원본 integration source로 추가한다.
3. 본문 text·paragraph·break command를 이동한다.
4. page·section·column command를 이동한다.
5. note, bookmark, generic control lifecycle을 이동한다.
6. 직접 계약과 전체·정적·정책 관문을 실행하고 완료 보고서를 커밋한다.

각 구현 절편은 stdout/stderr, exit code, JSON, 좌표, output 이름·형식, dry-run 무쓰기, HWP/HWPX
형식 보존과 verify 수명주기를 유지한다.

## 6. 중단 조건

- 새 characterization이 현재 HEAD에서 통과하지 않거나 현재 동작을 정상 규약으로 확정하기 어렵다.
- 기존 직접 계약의 stdout/stderr·exit·좌표·저장 결과가 달라진다.
- 새 파일이 1,200줄 또는 함수가 CC 25 상한을 넘는다.
- root와 새 모듈의 양방향 참조나 C4/C5/C6 사이 helper 복제가 생긴다.
- 최신 devel·열린 PR이 같은 함수, 테스트, 모듈 경계를 변경한다.
- move-only 범위를 넘어 core mutation, parser·serializer·renderer 변경이 필요하다.
