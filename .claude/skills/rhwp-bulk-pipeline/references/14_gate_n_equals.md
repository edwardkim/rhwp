# 14 — 게이트: 입력 N = 성공 + 실패

배치가 끝난 것이 아니라 **숫자가 맞은 것**이 끝난 것이다.

## 공식

```bash
입력=$(wc -l < 목록.txt)
성공=$(jq -s '[.[]|select(.error|not)]|length' 결과.ndjson)
실패=$(jq -s '[.[]|select(.error)]|length' 결과.ndjson)
echo "입력 $입력 = 성공 $성공 + 실패 $실패"
test "$입력" -eq $((성공 + 실패))
```

레시피 9 실측: **입력 5 = 성공 4 + 실패 1**.

fill 축은 입력이 목록이 아니라 데이터 행 수다.

```bash
입력=$(jq -s 'length' 행.jsonl)   # 또는 csv 데이터 행
```

## 안 맞을 때

결과 파일을 의심하지 말고 **파이프 중간**을 의심한다.

| 용의자 | 증상 |
| --- | --- |
| `head` / `Select-Object -First` | 앞 N줄만 남음 |
| `grep` / `Select-String` | 줄이 사라지거나 요약이 섞임 |
| `2>&1` | stderr 요약이 줄 수를 오염 |
| Broken pipe | 소비자가 먼저 죽음. 뒤 행 증발 |
| 인코딩 | 한 줄이 여러 줄로 깨짐 |
| 빈 줄이 목록에 | 입력 N 이 실제 파일 수보다 큼 |

## 스트리밍 카운트

거대 코퍼스에서 `jq -s` 는 메모리를 다 먹는다.

```bash
성공=$(jq -c 'select(.error|not)' 결과.ndjson | wc -l)
실패=$(jq -c 'select(.error)' 결과.ndjson | wc -l)
```

Windows 는 `29_windows_powershell.md` 의 카운트 함수.

## 종료 코드와 게이트는 다른 층

exit 1 은 "실패가 하나라도 있다"이지 "줄이 맞다"가 아니다.
exit 0 이어도 파이프가 줄을 잘라 N 이 안 맞을 수 있다 — 그래서 숫자를 센다.

사용법 오류(exit 2)는 줄이 0 이다. 게이트는 `입력 == 0 && 성공 == 0 && 실패 == 0`
이 아니라 **호출 자체를 다시** 하는 쪽이다. 빈 성공으로 위장하지 않는다.

## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `14_gate_n_equals.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
