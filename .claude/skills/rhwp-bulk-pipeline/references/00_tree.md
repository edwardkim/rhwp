# 00 — 판단 트리

폴더를 받으면 본문부터 뽑지 않는다. 목록 → info 선점검 → 질문과 맞는 축.
한 축이 답을 내면 다음 축으로 내려가지 않는다 (정지 B17).

```
사용자 폴더
  │
  ├─ 파일 목록이 없다 ──▶ 25_listing / 29_windows_powershell
  ├─ 한 문서만 파악 ──▶ rhwp-doc-triage 인계 (20)
  ├─ 서식 1 + 명단 N ──▶ batch fill (12). stdin 목록 금지 (17)
  └─ 문서 N
       ├─ 규모/형식 ──▶ batch info (04)
       ├─ 본문 ──▶ export-text (05)
       ├─ 개요/조문 ──▶ export-structure (06)
       ├─ 표 ──▶ export-tables (07)
       ├─ 누름틀 조사 ──▶ fields (08)
       ├─ 전역 검색 ──▶ search --query (09)
       ├─ 날짜·금액·수량 ──▶ extract-data (10)
       └─ HWP5 변환 ──▶ convert --out-dir (11, 16)
```

## 공통 후처리

모든 stdin 축은 같은 꼬리를 단다.

1. stdout 을 `결과.ndjson` 에만 태운다. stderr 는 터미널 또는 `요약.err`.
2. `jq 'select(.error)'` 로 실패 행을 가른다.
3. 재시도 부류를 본다 (`28_retry_classes.md`). 경로 오타는 재시도하지 않는다.
4. 게이트: 입력 줄 수 = 성공 + 실패 (`14_gate_n_equals.md`).

## 축을 섞지 않는다

`batch` 한 호출은 축 하나다. info 와 export-text 를 한 프로세스에 섞는
플래그는 없다. 목록 파일을 재사용해 호출을 나눈다.

## fill 은 이 트리의 옆가지

메일머지는 문서 N 이 아니라 서식 1 + 데이터 N 이다. 폴더에 서식이 수백
개 있고 명단이 하나면 그 수백은 `batch fields` 로 조사하고, 채울 서식
하나를 고른 뒤 `batch fill` 로 넘어간다. 수백 서식 × 명단 1 을 한 번에
돌리는 명령은 없다 — 발명하지 않는다.

## 암호 문서

info 실패 문구에 암호/encrypted 가 보이면 그 경로를 목록에서 빼 단건
`--password` 로 처리한다. 나머지 평문은 그대로 batch. 전역 플래그를
붙이면 평문까지 전부 exit 2 로 죽는다 (`15_no_global_password.md`).

## 정지 규칙 한눈에

- `B01` — 목록이 비었거나 경로가 디렉터리 → find/Get-ChildItem 부터. 본작업 금지
- `B02` — info 전건 error → 작업 디렉터리·상대경로 확인
- `B03` — batch 에 --password 계열 → exit 2. 단건 명령으로 분리
- `B04` — 질문이 규모/형식뿐 → info 에서 정지
- `B05` — export-text error 행 → jq 로 실패만 재시도
- `B06` — export-structure --mode 오타 → exit 2. auto|outline|clause
- `B07` — tableCount 0 → 실패 아님. 표 없는 문서
- `B08` — fieldCount 0 → 누름틀 없음. table-exchange 후보
- `B09` — search --query 없음 → exit 2. 입력 미소비
- `B10` — extract-data truncated → 문서마다 한도. counts 는 절단 전
- `B11` — convert 이름 충돌 → exit 2, 한 파일도 안 씀
- `B12` — fill 에 stdin 목록 → --form + --data 로 다시
- `B13` — N ≠ 성공+실패 → 파이프 중간(head/grep) 의심
- `B14` — exit 1 인데 실패 행 안 보임 → stderr 요약을 stdout 과 섞지 말 것
- `B15` — verify IR 차이 → exit 3. 산출은 남음
- `B16` — verify-pages 불일치 → exit 4 (error 없을 때)
- `B17` — 질문이 이미 답 → 다음 축으로 내려가지 않음
- `B18` — --out-dir 가 - 로 시작 → ./-결과 로 명시

## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `00_tree.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
