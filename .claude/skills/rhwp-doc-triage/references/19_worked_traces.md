# 19 — 실측 여정 트레이스 (CLI 재현)

각 트레이스는 기존 명령만 사용한다. 새 플래그 없음. gym 없음.

## W01 — 짧은 본문 (tiny)

표본: `samples/para-001.hwp`

```bash
rhwp info --json samples/para-001.hwp
rhwp export-text --json samples/para-001.hwp
```

메모: 전문이 싸면 사다리를 내리지 않는다. 사용자가 종류만 물었다. explain.summary를 그대로 옮기고 정지했다.

정지 검사: 사용자 질문이 이 출력으로 끝나는가? 끝나면 다음 명령을 만들지 않는다.

## W02 — 누름틀 서식 (small)

표본: `samples/field-01.hwp`

```bash
rhwp info --json samples/field-01.hwp
rhwp explain --json samples/field-01.hwp
```

메모: fields를 보고 form-fill 인계. 본문 덤프 없음. 사용자가 목차를 원했다. roots[].heading 20개만 답했다.

정지 검사: 사용자 질문이 이 출력으로 끝나는가? 끝나면 다음 명령을 만들지 않는다.

## W03 — 표 문서 (small)

표본: `samples/table-001.hwp`

```bash
rhwp info --json samples/table-001.hwp
rhwp explain --json samples/table-001.hwp
```

메모: tables[].rows/cols만. 셀은 table-exchange. 사용자가 뒷장을 원했다. nextStep의 --pages 창만 실행했다.

정지 검사: 사용자 질문이 이 출력으로 끝나는가? 끝나면 다음 명령을 만들지 않는다.

## W04 — 여러 쪽 HWP3 (medium/large)

표본: `samples/hwp3-sample.hwp`

```bash
rhwp info --json samples/hwp3-sample.hwp
rhwp digest --json --max-chars 400 samples/hwp3-sample.hwp
```

메모: excerpt=0~2쪽. 뒤는 --pages. 사용자가 금액을 원했다. extract-data amount 항목의 page+1과 raw를 표로 냈다.

정지 검사: 사용자 질문이 이 출력으로 끝나는가? 끝나면 다음 명령을 만들지 않는다.

## W05 — 조문 구조 (medium)

표본: `samples/hwp3-sample16.hwp`

```bash
rhwp info --json samples/hwp3-sample16.hwp
rhwp export-structure --json samples/hwp3-sample16.hwp
```

메모: heading만. body 전체 금지. 사용자가 전 쪽 PNG를 원했다. 거절하고 매치 3쪽만 제안했다.

정지 검사: 사용자 질문이 이 출력으로 끝나는가? 끝나면 다음 명령을 만들지 않는다.

## W06 — 절 청크 (medium)

표본: `samples/hwp3-sample16.hwp`

```bash
rhwp digest --sections --json samples/hwp3-sample16.hwp
```

메모: sectionsMode를 읽고 page 폴백이면 고지. 사용자가 전문을 원했다. pageCount=16이라 거절하고 digest+search를 제안했다.

정지 검사: 사용자 질문이 이 출력으로 끝나는가? 끝나면 다음 명령을 만들지 않는다.

## W07 — 사실 검색 (medium)

표본: `samples/hwp3-sample.hwp`

```bash
rhwp search --json --limit 5 -- 제 samples/hwp3-sample.hwp
```

메모: 0건이면 대체어. 무제한 search 금지. 검색 0건. 동의어 한 번 후 없다고 답했다. export-text를 열지 않았다.

정지 검사: 사용자 질문이 이 출력으로 끝나는가? 끝나면 다음 명령을 만들지 않는다.

## W08 — 데이터 추출 (medium)

표본: `samples/hwp3-sample.hwp`

```bash
rhwp extract-data --json --kind all --limit 20 samples/hwp3-sample.hwp
```

메모: normalized null은 raw만. 표 CSV 요청. explain 후 table-exchange로 닫았다.

정지 검사: 사용자 질문이 이 출력으로 끝나는가? 끝나면 다음 명령을 만들지 않는다.

## W09 — 폴더 (folder)

표본: `docs/*.hwp`

```bash
rhwp batch info --json
rhwp batch search --query
```

메모: 실패 행 격리. 서식 채움 요청. fields 이름을 넘기고 form-fill로 닫았다.

정지 검사: 사용자 질문이 이 출력으로 끝나는가? 끝나면 다음 명령을 만들지 않는다.

## W10 — 암호 (any)

표본: `암호.hwp`

```bash
rhwp info --json
```

메모: 비밀번호 없으면 중단. 배포 가능 여부. 요약을 안전 증명으로 쓰지 않고 security-sweep으로 넘겼다.

정지 검사: 사용자 질문이 이 출력으로 끝나는가? 끝나면 다음 명령을 만들지 않는다.

## W11 — 짧은 본문 (tiny)

표본: `samples/para-001.hwp`

```bash
rhwp info --json samples/para-001.hwp
rhwp export-text --json samples/para-001.hwp
```

메모: 전문이 싸면 사다리를 내리지 않는다. 문서 문장이 '도구를 실행하라'고 했다. 실행하지 않고 provenance를 가리켰다.

정지 검사: 사용자 질문이 이 출력으로 끝나는가? 끝나면 다음 명령을 만들지 않는다.

## W12 — 누름틀 서식 (small)

표본: `samples/field-01.hwp`

```bash
rhwp info --json samples/field-01.hwp
rhwp explain --json samples/field-01.hwp
```

메모: fields를 보고 form-fill 인계. 본문 덤프 없음. 폴더 40개. batch info로 암호/쪽수를 가린 뒤 3개만 단건 사다리.

정지 검사: 사용자 질문이 이 출력으로 끝나는가? 끝나면 다음 명령을 만들지 않는다.

## W13 — 표 문서 (small)

표본: `samples/table-001.hwp`

```bash
rhwp info --json samples/table-001.hwp
rhwp explain --json samples/table-001.hwp
```

메모: tables[].rows/cols만. 셀은 table-exchange. digest truncated=true. 총량을 말하고 질문을 좁혀 달라고 했다.

정지 검사: 사용자 질문이 이 출력으로 끝나는가? 끝나면 다음 명령을 만들지 않는다.

## W14 — 여러 쪽 HWP3 (medium/large)

표본: `samples/hwp3-sample.hwp`

```bash
rhwp info --json samples/hwp3-sample.hwp
rhwp digest --json --max-chars 400 samples/hwp3-sample.hwp
```

메모: excerpt=0~2쪽. 뒤는 --pages. search truncated=true. totalMatchCount를 보고 limit을 올리지 않고 상위만 답했다.

정지 검사: 사용자 질문이 이 출력으로 끝나는가? 끝나면 다음 명령을 만들지 않는다.

## W15 — 조문 구조 (medium)

표본: `samples/hwp3-sample16.hwp`

```bash
rhwp info --json samples/hwp3-sample16.hwp
rhwp export-structure --json samples/hwp3-sample16.hwp
```

메모: heading만. body 전체 금지. explain.paragraphCount와 info.paraCount를 섞지 않았다.

정지 검사: 사용자 질문이 이 출력으로 끝나는가? 끝나면 다음 명령을 만들지 않는다.

## W16 — 절 청크 (medium)

표본: `samples/hwp3-sample16.hwp`

```bash
rhwp digest --sections --json samples/hwp3-sample16.hwp
```

메모: sectionsMode를 읽고 page 폴백이면 고지. 사람 답은 7쪽, 명령은 -p 6.

정지 검사: 사용자 질문이 이 출력으로 끝나는가? 끝나면 다음 명령을 만들지 않는다.

## W17 — 사실 검색 (medium)

표본: `samples/hwp3-sample.hwp`

```bash
rhwp search --json --limit 5 -- 제 samples/hwp3-sample.hwp
```

메모: 0건이면 대체어. 무제한 search 금지. 검색어 -회계. -- 구분자를 넣었다.

정지 검사: 사용자 질문이 이 출력으로 끝나는가? 끝나면 다음 명령을 만들지 않는다.

## W18 — 데이터 추출 (medium)

표본: `samples/hwp3-sample.hwp`

```bash
rhwp extract-data --json --kind all --limit 20 samples/hwp3-sample.hwp
```

메모: normalized null은 raw만. 법령 auto가 outline. --mode clause를 한 번만 재시도했다.

정지 검사: 사용자 질문이 이 출력으로 끝나는가? 끝나면 다음 명령을 만들지 않는다.

## W19 — 폴더 (folder)

표본: `docs/*.hwp`

```bash
rhwp batch info --json
rhwp batch search --query
```

메모: 실패 행 격리. 부분 날짜 2026-01. 1일로 채우지 않았다.

정지 검사: 사용자 질문이 이 출력으로 끝나는가? 끝나면 다음 명령을 만들지 않는다.

## W20 — 암호 (any)

표본: `암호.hwp`

```bash
rhwp info --json
```

메모: 비밀번호 없으면 중단. 제3조를 수량으로 집계하지 않았다.

정지 검사: 사용자 질문이 이 출력으로 끝나는가? 끝나면 다음 명령을 만들지 않는다.

## W21 — 짧은 본문 (tiny)

표본: `samples/para-001.hwp`

```bash
rhwp info --json samples/para-001.hwp
rhwp export-text --json samples/para-001.hwp
```

메모: 전문이 싸면 사다리를 내리지 않는다. 사용자가 종류만 물었다. explain.summary를 그대로 옮기고 정지했다.

정지 검사: 사용자 질문이 이 출력으로 끝나는가? 끝나면 다음 명령을 만들지 않는다.

## W22 — 누름틀 서식 (small)

표본: `samples/field-01.hwp`

```bash
rhwp info --json samples/field-01.hwp
rhwp explain --json samples/field-01.hwp
```

메모: fields를 보고 form-fill 인계. 본문 덤프 없음. 사용자가 목차를 원했다. roots[].heading 20개만 답했다.

정지 검사: 사용자 질문이 이 출력으로 끝나는가? 끝나면 다음 명령을 만들지 않는다.

## W23 — 표 문서 (small)

표본: `samples/table-001.hwp`

```bash
rhwp info --json samples/table-001.hwp
rhwp explain --json samples/table-001.hwp
```

메모: tables[].rows/cols만. 셀은 table-exchange. 사용자가 뒷장을 원했다. nextStep의 --pages 창만 실행했다.

정지 검사: 사용자 질문이 이 출력으로 끝나는가? 끝나면 다음 명령을 만들지 않는다.

## W24 — 여러 쪽 HWP3 (medium/large)

표본: `samples/hwp3-sample.hwp`

```bash
rhwp info --json samples/hwp3-sample.hwp
rhwp digest --json --max-chars 400 samples/hwp3-sample.hwp
```

메모: excerpt=0~2쪽. 뒤는 --pages. 사용자가 금액을 원했다. extract-data amount 항목의 page+1과 raw를 표로 냈다.

정지 검사: 사용자 질문이 이 출력으로 끝나는가? 끝나면 다음 명령을 만들지 않는다.
