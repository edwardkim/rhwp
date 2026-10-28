# #5511 Stage 2 기능군 배치 C5 inventory — formatting command

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 조사 HEAD: `2f4034df360dbfeacd267186b50ef2260c93b26a`
- 통합 기준선: `upstream/devel` `b32113be61aefab049d03d6ab618c217104c080c`
- 작성일: 2026-08-20
- 상태: C5 진입 승인 — 실범위·보호 공백·책임 경계 고정

## 1. 최신 기준선과 원격 위험

C5 진입 직전 `upstream/devel`과 `origin/devel`은 모두 `b32113be6`이며 C4 완료 뒤 전진하지
않았다. 현재 브랜치는 원격보다 62개 커밋 앞, 0개 뒤다. 작업 트리는 깨끗하고 이슈 #5511은
메인테이너에게 할당된 열린 상태다.

열린 devel 대상 PR은 #5736 하나다. 최신 head `adef36628c9c0738fc2242667cb3d16da7212e66`은
renderer table layout, 전용 회귀 source·fixture와 sample을 변경한다. C5의 `src/main.rs`, edit
formatting 경로와 직접 계약에는 겹침이 없다. 이 판정은 시점 증거이므로 완료와 push 직전에
다시 조회한다.

## 2. 실범위 판정

마스터 계획은 C5를 1,720줄·13함수로 계측했다. C0~C4 이동 뒤 현재 함수 경계와 소유권을 다시
대조하면 C5의 실제 범위는 1,289줄·8 handler와 전용 helper 1개다.

| 책임 | handler·helper | 현재 규모 |
|---|---|---:|
| cell paragraph structure | split/merge paragraph in cell | 301줄·2함수 |
| body formatting | apply char/paragraph format, apply style | 372줄·3함수 |
| cell formatting | apply char/paragraph format, apply style, `cell_para_lens` | 616줄·3함수+helper 1개 |

기존 계측에 포함된 header/footer·footnote·endnote formatting은 C6 story 경계에서 lifecycle·본문
편집과 함께 다룬다. C5는 본문과 table cell의 paragraph 구조·formatting만 이동하며 core
mutation, parser·serializer·renderer 알고리즘은 바꾸지 않는다.

## 3. 보호 계약 기준선과 공백

최신 HEAD에서 C5 전용 계약 7개와 공통 JSON·provenance 계약을 합친 69/69가 통과했다.

| 계약 축 | 모듈 | 건수 |
|---|---|---:|
| cell paragraph structure | `split_paragraph_in_cell_contract`, `merge_paragraph_in_cell_contract` | 8 |
| body formatting | `apply_char_format_contract`, `apply_para_format_contract`, `apply_style_contract` | 12 |
| cell formatting | `apply_char_format_in_cell_contract`, `apply_para_format_in_cell_contract` | 8 |
| JSON·provenance 봉투 | `cli_json_contract`, `provenance_contract` | 41 |

`apply-cell-style`은 capabilities·help·MCP metadata에는 등록돼 있으나 실제 CLI option, dry-run,
저장 후 cell paragraph style id와 실패 stdout을 함께 검증하는 integration source가 없다. 이는
move-only 이동 전에 닫아야 할 characterization 공백이다. 다음 네 계약을 독립 커밋으로 먼저
추가한다.

1. style 적용 후 저장본의 대상 cell paragraph `style_id`가 바뀌고 JSON 주소가 일치한다.
2. dry-run은 출력 파일을 만들지 않고 요청 주소·style을 보고한다.
3. 잘못된 style 범위와 알 수 없는 option은 usage exit와 빈 stdout을 유지한다.
4. MCP tool 등록을 유지한다.

이 계약은 현재 동작을 관찰해 고정할 뿐 core style 상속·직접 서식 알고리즘을 바꾸지 않는다.
characterization이 현재 HEAD에서 통과하지 않거나 저장값과 성공 봉투가 다르면 C5 이동을 중단한다.

## 4. 공유 seam과 목표 소유권

cell command 다섯 개는 C3에서 정본화한 `resolve_table_cell`과 `CellResolveError`를 사용한다.
이를 복제하거나 formatting 모듈로 옮기지 않고 `edit` parent의 좁은 crate 내부 seam으로 계속
사용한다. `apply-char-format-in-cell`만 `--table/--row/--col`과 내부
`--section/--para/--ctrl/--cell` 주소를 함께 지원하므로 전용 `cell_para_lens` helper를 formatting
모듈에 같이 둔다. 공통 load·finish/verify/write 수명주기는 C0 runtime을 그대로 사용한다.

이동 전 `cargo clippy --locked --bin rhwp -- -W clippy::cognitive_complexity` 계측에서 C5 대상
함수의 CC 25 초과 경고는 0건이다. 저장소 library의 기존 경고는 이번 adapter 이동 범위가
아니며 새 경로로 옮겨 숨기지 않는다.

```text
src/cli/commands/edit/
├── cell_paragraphs.rs # cell paragraph split·merge
└── formatting.rs      # body·cell char/para/style formatting과 전용 helper
```

예상 크기는 각각 약 320줄과 1,010줄로 1,200줄 상한 안이다. handler만 edit parent에
`pub(super)`로 노출하고 root wrapper, helper 복제와 양방향 참조는 만들지 않는다.
`DocumentService`, typed error와 전역 인증 제거는 Stage 3 입력으로 남긴다.

## 5. 구현·커밋 순서

1. 이 inventory를 독립 커밋으로 고정한다.
2. `apply-cell-style` characterization 4건을 원본 integration source로 추가한다.
3. cell paragraph split·merge command를 이동한다.
4. body·cell formatting command와 전용 helper를 이동한다.
5. 직접 계약과 전체·정적·정책 관문을 실행하고 완료 보고서를 커밋한다.

각 구현 절편은 stdout/stderr, exit code, JSON 주소, output 이름·형식, dry-run 무쓰기, HWP/HWPX
형식 보존과 verify 수명주기를 유지한다.

## 6. 중단 조건

- 새 characterization이 현재 HEAD에서 통과하지 않거나 현재 동작을 정상 규약으로 확정하기 어렵다.
- 기존 직접 계약의 stdout/stderr·exit·좌표·저장 결과가 달라진다.
- 새 파일이 1,200줄 또는 함수가 CC 25 상한을 넘는다.
- root와 새 모듈의 양방향 참조나 C5/C6 사이 helper 복제가 생긴다.
- 최신 devel·열린 PR이 같은 함수, 테스트, 모듈 경계를 변경한다.
- move-only 범위를 넘어 core style 상속, parser·serializer·renderer 변경이 필요하다.
