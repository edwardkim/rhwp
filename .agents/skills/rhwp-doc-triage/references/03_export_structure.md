# 03 — export-structure : 개요/조문 뼈대

목차와 조문 계층만 뽑는다. 본문 전체를 싣지 않는 것이 존재 이유다.

이슈 #3261. 권위는 `cli_commands.md` 의 `export-structure` 절.

## 트리에서의 위치

사다리: `info → explain → export-structure → digest → search → extract-data`

이 단의 목적: 개요/조문 뼈대. 본문 전체를 싣지 않는다

- 성공 다음: 목차 요청이면 정지. 더 읽으려면 digest --sections
- 실패 다음: 법령인데 outline 이면 --mode clause 재시도

## 호출

```bash
rhwp export-structure <파일> --json
```

읽기 전용. 원본을 쓰지 않는다. 새 플래그를 만들지 않는다.

## 봉투 키

| 필드 | 필수 | 읽는 법 |
| --- | --- | --- |
| schemaVersion | 필수 | 개요/조문 뼈대. 본문 전체를 싣지 않는다 |
| source | 필수 | 개요/조문 뼈대. 본문 전체를 싣지 않는다 |
| mode | 필수 | 개요/조문 뼈대. 본문 전체를 싣지 않는다 |
| nodeCount | 필수 | 개요/조문 뼈대. 본문 전체를 싣지 않는다 |
| structure | 필수 | 개요/조문 뼈대. 본문 전체를 싣지 않는다 |

`schemaVersion` 은 `"1.0"`. 필드 추가는 허용, 삭제·형 변경은 계약 테스트가 잡는다.

## 종료 코드

| 코드 | 의미 | 에이전트 행동 |
| --- | --- | --- |
| 0 | 성공. 0건 포함 | 봉투를 읽고 정지 규칙을 적용 |
| 1 | 런타임 (없음·파싱·암호 틀림) | stdout 비었는지 확인. 덤프 우회 금지 |
| 2 | 사용법 | 옵션을 고친다. 0 을 무제한으로 바꾸지 않는다 |

## 모드

- `auto` (기본): Outline head_type 최우선. `제N조` 제목만 clause 증거.
- `outline`: ParaShape 개요 수준.
- `clause`: 편·장·절·관·조 / 항①②③ / 호 / 목.

항·호·목 모양은 auto 의 clause 증거가 아니다. 번호 목록 문서가 outline 인 것은 정상.
법령인데 outline 이면 `--mode clause` 를 한 번만 재시도한다.

## 트리 읽기

비제목 문단은 직전 제목의 `body` 에 귀속된다.
에이전트는 `heading` 과 `level` 만 먼저 보고, body 는 질문된 노드만 연다.

```bash
rhwp export-structure 문서.hwp --json | jq '{mode, nodeCount, roots: [.structure.roots[].heading]}'
```

## 하지 않는 것

- roots 전체 JSON 을 프롬프트에 붙이기
- auto 결과를 마음에 안 들어 전문 덤프
- 목차 요청인데 digest excerpt 로 대체

## 운용 시나리오 C01~C60

1. 사용자가 '한 줄 요약' 라고 하면 이 명령을 쓴다 — `export-structure` 시나리오 C01.
2. 이 명령의 결과가 질문이 이미 충족 이면 즉시 정지 (S15) — `export-structure` 시나리오 C02.
3. 이 명령을 무제한 덤프 로 쓰면 컨텍스트 고갈 — `export-structure` 시나리오 C03.
4. 쪽수가 31~100쪽 일 때 이 명령의 예산은 메타만 — `export-structure` 시나리오 C04.
5. 이 명령 이후 프롬프트에 넣을 수 있는 것은 주소와 짧은 발췌 뿐이고 전문·전 셀 텍스트 은 버린다 — `export-structure` 시나리오 C05.
6. 사용자가 '표가 있는지' 라고 하면 이 명령을 쓴다 — `export-structure` 시나리오 C06.
7. 이 명령의 결과가 질문이 이미 충족 이면 즉시 정지 (S15) — `export-structure` 시나리오 C07.
8. 이 명령을 무제한 덤프 로 쓰면 컨텍스트 고갈 — `export-structure` 시나리오 C08.
9. 쪽수가 31~100쪽 일 때 이 명령의 예산은 메타만 — `export-structure` 시나리오 C09.
10. 이 명령 이후 프롬프트에 넣을 수 있는 것은 주소와 짧은 발췌 뿐이고 전문·전 셀 텍스트 은 버린다 — `export-structure` 시나리오 C10.
11. 사용자가 '사실 위치' 라고 하면 이 명령을 쓴다 — `export-structure` 시나리오 C11.
12. 이 명령의 결과가 질문이 이미 충족 이면 즉시 정지 (S15) — `export-structure` 시나리오 C12.

## 정지

이 단에서 질문이 답이면 [07_when_to_stop.md](07_when_to_stop.md) 로 간다.
다음 단은 필요 신호(표, 누름틀, 특정 어휘, 수치)가 있을 때만 연다.
