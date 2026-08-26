# rhwp-bulk-pipeline 레퍼런스

이 폴더는 실 에이전트가 폴더 단위 HWP/HWPX 를 `rhwp batch` 로 처리할 때
여는 장이다. SKILL.md 는 30초 내비게이터이고, 축·게이트·함정은 여기 있다.

읽기 순서:

1. `00_tree.md` 로 축을 고른다.
2. `01`–`03` 으로 stdin/NDJSON/실패 봉투/순서 계약을 확인한다.
3. `04`–`12` 에서 해당 축만 연다. 필요 없는 축은 읽지 않는다.
4. `13`–`18` 로 분리·재시도·게이트·암호 금지·이름 예약·fill 입력·종료 집계를 닫는다.
5. `19`–`30` 은 함정·인계·여정·발화·봉투 표·stderr·목록·트레이스·jq·재시도 부류·PowerShell·표본.

예제 레시피는 `../examples/`. NDJSON 전사는 `../examples/transcripts/`.
기계 가독 픽스처는 `../fixtures/` (테스트가 읽는다).

금지: gym 과제, 새 batch 서브커맨드, 전역 `--password`, fill 에 stdin 목록,
convert 부분 쓰기, 실패 행 침묵 삭제.
