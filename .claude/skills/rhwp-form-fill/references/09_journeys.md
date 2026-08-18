# 09 — 실사용 여정

gym 여정이 아니다. 사람이 에이전트에게 실제로 거는 요청이다.
기계 목록은 `fixtures/journeys.json`.

## J01 조사만

요청: "이 신청서에 뭘 채워야 해?"

```
fields --json → names/guide/memo 보고 정지 (F04)
```

채우지 않는다. dry-run 도 하지 않는다.

## J02 단건 채움+검증

요청: "홍길동으로 한 장 채워. 제대로 들어갔는지도."

```
fields → row.json (name 복사) → dry-run → fill-fields -o --verify
통과: F07
```

표본: `samples/form-01.hwp`, 키 `myMsg01`.

## J03 제출본

요청: "제출용으로 만들어. 작성자 흔적 지워."

```
J02 → sanitize → 두 번째 sanitize removedCount 0
```

## J04 직인 포함 제출

요청: "도장 찍고 제출본."

```
J02 → insert-image --page 0 --x --y (HWPUNIT) → overflow 확인 → sanitize
```

## J05 반복 필드

요청: "목차 다섯 칸에 각각 다른 제목."

```
fields 에서 목차1 ×5 → 목차1[0]…[4] → dry-run 에 ambiguous 없음 → verify
```

표본: `samples/field-01.hwp`.

## J06 메일머지 3행

요청: "명단 세 명분 파일."

```
fields → JSONL 3줄 → batch fill --out-dir --json → 행별 게이트
```

## J07 메일머지 미리보기

요청: "만들기 전에 각 행이 채워지는지만."

```
같은 인자 + --dry-run. --out-dir 유지. 파일 없음.
```

## J08 메일머지 검증

요청: "일괄 작성하고 재파싱까지."

```
batch fill --verify → 행별 verify.identical
```

## J09 파일명을 이름으로

요청: "파일명을 성명으로."

```
--name-field 성명 → 게이트에서 성명을 notFound 비교에서 제외 (F11)
```

## J10 누름틀 없음

요청: "이 양식 채워."

```
fields fieldCount 0 → table-exchange 인계 (F02)
```

표본: `samples/hwp3-sample.hwp`.

## J11 보안 먼저

요청: "받은 서식인데 채워."

```
fields.textSecurity != clean → security-sweep. 값을 넣지 않음 (F03)
```

## J12 오타

요청: "이름 칸에 홍길동." (실제 키는 myMsg01)

```
dry-run notFound → fields name 을 보여 주고 키를 고침 (F06)
```

## J13 빈 명단

요청: "조회 결과로 일괄." (CSV 헤더만)

```
exit 2 → 상류 0건 (F09)
```

## J14 CP949

요청: "엑셀에서 저장한 CSV 로."

```
exit 1 UTF-8 아님 → UTF-8 재저장 (P01)
```

## J15 HWPX 보존

요청: "hwpx 그대로 채워."

```
-o out.hwpx → outputFormat hwpx. .hwp 로 바꾸지 않음
```

## J16 규제영향 14칸

요청: "피규제집단명 전부."

```
이름[0]…[13]. 선택 표본 없으면 목차1×5 로 같은 문법 (F05)
```

## J17 로고 셀

요청: "기관명도 넣어."

```
location.nested + export-tables → 그림 칸이면 키를 뺌 (P10)
```

## J18 도장 overflow

요청: "오른쪽 아래 도장."

```
dry-run insert-image → overflow 있으면 좌표 축소
```

## J19 sanitize 멱등

요청: "이미 정리한 파일 다시."

```
removedCount 0 이 정상 (F08)
```

## J20 폴더 수백

요청: "forms/ 전부 채워."

```
서식마다 명단이 다르면 이 스킬을 N번. 선별은 batch fields.
파이프라인 전체는 bulk-pipeline (F13)
```

## 확장 여정 (J21+)

`fixtures/journeys.json` 에 휴가원·지출결의·수료증·정보공개 등 실무
서식 이름을 붙인 행이 더 있다. 명령 집합은 위 20개와 같고, 제목만
다르다. 새 동사를 만들지 않는다.

각 행의 `stop` 은 F01–F14 중 하나여야 한다. 테스트가 이를 고정한다.
