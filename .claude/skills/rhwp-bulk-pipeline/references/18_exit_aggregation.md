# 18 — 종료 집계

capabilities `batch.exitAggregation`:

| 코드 | 조건 |
| --- | --- |
| 0 | 전부 통과 |
| 1 | error 레코드가 하나라도 |
| 2 | 사용법 (레코드 없이 종료하는 경우가 대부분) |
| 3 | error 없고 `--verify` IR 차이만 |
| 4 | error 없고 `--verify-pages` 불일치 |

우선순위: 사용법(2)이 레코드보다 앞선다. 레코드가 있으면 error(1) 가
verify(3/4) 를 이긴다. verify-pages(4) 는 verify(3) 보다 높다
(둘 다 켜고 error 가 없을 때). 구현 세부는 capabilities 가 정본.

## 성공 4 + 실패 1 = exit 1

정상이다. 에이전트가 exit 1 을 "파이프 전체 실패, 결과 버리기"로
읽으면 4건의 본문을 잃는다. 행별 판정은 NDJSON.

## 사용법 2

- `--password` 계열
- `search` 에 `--query` 없음
- convert 이름 예약 충돌
- `--out-dir` 가 다음 플래그로 파싱
- fill 빈 CSV / 필수 플래그 누락

stdout 이 비면 게이트 공식에 넣지 말고 호출을 고친다.

## 3 과 4

convert/fill `--verify` 차이는 산출이 남은 채 3.
convert `--verify-pages` 불일치는 4.
둘을 한 덩어리 "검증 실패"로 뭉개면 페이지 문제와 IR 문제를 섞는다.

## 단건과의 차이

단건도 0/1/2/3 을 쓴다 (#2707). 배치의 1 은 **집계**라서
성공 행이 같은 stdout 에 있다. 단건 1 은 stdout 0바이트.

## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `18_exit_aggregation.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
