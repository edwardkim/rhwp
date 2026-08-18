# 예제 — 에이전트 계약 정합

이슈 #5324. playbook 예시 5. gym 아님.

## 정답지

`mydocs/manual/cli_commands.md` 종료 코드 표와 `capabilities`
자기서술. 문서가 권위이고 구현이 어긋나면 구현 격차다.

## 명령

```bash
rhwp capabilities
rhwp export-svg --help
rhwp search --json --limit 1 -- 없는단어  파일.hwp
rhwp ir-diff A.hwp B.hwp --json; echo $?
```

## 읽는 법

자기서술과 실제 가용성, 파싱 규약, 절단·유실을 봉투가 숨기는지.
이미 남은 이슈(#3349 #3353 #3355 #3357 #3359 #3366)는 F14 로
devel 생존을 확인한다. 콘솔 깨짐은 F10.

관련: `references/11_exit_json_contract.md`.
