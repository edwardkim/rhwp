# 예제 04 — convert 이름 예약

성공 경로 (레시피 9 실측):

```bash
rhwp batch convert --out-dir out/bulk --json < examples/lists/convert_ok.txt
```

전사 `T04.ndjson`: bytes 9083392, format hwp5.

충돌 경로:

```bash
rhwp batch convert --out-dir out --json < examples/lists/convert_collision.txt
# exit 2, stdout 빈 줄, 한 파일도 안 생김
```

전사 `T08.ndjson` 은 비어 있다. `Report.HWP` 와 `report.hwp` 를
`out/case1`, `out/case2` 로 나눈다.

이슈 #5311. gym 아님. 새 CLI 아님.
