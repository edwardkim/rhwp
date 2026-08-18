# rhwp-explore 레퍼런스

이 디렉터리는 `rhwp explore` 한 명령을 실 에이전트가 소비하기 위한 장이다.
gym 경로가 아니다. 새 하위명령도 새 플래그도 없다.

생성: `python references/_gen_pack.py` (이슈 #5313).

## 읽는 순서

1. 세 축이 헷갈리면 `00_three_axes.md`
2. 첫 수가 필요하면 `01_first_move.md`
3. 봉투 키는 `02_envelope.md`
4. 메뉴가 문서마다 다른 이유는 `03_menu_priority.md`
5. 여덟 어포던스는 `04_routing_table.md` 와 `08`–`14`
6. 외부 문서는 `05_security_first.md` 를 빼먹지 않는다
7. 실패·암호·빈 파일은 `07_exceptions.md`

기계 가독 자료는 스킬 루트의 `fixtures/` 다.
일한 예는 `examples/` 다.

## 권위

- `mydocs/manual/cli_commands.md` 의 `explore` 절
- `src/document_core/queries/explore.rs`
- 이 스킬은 그 표면을 복제할 뿐 바꾸지 않는다
