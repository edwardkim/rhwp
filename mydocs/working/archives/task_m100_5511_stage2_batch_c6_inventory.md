# #5511 Stage 2 기능군 배치 C6 inventory — story command

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 최신 통합 기준: `upstream/devel` `d5a99a6f726afeb0aa71503c80bb4128a88bacae`
- inventory HEAD: `06f8cc3c852f2f63d6f7eeff14d38b8f041c184a`
- 수행일: 2026-08-20
- 상태: C6 진입 승인 — 보호 공백 보강 뒤 move-only 진행

## 1. 최신 기준선과 병렬 변경

C6 시작 직전 `devel`이 이전 기준 `b32113be6`에서 `d5a99a6f7`로 6개 커밋 전진했다. 변경은
nextest archive 분할, q-more volume probe 공통화, renderer table body-bottom 보정과 그 증빙이다.
C6의 `src/main.rs` story handler, `src/cli/commands/edit/` dispatch와 전용 계약 source를 건드리지
않았고 merge-tree에도 충돌이 없었다. 최신 `upstream/devel`을 정상 merge한
`06f8cc3c8`을 inventory 기준으로 삼는다.

열린 devel 대상 PR은 #5739, #5741~#5745, #5754, #5758, #5762다. #5742는 footnote
renderer layout, #5762는 parser·serializer·renderer를 바꾸지만 C6의 CLI adapter 경로와 직접
겹치지 않는다. 나머지도 Studio, Docker, renderer/model 경로로 C6 source·test·문서와 경로 중첩이
없다. 이 판정은 inventory 시점 증거이며 완료와 push 직전에 다시 조회한다.

## 2. 계획 대비 실범위 보정

마스터 계획은 C6를 1,152줄·9함수로 추산했으나 실제 root에는 2,430 source line·18 handler가
남아 있다. 초기 계측에서 header/footer lifecycle 일부와 story text·paragraph tail을 서로 다른
기능군으로 중복·누락해 생긴 차이다. C5에서 일반 body/cell formatting을 분리한 뒤에는 아래 18개가
하나의 명확한 story 주소 경계를 이룬다.

| 책임 | handler | source line |
|---|---:|---:|
| header/footer lifecycle·text·paragraph·field | 8 | 1,159 |
| header/footer picture·template·visibility·format | 4 | 574 |
| footnote/endnote text·paragraph·shape·format | 6 | 697 |
| 합계 | 18 | 2,430 |

대상 handler는 다음과 같다.

- header/footer content: `edit_insert_header_footer`, `edit_delete_header_footer`,
  `edit_insert_header_footer_text`, `edit_set_header_footer_text`, `edit_delete_hf_text`,
  `edit_split_paragraph_in_hf`, `edit_merge_paragraph_in_hf`, `edit_insert_field_in_hf`
- header/footer properties: `edit_set_hf_picture`, `edit_apply_hf_template`,
  `edit_toggle_hide_hf`, `edit_apply_para_format_in_hf`
- note story: `edit_delete_text_in_footnote`, `edit_apply_endnote_shape`,
  `edit_insert_footnote_text`, `edit_split_paragraph_in_footnote`,
  `edit_merge_paragraph_in_footnote`, `edit_apply_para_format_in_footnote`

대상 함수의 기존 cognitive complexity 25 초과 경고는 0건이다. 기능 변경이나 책임 분해 없이
물리 이동할 수 있다.

## 3. 모듈 배치

단일 파일로 옮기면 1,200줄 상한을 넘으므로 story 종류와 mutation 성격으로 세 파일에 나눈다.

```text
src/cli/commands/edit/
├── header_footer_content.rs
├── header_footer_properties.rs
└── note_content.rs
```

`header_footer_content.rs`는 header/footer control과 내부 text·paragraph·field lifecycle을,
`header_footer_properties.rs`는 이미 존재하는 story의 picture·template·visibility·paragraph
properties를, `note_content.rs`는 footnote/endnote 내부 text·paragraph와 endnote shape를 소유한다.
세 파일 모두 예상 1,200줄 이하다.

C4의 body note lifecycle(`insert-footnote`, `insert-endnote`, `delete-footnote`)은 본문 control을
만들고 지우므로 기존 `notes.rs`에 유지한다. C6는 그 control 내부 story를 편집하는 command만
소유한다. C0의 load·serialize·verify·write runtime을 그대로 재사용하고 core mutation,
parser·serializer·renderer는 바꾸지 않는다.

## 4. 보호 계약

기존 C6 직접 계약은 18개 source, 71 tests다. handler별로 usage/unknown option, dry-run 무쓰기,
MCP 등록을 보호하고 `set-hf-picture`를 제외한 17개 handler는 성공 저장 결과도 직접 확인한다.
공통 JSON 31건과 provenance 10건을 합치면 이동 후 focused 관문은 현재 112건이다.

`set-hf-picture`의 기존 3건은 dry-run, unknown option, MCP 등록뿐이다. dry-run은 mutation을
호출하지 않으므로 현재 fixture의 지정 주소가 실제 header/footer picture인지 검증하지 않는다.
따라서 command wiring이나 성공 저장 경로가 깨져도 기존 계약은 통과할 수 있다.

이 공백은 이동 전에 실제 header/footer 안에 picture가 있는 fixture를 구성하고 width 변경을
저장·재파싱해 확인하는 characterization 1건으로 보강한다. 이 테스트가 현 구현에서 통과한 뒤에만
handler 이동을 시작한다.

## 5. 실행 순서와 중단 조건

1. 기존 C6 71건과 공통 JSON·provenance 41건의 기준선을 확인한다.
2. `set-hf-picture` 성공 저장 characterization을 독립 커밋으로 고정한다.
3. note story와 header/footer content·properties를 세 모듈로 이동한다.
4. 각 이동 커밋마다 관련 focused test, fmt와 diff check를 실행한다.
5. 최종 HEAD에서 전체 release-test, fmt, clippy, doc-test, integration manifest, unit-tier와 CI
   impact 정책을 실행한다.
6. 실제 줄 수·계약·원격 위험을 C6 완료 보고서와 마스터 계획서에 반영한다.

성공 JSON·exit/stdout, 저장 결과, story 주소, parser·serializer·renderer 동작이 달라지거나 새
모듈이 1,200줄을 넘으면 다음 handler로 진행하지 않는다. `set-hf-picture` characterization이 현
구현에서 실패하면 정상 규약으로 고정하지 않고 원인과 수정 선택지를 메인테이너에게 보고한다.
