---
kind: working-note
status: active
canonical: mydocs/working/gym_oracle_probe.md
last_verified: 2026-08-18
related:
  - gym/tools/oracle_probe.py
  - gym/tools/README.md
  - gym/packs/oracle-probe/README.md
  - scripts/tests/test_gym_oracle_probe.py
  - "#5207"
  - "PR #5214"
---

# 작업 노트 — 라이브 오라클 이중 계산 프로브

이 노트는 구현 해설이 아니라 **왜 이렇게 만들었는가** 와 **무엇이
남았는가** 를 남긴다. 도구의 정본 API 는 `gym/tools/README.md` 다.
과제의 지도는 `gym/packs/oracle-probe/README.md` 다.

작성 맥락: PR #5214 (`feat/gym-oracle-probe`) 를 프로브 모듈만으로
두지 않고, 예외 시험·팩·작업 노트까지 확장한 기록.

------------------------------------------------------------------------

## 1. 왜 이중 계산인가

채점의 유혹은 간단하다. 어제 센 숫자를 파일에 적어 두고, 오늘
에이전트가 그 숫자를 맞히는지 본다. 빠르고, 바이너리가 필요 없고,
CI 가 가볍다.

그 유혹이 깨지는 순간은 항상 같다.

- 페이지네이션을 고친다. 쪽수가 17 에서 18 이 된다.
- 검색 토크나이저를 고친다. '국어' 매치가 12 에서 11 이 된다.
- 필드 추출이 숨은 누름틀을 하나 더 본다.

골든 파일을 믿으면 **정직한 에이전트가 실패** 하고 **숫자를 외운
에이전트가 통과** 한다. 운동장이 측정하는 것은 "문서를 열어 값을
읽는 능력" 이 아니라 "어제 커밋을 외우는 능력" 이 된다.

이중 계산은 그 순서를 뒤집는다.

1. 에이전트가 값을 보고한다.
2. 채점기가 **같은 입력에 같은 명령을 지금** 실행한다.
3. 두 값이 같으면 통과다.

어제 18 이 정답이었는지 아무도 묻지 않는다. 오늘의 바이너리가
내는 값이 정답이다. 픽스처가 진화하면 정답도 따라 진화한다.
`gym/README.md` 가 "채점은 라이브다" 라고 쓴 문장의 구현이 이것이다.

이중 계산이 성립하려면 오라클 함수 자체가 결정적이어야 한다.
같은 명령을 두 번 돌려 다른 봉투가 나오면, 채점기는 동전 던지기다.
`probe_determinism` 이 그 동전을 감사한다.

자리표가 남으면 오라클이 엉뚱한 파일을 연다. `{sub:out.hwp}` 가
리터럴 파일명으로 남으면 제출 폴더가 아니라 작업 디렉터리의
`{sub:out.hwp}` 를 찾는다. 부재가 나고, 혹은 더 나쁘게 다른
제출물의 파일을 읽는다. `probe_placeholders` 가 그 사고를 막는다.

산출물이 없는데 통과하면, 아무 것도 하지 않은 에이전트가 만점이다.
`probe_missing_artifact` 가 그 구멍을 닫는다.

세 프로브는 채점기의 세 전제다. 과제가 늘어날수록 이 전제가
흔들리면 피해가 pack 수만큼 곱해진다.

------------------------------------------------------------------------

## 2. 위협 모델 — 썩은 골든 파일

### 2.1 공격자 / 실패 주체

이 노트에서 "공격자" 는 악의적 해커가 아니다. 다음 셋이다.

1. **바쁜 기여자.** 과제 하나를 빨리 넣고 싶어 숫자를 박제한다.
2. **오래된 베이스라인.** 지난 달 스코어카드의 `pageCount` 를
   오늘 과제에 복사한다.
3. **비결정 오라클.** 시계, 난수, 해시 시드, 파일 순서에 의존하는
   명령을 라이브 오라클로 등록한다.

셋 모두 의도는 선하다. 결과는 운동장의 거짓 점수다.

### 2.2 자산

- 과제 JSON 의 `checks` — 무엇이 정답인가를 선언하는 곳.
- 기준 풀이 `reference/` — 제출물을 재현하는 절차.
- 스코어카드 — 외부에 보이는 점수.
- `oracle_probe.py` — 전제를 감사하는 도구.

### 2.3 위협 T1 — 박제된 기대값

과제에 `"value": 17` 과 `"path": "pageCount"` 를 같이 적는다.
페이지네이션이 바뀌면 정직한 제출이 실패한다.

완화: 쪽수·건수·개수는 `answer_eq` / `len_answer_eq` 로만 묻는다.
상수는 "이 좌표가 이 값이어야 한다" 가 문서 불변이 맞을 때만
쓴다. 예: `format == "hwp"` 는 표본이 hwp 인 한 불변이다.

oracle-probe pack 은 쪽수에 `value_eq` 를 쓰지 않는다.

### 2.4 위협 T2 — 기준 풀이의 숫자 리터럴

`reference/OP01.json` 에 `"pages": 1` 을 적으면 기준 풀이가
라이브가 아니다. 바이너리가 바뀌어도 제출물은 1 을 낸다.

완화: 기준 풀이는 명령을 다시 적는다. `build_baseline.py` 가
그 명령을 실행해 `answer.json` 을 만든다. OP01–OP44 전부가 이
형식이다.

### 2.5 위협 T3 — 자리표 잔존

`{sub:a.hwp}` 와 `{sub:b.hwp}` 가 한 JSON 문자열에 있을 때
첫 것만 바꾸면 `b.hwp` 가 리터럴로 남는다. #4664 가 이 사고로
열렸다.

완화: `resolve_placeholders` 가 루프로 전부 바꾸고, 프로브가
`leftover` 를 검사한다. 단위 시험이 `build_baseline.resolve` 와
출력을 대조한다.

### 2.6 위협 T4 — 부재 위장

`os.path.exists` 가 거짓이면 "비교 생략, 통과" 로 읽는 채점기.
제출하지 않은 에이전트가 만점.

완화: 부재는 `ok=false`, `status=absent`. 구조 자기점검이
존재하지 않는 경로를 넣어 `ok is True` 이면 이슈를 남긴다.

### 2.7 위협 T5 — 예외를 결정적 성공으로 세기

오라클이 두 번 같은 `RuntimeError` 를 낸다. "결정적이다" 라고
통과시키면, 항상 죽는 명령이 만점 오라클이 된다.

완화: 예외는 값이 아니다. `ok=false`. 자기점검에
`determinism-exception-is-not-pass` 가 있다.

### 2.8 위협 T6 — 비결정 정규화

집합을 해시 순으로 직렬화하면 같은 값이 다른 문자열이 된다.
키 순서가 삽입 순이면 마찬가지다.

완화: `json_canonicalize` 가 `sort_keys=True` 이고, 집합은
정규화 후 정렬한다. 스냅샷은 JSON 왕복이라 원본 변이에 닫혀 있다.

### 2.9 위협 T7 — 프로브가 채점기를 import

`build_baseline` 을 import 하면 러너가 로드되고 스트림이
재설정된다. `--json` 구조 자기점검이 "환경이 깨끗한가" 가 아니라
"러너가 살아 있는가" 를 측정하게 된다.

완화: 규칙을 복제하고 테스트로 동기화한다. 프로브는 순수하다.

### 2.10 위협 T8 — pack ID / 과제 ID 충돌

리더보드가 과제 ID 로 행을 가른다. `OP01` 이 다른 pack 에도
있으면 집계가 섞인다.

완화: `audit.py` 와 `test_gym_packs.py` 가 전역 고유를 강제한다.
이 확장의 ID 접두사는 `OP` 다. 2026-08-18 기준 충돌 없음.

### 2.11 위협 T9 — 바이너리 없는 CI 에서 라이브 채점을 돌림

rhwp 가 없는 러너에서 `score.py --pack oracle-probe` 를 돌리면
전 과제가 `unavailable` 이거나 크래시한다. 부재를 0점으로
위장하면 T4 와 같은 거짓말이다.

완화: CI 의 gym 단계는 unittest + audit + oracle_probe 만 돈다.
라이브 채점은 바이너리가 있는 작업자의 왕복이다.

### 2.12 위협 T10 — 문서와 코드의 드리프트

README 가 없는 함수를 설명하거나, 시험이 없는 불변식을 주장하면
다음 기여자가 거짓 문서를 믿는다.

완화: 공개 함수 목록을 시험이 들고 있다.
`PublicSurfaceSmokeTests.PUBLIC`. 새 함수는 목록과 행복·예외
시험을 같이 넣는다.

### 2.13 수용하는 위험

- 에이전트가 `reference/` 를 읽는다. 규칙으로 금할 뿐 기술로
  막지 않는다. 채점은 여전히 라이브라 점수는 맞다. 측정되는
  능력만 바뀐다.
- rhwp 가 비결정이면 라이브 채점이 흔들린다. 프로브는 파이썬
  함수만 본다. 바이너리 결정성은 별 시험의 몫이다.
- `capabilities` 의 명령 수는 버전마다 바뀐다. OP21 은 그것이
  의도다. 숫자를 외운 제출은 버전업 후 실패해야 한다.

------------------------------------------------------------------------

## 3. 프로브 행렬

단위 시험과 자기점검이 공유하는 표. 코드가 바뀌면 이 절과
`gym/tools/README.md` §10 을 같이 고친다.

### 3.1 결정성

```
compute_fn                ok    equal   error
------------------------  ----  ------  -----
안정 dict                 T     T       없음
드리프트 카운터           F     F       없음
RuntimeError 반복         F     F       있음
TimeoutError              F     F       있음
비호출 / None             F     F       있음
첫 성공 + 둘째 예외       F     F       있음
{} / [] / None 값         T     T       없음
키 순서만 다른 두 값      T     T       없음
"2" 와 2                  T     T       없음
True 와 1 (정규화 문자)   —     다름    —
float NaN / inf           T     T       null 로 접힘
n=1 / n=True / n="2"      F     F       사용 오류
n=5 안정                  T     T       없음
n=4 드리프트              F     F       없음
```

### 3.2 자리표

```
token                     ok    leftover   notes
------------------------  ----  ---------  -----
{input}                   T     []         inputExact
plain-literal             T     []         그대로
{sub:a.hwp}               T     []         mkdir dirname
다중 {sub:} JSON          T     []         전부 치환
keep {input} embedded     T     []         치환 안 함
keep {sub:                F     있음       미닫힘
None / 1 / dict           F     []         타입
{input} + 빈 task         F     []         KeyError
중복 {sub:a}{sub:a}       T     []         names 두 번
한글 {sub:이름.hwp}       T     []         경로에 그대로
```

### 3.3 산출물

```
path                      status       ok    present
------------------------  -----------  ----  -------
존재하는 파일             present      T     T
0바이트 파일              present      T     T
PathLike 파일             present      T     T
없는 경로                 absent       F     F
디렉터리                  not-a-file   F     F
""                        invalid      F     F
None                      invalid      F     F
int / list                invalid      F     F
묶음 중 하나 부재         (묶음)       F     —
묶음 빈 목록              (묶음)       F     —
묶음 전부 present         (묶음)       T     —
```

### 3.4 묶음 (probe_live_oracle)

```
결정성   자리표        산출물        묶음 ok
------   -----------   -----------   -------
T        (생략)        (생략)        T
T        실패          (생략)        F
T        (생략)        부재          F
F        성공          present       F
T        성공          present       T
```

논리곱이다. 켜지 않은 축은 보지 않는다.

### 3.5 봉투

```
mode          필수 키
------------  ------------------------------------------
structural    kind, schemaVersion, ok, mode, exports,
              required, issues, issueCount, probes
selftest      kind, schemaVersion, ok, mode, checks,
              failed, checkCount, issueCount
```

`probes.missingArtifact.ok` 는 구조 모드에서 반드시 false.

------------------------------------------------------------------------

## 4. 설계에서 버린 대안

### 4.1 골든 JSON 을 저장소에 커밋

빠르다. 썩는다. §2.3. 버렸다.

### 4.2 프로브가 rhwp 를 직접 실행

진짜 이중 계산에 더 가깝다. CI 가 바이너리를 요구한다.
이 저장소의 gym CI 단계는 순수 파일이라는 결을 깨는다. 버렸다.
실문서 이중 계산은 pack 의 `answer_eq` 가 담당한다.

### 4.3 build_baseline 을 import

규칙의 단일 출처. 부수효과가 `--json` 을 더럽힌다. 복제 +
대조 시험을 골랐다.

### 4.4 새 clap 하위명령 `rhwp gym-oracle-probe`

에이전트가 발견하기 쉽다. 이 PR 의 금지 사항이다
("No new CLI binary"). Python 도구로 충분하다.

### 4.5 검사 연산자 `oracle_eq` 를 checks.py 에 추가

의미가 `answer_eq` 와 겹친다. pack 이 기존 연산자만 쓰라는
규약과도 어긋난다. 기존 `answer_eq` / `len_answer_eq` 로
이중 계산을 표현한다.

### 4.6 gym/README 12 pack 표에 oracle-probe 를 넣기

운동장 입문 코스를 13 으로 늘린다. 이 pack 은 채점 계약을
드러내는 확장이지 가족 코스의 새 놀이기구가 아니다.
work-receipt 확장이 표를 건드리지 않은 것과 같다.

### 4.7 편집 과제를 이 pack 에 넣기

누름틀을 채우면 이중 계산보다 편집 축을 측정한다.
objects-media / text-editing / table-editing 의 일이다.
이 pack 의 제출은 `answer.json` 만.

------------------------------------------------------------------------

## 5. pack 설계 메모

### 5.1 왜 44 과제인가

OP01–OP08 은 같은 질문(쪽·문단·표·검색)을 두 표본에 반복한다.
표본을 바꾸면 답이 바뀐다는 것을 과제가 스스로 보여야
"숫자를 외우면 된다" 가 성립하지 않는다.

OP09–OP16 은 필드·검색·추출. 스칼라(`fieldCount`)와 배열 길이
(`len(fields)`)를 짝지워 오라클 내부 정합을 드러낸다.

OP17–OP24 는 형식 표지, 폴더 스윕, capabilities, verify.
문서가 아니라 도구 자신이나 폴더를 오라클로 쓴다.

OP25–OP39 는 표본을 더 갈아 끼운다. form-01/02, exam-kor-1p,
PII 표본의 형식/쪽수. PII 표본을 넣되 마스킹하지 않는다 —
보안 pack 을 복제하지 않기 위해서다.

OP40–OP42 는 한 장의 `answer.json` 에 두 오라클을 동시에 묻는다.
한쪽만 맞으면 실패. 이중 계산의 판별력을 과제 하나로 압축한다.

OP43–OP44 는 form 표본의 표/문단. 쪽수와 다른 축.

### 5.2 왜 이 명령들인가

`pack.json` requires 는 기존 명령만:

```
capabilities dump-pages explain export-tables extract-data
fields info scan search verify
```

전부 다른 pack 이 이미 쓰고, `answer_eq` 경로가 실측된 명령이다.
새 플래그, 새 경로를 발명하지 않았다.

### 5.3 runner 신원

core-cli 와 같은 선언을 복사했다.

```
rhwpVersion: 0.8.2
rhwpCommit:  94e4790e5a6bc766b75c3c9695b290f87e3793d4
capabilitiesSha256: 2c7c41bc8952b63c4502ec0685b76990e4ece5b178f6dc28a1a495b12880af75
```

이 값은 "어느 바이너리로 기준을 세웠는가" 의 표지다.
라이브 채점의 기대값을 고정하지 않는다. 신원이 비면
`schema.validate_pack` 이 거부한다.

### 5.4 기준 풀이 형식

모든 reference 는:

```json
{
  "id": "OP01",
  "steps": [
    { "answer": { "pages": { "cmd": ["info", "{input}", "--json"], "path": "pageCount" } } }
  ]
}
```

숫자 리터럴이 없다. `len: true` 는 `len_answer_eq` 과제에만 있다.

------------------------------------------------------------------------

## 6. 시험 설계 메모

순수 시험이다. `importlib` 로 파일을 로드한다. rhwp 를 부르지
않는다. `--json` / `--selftest` 는 같은 인터프리터로 서브프로세스를
띄울 뿐 바이너리를 띄우지 않는다.

예외 경로를 많이 넣은 이유: 행복 경로만 있으면 프로브가
"항상 통과하는 도구" 가 된다. 실패해야 할 것이 실패하는지가
이 도구의 존재 이유다.

서브프로세스 시험은 세 개만 둔다 (구조 JSON, 자기점검 JSON,
인자 없는 사람 문구). 나머지는 `run()` / `parse_args()` 를
직접 부른다. Windows 에서 프로세스 기동이 느린 것을 피한다.

`load(name=...)` 에 매번 다른 모듈 이름을 주는 이유:
`sys.modules` 충돌과 테스트 격리. 같은 이름을 재쓰면 이전
모듈 전역이 남을 수 있다.

------------------------------------------------------------------------

## 7. 남은 일

이 확장이 닫지 않은 것. 후속 PR 의 재료다.

### 7.1 바이너리 왕복은 이 작업 트리에서 돌리지 않았다

`build_baseline.py --pack oracle-probe` 와
`score.py --pack oracle-probe` 는 rhwp 가 필요하다.
이 작업 트리는 sparse/격리라 바이너리 빌드를 강제하지 않았다.
메인테이너가 바이너리 있는 트리에서 한 번 왕복하면
"풀 수 있음이 실측된 과제" 계약이 닫힌다.

왕복이 실패할 수 있는 지점:

- `info.format` 필드 이름 변경.
- `explain.paragraphCount` 부재.
- `search --json -- 표` 의 `matchCount` 대 `matches`.
- `verify --expect-min-pages` 의 종료 코드. 지금 과제는
  `answer_eq` 만 있고 `expect_exits` 가 없다. verify 가
  3 을 내면 기준 풀이가 죽을 수 있다. 그 경우
  `allowExits` / 과제의 `expect_exits` 를 맞추면 된다.
- `scan samples/hml` 의 상대 경로. 채점기는 저장소 루트에서
  실행한다고 가정한다.

### 7.2 verify 종료 코드

OP24 는 `verdict` 를 라이브 대조한다. `verify` 가 기대를
충족하지 못하면 exit 3 을 내는 것이 다른 pack (LR02) 의
관측이다. OP24 에 `expect_exits: [0, 3]` 을 넣을지, 17쪽
이상인 표본이라 0 만 허용할지는 왕복 때 정한다.
issue2007 중첩 표는 LR02 가 17쪽 이상으로 쓰는 표본이라
0 이 나올 가능성이 높다.

### 7.3 capabilities 경로

OP21–OP23 은 `capabilities` 에 `--json` 을 붙이지 않는다.
SD01/SD02 와 같다. 기본 출력이 JSON 이라는 기존 계약에
올라탄다. 기본 출력이 텍스트로 바뀌면 이 세 과제와
self-description pack 이 같이 깨진다. 그때는 `--json` 을
양쪽 pack 에 동시에 넣으면 된다.

### 7.4 dump-pages 와 info 의 쪽수 불일치

OP05 (info) 와 OP07 (dump-pages) 는 같은 표본의 쪽수를
다른 명령으로 묻는다. 둘이 어긋나면 오라클 내부 정합
이슈다. 과제는 둘을 따로 채점하므로 한쪽만 실패하는
것으로 드러난다. 자동으로 둘을 비교하는 검사는 넣지
않았다 — 그건 제품 회귀이지 gym 과제의 일이 아니다.

### 7.5 프로브가 시계/시드를 주입하지 않는다

비결정 오라클을 "같은 시드로 두 번" 돌리는 기능은 없다.
gym 오라클은 시드가 필요 없는 조회여야 한다는 입장이다.
시계가 들어가는 명령을 오라클로 등록하지 마라.

### 7.6 schemaVersion 진화

1.0 은 필수 키 네 개(kind, schemaVersion, ok, mode)다.
프로브 결과 요약 필드를 늘려도 1.0 안에서 가능하다.
필수 키를 바꾸면 1.1 과 CI 의 JSON 단언을 같이 올린다.

### 7.7 다국어 오류 메시지

오류 문자열은 한국어다. 테스트가 그 조각에 의존한다.
영문 로케일 변환은 계획에 없다. 바꿀 일이 있으면 시험과
README §23 을 한 커밋에서 고친다.

### 7.8 PARK.md / INVITE.md / profile

테마파크 지도에 oracle-probe 존을 그리지 않았다.
`--profile` 에도 넣지 않았다. 원하면 후속. 넣지 않는 편이
"채점 계약을 드러내는 확장" 이라는 자리와 맞다.

### 7.9 프로브 결과를 scorecard 에 붙이기

`score.py` 가 채점 앞에 `oracle_probe.structural_self_check()` 를
돌려 `ok=false` 면 채점을 거절하는 훅은 매력적이다.
이번 범위에 넣지 않았다. 채점 경로를 바꾸면 회귀 면적이
커진다. 지금은 CI 가 프로브를 따로 돌리는 것으로 충분하다.

### 7.10 Windows mkdir 의 정확한 토큰 분기

`{sub:file.hwp}` (중첩 폴더 없음) 에서
`os.makedirs(os.path.dirname(path), exist_ok=True)` 는
`dirname` 이 `sub_dir` 이라 안전하다. `sub_dir` 이 `""` 이면
`makedirs("")` 가 OSError 가 날 수 있다. 프로브는 그 예외를
잡아 실패 보고로 바꾼다. 호출자는 빈 `sub_dir` 에 정확한
토큰을 넣지 않는 것이 좋다. 단위 시험은 TemporaryDirectory 를
쓴다.

### 7.11 대형 페이로드 한계

단위 시험은 400 키 + 80 중첩을 두 번 정규화한다. 메가바이트
급 봉투는 시험하지 않았다. 오라클이 그런 봉투를 내면
`json_canonicalize` 가 메모리를 쓴다. gym 조회 명령의
`--json` 은 그 크기가 아니다.

### 7.12 lone surrogate

`json.dumps` 가 고립 서로게이트를 거부할 수 있다. 시험은
예외를 허용한다. 문서 오라클이 고립 서로게이트를 내는 일은
관측된 바 없다.

------------------------------------------------------------------------

## 8. 결정 로그

| 날짜 | 결정 | 이유 |
|------|------|------|
| 2026-08 | 프로브는 순수 Python | CI 가 바이너리 없이 전제를 감사 |
| 2026-08 | build_baseline 미import | 부수효과 차단, 대조 시험으로 동기화 |
| 2026-08 | 예외는 비통과 | 항상 죽는 오라클을 만점 처리하지 않음 |
| 2026-08 | 부재는 비통과 | 무제출 만점 방지 |
| 2026-08 | 새 CLI 없음 | PR 범위, clap 표면 고정 |
| 2026-08 | pack ID `oracle-probe`, 과제 `OP**` | 전역 충돌 없음 |
| 2026-08 | 12 pack 표 미수정 | 입문 코스 교체가 아님 |
| 2026-08 | 편집 과제 없음 | 축이 다름 |
| 2026-08 | PII 표본은 형식/쪽수만 | 보안 pack 비복제 |
| 2026-08 | 단위 시험을 예외 경로 중심으로 | 도구의 존재 이유가 실패 탐지 |

------------------------------------------------------------------------

## 9. 재현 체크리스트 (후속 작업자)

바이너리가 있는 트리에서:

```bash
python gym/tools/oracle_probe.py --json --selftest
python -m unittest scripts.tests.test_gym_oracle_probe
python gym/tools/audit.py
python gym/tools/build_baseline.py --agent op-ref --pack oracle-probe
python gym/score.py --agent op-ref --pack oracle-probe
```

기대:

- 프로브 `ok=true`, 부재 하위 프로브는 `ok=false`.
- unittest 전부 통과.
- audit 19 pack (또는 그 이상) 위반 0.
- 기준 풀이가 OP01–OP44 의 `answer.json` 을 만든다.
- 채점이 그 제출을 만점으로 읽는다.

실패하면 이 노트의 §7 해당 항을 먼저 보라.

------------------------------------------------------------------------

## 10. 관련 이슈·PR

- #5207 — 라이브 오라클 이중 계산 프로브 요청.
- PR #5214 — `feat/gym-oracle-probe`.
- #4664 — 다중 `{sub:}` 치환. 프로브가 그 규칙을 복제.
- #4653 — pack 스키마, 기준 풀이 왕복, 검사 연산자 등록부.
- #4600 — 편집 과제에 전역 훑기 금지. 이 pack 은 편집이 아님.

------------------------------------------------------------------------

## 11. 한 페이지 회고

골든 파일은 편하다. 그리고 거짓말한다.

gym 은 그 거짓말을 거절하고, 채점 순간에 rhwp 를 다시 돌린다.
그 거절이 진짜이려면 오라클이 결정적이고, 자리표가 깨끗하고,
없는 파일을 통과로 세지 않아야 한다.

`oracle_probe.py` 는 그 세 문장을 바이너리 없이 검사한다.
`oracle-probe` pack 은 그 세 문장을 실문서로 보여 준다.
단위 시험은 실패해야 할 것이 실패하는지 본다.

남은 일은 바이너리 왕복과, 원하면 score.py 앞단 훅이다.
이번 작업은 전제를 문서화하고, 예외를 시험하고, 과제로
구체화한 것으로 충분하다.

부재는 실패다. 그것이 이 노트의 마지막 문장이어야 했지만, 후속
작업자가 길을 잃지 않도록 부록을 붙인다.

------------------------------------------------------------------------

## 12. 부록 — 과제별 오라클 한 줄

이 표는 pack README 의 지도를 작업 노트에 복제한 것이 아니다.
**왕복 때 어디를 보면 되는가** 를 한 줄로 적는다.

```
OP01  info pageCount              table-001
OP02  explain paragraphCount      table-001
OP03  export-tables tableCount    table-001
OP04  search matchCount 표        table-001
OP05  info pageCount              issue2007 중첩
OP06  explain paragraphCount      issue2007 중첩
OP07  dump-pages pageCount        issue2007 중첩
OP08  export-tables tableCount    issue2007 중첩
OP09  fields fieldCount           field-01
OP10  len(fields)                 field-01
OP11  fields fieldCount           field-01-memo
OP12  fields fieldCount           form-01
OP13  fields fieldCount           form-02
OP14  len(search.matches) 국어    국립국어원
OP15  info pageCount              국립국어원
OP16  len(extract-data.items)     수출입 현황
OP17  info format                 table-001
OP18  info format                 hwpx_sample2
OP19  len(scan.files)             samples/hml
OP20  files[0].extMismatch        samples/hml --probe
OP21  len(capabilities.commands)  (도구)
OP22  len(formats.read)           (도구)
OP23  len(formats.write)          (도구)
OP24  verify verdict min-pages 17 issue2007 중첩
OP25  info pageCount              form-01
OP26  info pageCount              form-02
OP27  info pageCount              field-01
OP28  explain paragraphCount      field-01
OP29  explain paragraphCount      form-01
OP30  export-tables tableCount    multi-table-001
OP31  len(tables)                 multi-table-001
OP32  explain paragraphCount      para-001
OP33  search matchCount 마케팅    field-01
OP34  search matchCount 업무      국립국어원
OP35  info pageCount              exam-kor-1p
OP36  explain paragraphCount      exam-kor-1p
OP37  info format                 PII 분석 표본
OP38  info pageCount              PII 분석 표본
OP39  info format                 field-01
OP40  info + export-tables        table-001
OP41  fields + info               field-01
OP42  search + info               국립국어원
OP43  export-tables tableCount    form-01
OP44  explain paragraphCount      form-02
```

왕복이 한 과제에서 죽으면 이 표의 명령과 좌표를 먼저 확인하라.
좌표 이름이 바뀌었으면 과제·기준 풀이·이 표를 한 커밋에서 고친다.

------------------------------------------------------------------------

## 13. 부록 — 프로브 함수와 시험 클래스

| 함수 | 행복 시험 | 예외 시험 |
|------|-----------|-----------|
| leftover_sub_names | extract/dup names | 비문자열 → [] |
| extract_sub_names | 인벤토리 | (위와 동일) |
| is_exact_sub_token | `{sub:a}` | 임베디드·None |
| resolve_placeholders | 베이스라인 대조 | mkdir=False |
| resolve_cmd | argv 치환 | 비목록 TypeError |
| probe_placeholders | 다중 sub, {input} | 타입·KeyError·미닫힘 |
| probe_cmd_placeholders | 전 토큰 | 비순회·혼합 실패 |
| norm_scalar | int→float, strip | NaN/inf → None |
| json_ready | set 정렬, bytes | 커스텀 str() |
| json_canonicalize | 키 정렬, 한글 | NaN → null |
| snapshot | 변이 격리 | — |
| probe_determinism | 안정, 숫자 정규화 | 드리프트·예외·비호출 |
| probe_determinism_n | n=5 안정 | n<2, 드리프트, 예외 |
| classify_artifact | 행렬 | invalid 집합 닫힘 |
| probe_missing_artifact | present, 0바이트 | absent/dir/empty/None |
| probe_artifacts | 전부 present | 빈 목록, 하나 부재 |
| probe_live_oracle | 세 축 통과 | 축 논리곱 실패 |
| envelope | kind/version | kind 덮어쓰기 관측 |
| structural_self_check | exports 전부 | (모듈이 건강하면 ok) |
| run_selftest | failed=[] | 내장 실패 케이스 포함 |
| render_human | 통과 문구 | 실패 O/X |
| parse_args | 기본·플래그 | — |
| run / main | 종료 0, 봉투 | — |

이 표에 없는 공개 함수를 추가하면 시험이 `PUBLIC` 목록에서 실패한다.

------------------------------------------------------------------------

## 14. 마지막 문장

부재는 실패다.
