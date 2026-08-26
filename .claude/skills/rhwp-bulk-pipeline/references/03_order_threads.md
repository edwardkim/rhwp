# 03 — `--threads` 와 입력 순서 보존

`--threads <N>` 기본값은 CPU 코어 수. 파일 간 병렬이다.
출력 순서는 병렬이어도 **stdin 입력 순서**다.

## 왜 순서인가

에이전트는 `목록.txt` 의 i번째 줄과 `결과.ndjson` 의 i번째 줄을 짝낸다.
정렬 키가 섞이면 게이트 `N = 성공+실패` 는 맞아도 "어느 경로가 어느
봉투인지" 를 다시 조인해야 한다. 계약이 순서를 보장하므로 조인은
줄 번호로 끝난다.

`tests/batch_parallel_determinism_contract.rs` 가 이 계약을 고정한다.
이 스킬은 그 테스트를 바꾸지 않고 인용만 한다.

## 쓰는 법

```bash
rhwp batch export-text --json --threads 4 < 목록.txt > 결과.ndjson
```

- 디스크가 느린 네트워크 드라이브면 스레드를 낮춘다.
- CPU 가 남는 로컬 SSD 면 기본값(코어 수)을 쓴다.
- `--threads 1` 은 디버그·재현용. 순서 계약은 1이든 8이든 같다.

## 순서 보존이 아닌 것

- **완료 시각**은 뒤 파일이 먼저 끝날 수 있다. 내부 큐가 입력 순서로
  stdout 에 내보낸다. stderr 진행 메시지가 뒤섞여 보여도 stdout 은 정렬돼 있다.
- **재시도 출력**은 재시도 목록의 순서다. 원본 목록 순서로 되돌리려면
  `source` 키로 원본 NDJSON 의 실패 줄을 치환한다 (`13_jq_split_retry.md`).
- **fill** 의 순서는 데이터 파일 행 순서다. stdin 파일 목록이 없다.

## 결정성

같은 목록, 같은 바이너리, 같은 옵션이면 스레드 수와 무관하게
stdout 바이트가 같아야 한다(실패 메시지의 os 문구가 로케일에 묶인
경우는 제외 — 키 집합과 `source` 순서는 같다).

에이전트는 병렬을 "비결정적"으로 취급해 결과를 정렬하는 후처리를
넣지 않는다. 넣으면 계약이 바뀐 줄 알고 잘못된 패치를 올린다.

## 함정

- GNU `parallel` 로 `rhwp export-text` 를 파일마다 띄우면 순서는 깨지고
  실패는 단건 계약(stdout 0바이트)으로 바뀐다. 그게 필요하면 이 스킬이 아니다.
- `xargs -P` 도 같다. 파일 간 병렬은 `batch --threads` 가 이미 한다.

## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `03_order_threads.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
