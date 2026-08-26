---
kind: reference
status: active
canonical: mydocs/tech/agent_runtime/surface_spec.md
last_verified: 2026-08-03
---

# 진입로별 실패 사전 — 증상 문자열로 찾는다

> **v0.8.4 현행성 주의:** Python·Node 바인딩 오류 항목은 철회 전 기록이다.
> 두 공식 바인딩은 #4655에서 제거됐으며 현재 지원되는 진입로의 오류가 아니다.

받은 오류 메시지를 **그대로 검색**해 찾을 수 있게 만든 사전이다. 표제는 실제로
재현해서 받은 문자열이고, 재현하지 못한 항목은 근거(코드 경로)를 명시했다.

이 문서는 **진입로가 다르면 같은 사고가 다르게 보인다**는 문제를 다룬다.
진입로와 무관한 공통 실패(인코딩, 한글 파일명, 폰트 대체, 반복 필드 순번 등)는
[에이전트 실패 사전](../../manual/agent_troubleshooting_guide.md)에 있다 —
**먼저 그쪽을 보고 없으면 여기다.** 중복하지 않는다.

- 어느 진입로를 골라야 하나: [entrypoint_decision.md](entrypoint_decision.md)
- 비용 숫자: [cost_model.md](cost_model.md)
- 봉투 동등성 계약: [envelope_parity.md](envelope_parity.md) · 축 지도: [README.md](README.md)
- 명령·플래그 권위: [CLI 명령어 매뉴얼](../../manual/cli_commands.md)
- 로드맵: [#3869](https://github.com/edwardkim/rhwp/issues/3869)

재현 환경: Windows 11, `target/release/rhwp.exe` v0.8.2 (2026-08-03 빌드),
저장소 `samples/`. 아래 모든 문자열은 이 바이너리를 직접 돌려 받은 것이다.

---

## 0. 먼저 — 어느 층에서 깨졌나

진입로마다 실패를 신고하는 채널이 다르다. **채널을 착각하면 사전을 뒤져도 안 나온다.**

| 진입로 | 실패 신호가 있는 곳 | 판정(오류 아님)이 있는 곳 |
|---|---|---|
| CLI `--json` | **종료 코드** + stderr | 종료 코드 3/4 + 봉투 필드 |
| CLI `batch` | 최종 종료 코드 + **레코드의 `error`/`exitClass`** | 레코드 필드 |
| MCP 프로토콜 | `error{code,message}` (JSON-RPC) | — |
| MCP 도구 | `result.isError: true` | `isError:false` + **봉투 필드** |
| 세션 | `isError:true` + `nextCall` 힌트 | 봉투 필드 |
| 계획 실행기 | 종료 코드 + **`invalid[]`** (선검증) | `verify`·`steps[]` 저널 |
| 바인딩 | **예외 클래스** | 반환값의 판정 필드 |

대원칙은 [#2707 종료 코드 계약](../../manual/cli_commands.md#종료-코드-2707)이다:
**2 = 호출을 조립한 쪽의 버그, 1 = 환경·입력의 문제, 3/4 = 오류가 아니라 판정.**

---

## 1. CLI — 종료 코드가 뜻하는 것

`rhwp capabilities` 의 `exitCodes` 가 단일 출처다(실측 원문):

```json
{"0":"성공",
 "1":"런타임 실패 (읽기·파싱·렌더·쓰기)",
 "2":"사용법 오류 (인자 없음, 알 수 없는 옵션/명령, 페이지 범위 초과)",
 "3":"검증 단언 실패 — convert/export-hwpx --verify IR 차이, edit 3종 --verify 저장본 불일치, run 계획 assertions 미충족, render-diff --json 시각 회귀 검출(사람 모드는 종전대로 1)",
 "4":"--verify-pages 페이지 수 불일치 (convert/export-hwpx)"}
```

### stdout 0바이트 규약과 그 예외

선언(`capabilities.jsonContract.failure`)은 이렇다:

> 단건 명령 실패 시 stdout 0바이트; batch 는 error 레코드 + 최종 exit 1

실측으로 확인한 값:

| 명령 | 상황 | exit | stdout | stderr |
|---|---|---:|---:|---|
| `info --json` | 없는 파일 | 1 | **0 B** | 128 B |
| `run` | 계획 파일이 없음 | 1 | **0 B** | 메시지 |
| `run` | 계획 JSON 문법 오류 | 2 | **0 B** | 메시지 |
| `run` | **계획 안의** 문서를 못 읽음 | 1 | **200 B(봉투)** | 0 B |
| `run` | 계획 필드 누락·선검증 위반 | 2 | **봉투** | 0 B |

**`run` 은 규약의 예외다.** `run_plan_engine`(`src/main.rs:13341`)이 CLI 와 MCP
`hwp_run_plan` 에 **같은 저널**을 주도록 설계돼 있어, 실패도 봉투로 표현한다.
계획 파일 자체를 읽거나 파싱하는 단계는 엔진 진입 **전**이라 종전 규약을 따른다.

> 파서를 쓸 때: `run` 은 exit 0/1/2/3 모두에서 stdout 을 파싱해 보고, 비어 있으면
> 계획 파일 단계의 실패로 분류한다. 다른 명령은 exit != 0 이면 stdout 을 읽지 않는다.

---

## 2. CLI 증상 사전

### "오류: 파일을 읽을 수 없습니다 - <경로>: 지정된 파일을 찾을 수 없습니다. (os error 2)" (exit 1)

- **원인**: 경로가 틀렸다. 상대 경로면 rhwp 의 **현재 작업 디렉터리** 기준이다.
- **처방**: 절대 경로를 쓴다. MCP·바인딩을 통해 부를 때는 서버·인터프리터의 cwd 가
  에이전트의 cwd 와 다를 수 있다 — 이 증상의 절반은 그것이다.

### "오류: 문서 파싱 실패 - 유효하지 않은 파일: 지원하지 않는 포맷입니다: 알 수 없는 파일 형식. 오류코드: UNSUPPORTED_FILE_FORMAT." (exit 1)

- **재현**: `rhwp info --json README.md`
- **원인**: HWP5·HWPX·일부 HWP3·HWPML 2.9 가 아니다. 확장자가 `.hwp` 라도 내용이
  다르면(HTML·PDF·ZIP) 여기로 온다.
- **처방**: 파일 앞머리를 확인한다. 아카이브 스윕이면 `batch` 의 `error` 레코드로
  격리되므로 전체를 버리지 말고 그 건만 분류한다.

### "오류: 알 수 없는 명령입니다 - <이름>" (exit 2)

- **원인**: 명령 이름 환각. stderr 에 전체 사용법이 이어진다.
- **처방**: `rhwp capabilities` 의 `commands[].name` 이 유일한 목록이다(실측 61개,
  그중 `--json` 지원 31개). 도구 정의를 여기서 생성하면 이 증상이 사라진다.

### "알 수 없는 옵션: -o" (exit 2)

- **재현**: `rhwp info --json -o out.json 문서.hwp`
- **원인**: **옵션 표면은 명령별 계약이다.** `info` 에는 `-o` 가 없다.
  (`export-hwpx` 는 출력을 positional 로 받는 것도 같은 이유다.)
- **처방**: `capabilities` 의 해당 명령 `flags` 를 읽는다. 온보딩 시 한 번 캐시한다.

### "오류: 페이지 번호가 범위를 벗어났습니다 (0~N)" (exit 2)

- **원인**: 페이지는 **0 기준**이다. 1쪽 문서에 `-p 1` 을 주면 `(0~0)` 이 나온다.
- **처방**: `search --json` 의 `matches[].page` 는 이미 0 기준이므로 그대로 `-p` 에
  넣는다. 사람에게 보여줄 때만 +1 한다.

### "오류: export-png 명령은 native-skia feature 가 활성화되어야 합니다." (exit 2)

- **원인**: 그 feature 없이 빌드된 바이너리다.
- **처방**: 호출 **전에** `capabilities` 의 해당 명령 `available` 을 본다.
  `false` 면 `export-svg` 로 대체한다. 이 저장소의 기본 릴리스 빌드가 그렇다.

### "오류: 비밀번호가 필요한 암호 문서입니다 (--password <pw> 로 전달)." (exit 2)

- vs **"오류: 비밀번호가 일치하지 않거나 암호화 데이터가 손상되었습니다." (exit 1)**
- **구분이 계약이다**: 비밀번호를 **안 준 것**은 조립 버그(2), **틀린 것**은
  런타임 실패(1)다. 재시도 정책이 갈린다 — 2 는 재시도해도 같고, 1 은 다른
  비밀번호로 재시도할 여지가 있다.
- **처방**: 자동화는 `--password-stdin` 을 쓴다. `--password` 값은 프로세스 목록에
  노출된다.

### "오류: 알 수 없는 프로필 '<이름>'" (exit 2)

- **재현**: `rhwp capabilities --mcp --profile 없는프로필`
- stderr 둘째 줄이 목록을 준다: `사용 가능: 경영보고, 행정서식, 데이터분석,
  콘텐츠제작, 아카이브검색, 품질검증, 개발통합`
- **처방**: 프로필 이름은 **한글**이다. 영문으로 추측하면 반드시 실패한다.

### "오류: 마스킹은 되돌릴 수 없습니다. 산출 경로를 -o <출력> 으로 지정하거나, 원본을 덮어쓸 의도라면 --in-place 를 명시하세요" (exit 2)

- **원인 아님(보호 동작)**: `edit redact` 는 출력 지정 없이는 실행을 거부한다.
- **처방**: `--dry-run` 으로 먼저 확인한다. **`--dry-run` 출력에는 원문 개인정보가
  그대로 들어간다 — 로그·컨텍스트에 남기지 않는다.**

### "오류: batch 는 export-text·info·export-structure·export-tables·fields·search·convert·fill 만 지원합니다 - <이름>" (exit 2)

- **처방**: `capabilities` 의 `batch.subcommands` 가 목록이다. 단건 명령이 전부
  batch 축을 갖지는 않는다.

### "오류: batch search 는 --query <검색어> 가 필요합니다." / "오류: batch fill 은 --form <서식> --data <행 파일> --out-dir <폴더> 가 모두 필요합니다." (exit 2)

- **원인**: batch 축마다 필수 인자가 다르다. 특히 **`batch fill` 만 stdin 을 읽지
  않는다** — 입력이 경로 목록이 아니라 `--data` 파일의 '행'이기 때문이다.
- **처방**: `fill` 에 파일 목록을 파이프로 넣고 있으면 축을 잘못 고른 것이다.

### "오류: --data 파일을 읽을 수 없습니다 - <경로>: ..." (exit 1)

- **원인**: `--data @파일` 의 파일이 없다. **exit 2 가 아니라 1 이다** — 인자 모양은
  맞고 가리키는 파일이 없는 것이므로.
- 관련: 파일이 있는데 인코딩이 cp949 면 다른 증상
  (`stream did not contain valid UTF-8`)이 나온다 —
  [공통 사전](../../manual/agent_troubleshooting_guide.md)의 입력·인코딩 절.

### `--json` 을 줬는데 사람용 텍스트가 나오고 exit 0 (조용한 무시)

- **재현**: `rhwp dump 문서.hwp --json` → **exit 0, stdout 30,060 B 의 사람용 덤프**
- **원인**: `dump`·`diag`·`bench` 는 내부 개발 도구라 `capabilities` 에 `flags` 도
  `json` 도 선언돼 있지 않다(실측: `dump json= None flags= None`). 선언에 없는
  명령은 `--json` 을 해석하지 않는다.
- **처방**: 호출 전에 `capabilities` 에서 그 명령의 `json:true` 를 확인한다.
  **exit 0 인데 JSON 이 아니면 파서가 아니라 명령 선택이 틀린 것이다.**

### stderr 에 `LAYOUT_OVERFLOW: page=… overflow=…px` 가 쏟아짐

- **원인 아님(설계)**: 렌더 진단이다. stdout 은 여전히 순수 JSON/NDJSON 이다.
- **처방**: `2>diag.log` 로 분리해 보존한다. 이것을 실패로 오독해 배치를 중단시키면
  성공한 산출물을 버리게 된다.

---

## 3. CLI — 판정(exit 3/4)과 "성공한 실패"

### "검증 실패(--verify): <산출물> 재파싱 후 IR 차이 N건" (exit 3)

- **재현**: `rhwp export-hwpx samples/k-water-rfp-2024.hwp out.hwpx --verify --verify-pages`
  → `exit=3`, `… 재파싱 후 IR 차이 3건`
- **원인 아님**: 변환 산출물은 **이미 저장됐다.** 3 은 "재파싱한 IR 이 원본과
  다르다"는 판정이다.
- **처방**: `ir-diff <원본> <산출물> --json` 으로 `categories` 를 본다. 배치 게이트는
  exit 0 과 3 을 분기해 3 을 "불합격"이 아니라 "검토 큐"로 보낸다.

### "[hwpx] Picture 직렬화 실패: XML 쓰기 실패: <hp:pic> binaryItemIDRef 미등록 bin_data_id=… (BinDataContent 누락)" (exit 3)

- **재현**: `rhwp export-hwpx samples/hwpspec.hwp out.hwpx --verify`
- **구분**: 위 항목과 같은 exit 3 이지만 성격이 다르다 — **직렬화 자체가 일부
  실패**했다. 문서별 잔여 결함이므로 자동 재시도로 풀리지 않는다.
- **처방**: 증상 문자열 그대로 이슈에 넣는다. 파이프라인은 이 문서를 건너뛴다.

### exit 4 (`--verify-pages` 페이지 수 불일치)

- **이 조사에서 재현하지 못했다.** 시도한 10개 문서 전부 exit 0/3 이었다.
- **근거**: `capabilities.exitCodes["4"]`, `src/main.rs:2239`, `src/main.rs:6655`
  (쪽수 불일치면 IR 비교를 하지 않고 `verify: null` 로 단락).
- **처방**: 게이트에서 4 를 3 과 별도로 분기한다 — 쪽수가 달라졌다면 IR 비교
  결과가 없다는 뜻이므로 "차이 0건"으로 오독하면 안 된다.

### exit 0 인데 아무것도 안 바뀜 — **가장 위험한 증상**

같은 논리 오류가 진입로마다 다르게 나온다. 실측 대조표:

| 상황 | `edit` 단건 | `run` 계획 |
|---|---|---|
| 없는 표 번호 (`--table 9`) | **exit 1** (stdout 0 B) | **exit 2** + `invalid[]` |
| 병합으로 덮인 칸 | exit 2 (stdout 0 B) | exit 2 + `invalid[]` |
| 치환 0건 | **exit 0**, `replacedCount:0`, **출력 파일 미생성** | exit 2 + `invalid[]` |
| 없는 필드만 지정 | **exit 0**, `notFound:["없는필드"]`, `filledCount:0`, **출력 파일 생성됨** | exit 2 + `invalid[]` |

**종료 코드만 보는 게이트는 마지막 줄에서 조용히 통과한다.** `edit fill-fields` 는
아무것도 못 채웠는데 출력 파일을 만들고 exit 0 을 낸다.

- **처방(단건)**: `notFound == [] && ambiguous == [] && filledCount == 기대값` 을
  게이트로 건다. `replace-text` 는 `replacedCount > 0` 을 본다.
- **처방(구조적)**: 다단계 편집은 `run` 으로 옮긴다. 선검증이 같은 사고를
  **실행 전에** exit 2 로 만든다.

---

## 4. MCP — 세 층을 혼동하지 않기

같은 서버가 세 가지 방식으로 실패를 말한다. **층을 착각하면 오독한다.**

### 층 ① JSON-RPC 오류 (`error{code,message}`) — 프로토콜이 깨졌다

| 재현 | 응답 |
|---|---|
| JSON 이 아닌 줄을 보냄 | `{"error":{"code":-32700,"message":"JSON 파싱 실패: expected value at line 1 column 1"},"id":null,"jsonrpc":"2.0"}` |
| `{"method":"prompts/list"}` | `{"error":{"code":-32601,"message":"지원하지 않는 메서드: prompts/list"}}` |
| `tools/call` 에 `params.name` 없음 | `{"error":{"code":-32602,"message":"params.name 이 필요합니다"}}` |
| 없는 리소스 읽기 | `{"error":{"code":-32002,"data":{"uri":"rhwp://docs/nope.md"},"message":"알 수 없는 리소스: rhwp://docs/nope.md"}}` |

- **처방**: 이 층은 **클라이언트 구현 버그**다. 도구 인자를 고쳐도 안 풀린다.
- 지원 메서드는 `initialize`·`notifications/initialized`·`ping`·`tools/list`·
  `tools/call`·`resources/list`·`resources/read` 다. 나머지는 전부 -32601 이다
  (`prompts/list`·`completion/complete`·`logging/setLevel`·`sampling/createMessage`
  로 확인).

### 층 ② 도구 실행 실패 (`result.isError: true`) — 호출은 도달했고 일이 안 됐다

| 재현 | `content[0].text` |
|---|---|
| 없는 도구 이름 | `{"didYouMean":[],"error":"알 수 없는 도구: hwp_nope"}` |
| 필수 인자 누락 | `필수 인자 누락: path` |
| 없는 파일 | `종료 코드 1: 오류: 파일을 읽을 수 없습니다 - …: (os error 2)` |
| 모르는/닫힌 `docId` | `{"error":"열려 있지 않은 핸들: doc-99 (hwp_open 먼저)","nextCall":{"arguments":{"path":"<열 문서 경로>"},"name":"hwp_open","why":"핸들이 없거나 만료 — hwp_open 으로 docId 를 재발급한 뒤 재시도"}}` |

- **`종료 코드 N:` 접두사가 붙은 텍스트는 CLI 의 stderr 다.** 무상태 도구는 rhwp 를
  자식 프로세스로 돌리고 stdout 이 비면 `isError:true` 로 올린다. 그래서 §2 의
  CLI 사전이 그대로 적용된다 — 접두사를 떼고 문자열로 검색하라.
- **`nextCall` 이 있으면 그대로 따르면 된다.** 세션 실패는 다음 호출을 지정해준다.

### 층 ③ 성공적으로 전달된 "부정적 결과" — 오류가 아니다

`hwp_ir_diff` 로 서로 다른 두 문서를 비교하면:

```
isError = False,  structuredContent 있음
content[0].text = {"a":…,"b":…,"categories":{…},"diffCount":234,"identical":false,…}
```

CLI 였다면 exit 3 이었을 판정이 **`isError:false` 로 온다.**

- **처방**: `isError` 만 보고 "검증 통과"로 읽지 말 것. 봉투의 `identical`·
  `diffCount`·`notFound`·`replacedCount`·`invalid` 를 읽는다.
- 대응 규칙: exit 0 → `isError:false` / exit 1 → stdout 이 비면 `isError:true` /
  exit 2 → `isError:true` / exit 3 → `isError:false` + 판정 필드.
  **배치의 부분 실패(exit 1)는 stdout 에 NDJSON 이 있으므로 결과로 전달된다.**

### MCP 특유의 증상

**증상: 응답이 예상의 2배 크기다.**
원인 아님(설계). `tools/call` 은 봉투를 `content[0].text` 와 `structuredContent`
양쪽에 싣는다. 실측 배율 2.03~2.46배([cost_model.md](cost_model.md) §6).
처방: 둘 중 하나만 소비하고, 큰 봉투는 애초에 좁혀 부른다(`-p`·`--max-matches`).

**증상: 도구 목록만으로 컨텍스트가 40 KB 찬다.**
실측 `tools/list` 응답 40,503 B(도구 51개). 처방: `mcp-serve --profile <직무>`.

**증상: 서버가 갑자기 끝난다.**
`mcp-serve` 는 **stdin EOF 에서 종료**한다. 클라이언트가 파이프를 닫으면 그것이
정상 종료다. 세션 핸들은 프로세스와 함께 사라진다 — 영속되지 않는다.

---

## 5. 철회된 바인딩 오류 이력

이 절은 #4655 이전 Python·Node 바인딩의 오류 계약을 보존한 historical record다.
현재 지원 진입로의 트러블슈팅으로 사용하지 않는다.

### `BinaryNotFoundError: RHWP_BIN 가 가리키는 실행 파일을 쓸 수 없습니다: <경로>`

```
RHWP_BIN 가 가리키는 실행 파일을 쓸 수 없습니다: C:/없는경로/rhwp.exe
  (존재하지 않거나, 파일이 아니거나, 실행 권한이 없습니다)
```

- **원인 아님(설계)**: 환경변수를 **줬는데 못 쓰면 다음 후보로 넘어가지 않는다.**
  사용자가 그 바이너리를 쓴다고 믿는데 다른 게 실행되면 디버깅이 불가능해서다.
- **처방**: 경로를 고치거나 변수를 지운다. 윈도우에서는 확장자(`.exe`/`.bat`/`.cmd`)로
  실행 가능 여부를 판단하므로 확장자 없는 경로는 실패한다.

### `BinaryNotFoundError: rhwp 실행 파일을 찾지 못했습니다. 다음 순서로 탐색했습니다:`

```
  1. RHWP_BIN (미설정)
  2. 패키지 동봉 (…/src/rhwp/_bin/rhwp.exe)
  3. PATH (rhwp.exe 없음)

해결: rhwp 를 설치해 PATH 에 두거나, RHWP_BIN 로 경로를 지정하세요.
```

- **탐색 순서가 계약이다**: `RHWP_BIN` → 패키지 동봉 → `PATH`. 뒤집으면 로컬 빌드를
  가리켜도 동봉본이 실행돼 "왜 수정이 반영 안 되지"가 된다.
- Node 바인딩도 같은 변수 이름(`RHWP_BIN`)과 같은 순서를 쓴다.

### `RhwpRuntimeError: 문서 처리에 실패했습니다 (exit 1) — <CLI stderr>`

- exit 1 의 매핑. 예외의 `.command` 속성에 **재현 가능한 명령 문자열**이 그대로 들어
  있다(따옴표까지 붙여서). 버그 리포트에 그대로 붙인다.
- 예: `… edit set-cell samples/복학원서.hwp --table 9 --row 0 --col 0 --text z -o … --json`

### `UsageError` (exit 2)

- **호출자 버그**다. 재시도해도 같다. 도구가 `힌트:` 줄을 남겼으면
  `.suggestion` 으로 꺼낸다.

### exit 3/4 는 예외가 아니다

- `--verify` 불일치·`render-diff` 회귀는 **반환값의 판정 필드**로 온다.
  `result.verify.identical` / `result.verify.diff_count` 를 읽어야 한다.
- 예외로 받고 싶으면 `raise_on_verdict=True`(Python) / `throwOnVerdict: true`(TS)로
  **명시**한다. 기본값을 뒤집지 않는다.

### `AttributeError: "봉투에 'page_conut' 필드가 없습니다. 있는 필드: fonts, format, pageCount, …"`

- **원인 아님(설계)**: 없는 필드가 `None` 이 되면 오타 코드가 "값이 없네"로 흘러가
  가장 찾기 어려운 버그가 된다. `Envelope` 는 없는 키에 실패한다.
- **처방**: 메시지가 있는 필드를 전부 나열해 준다. 그중에서 고른다.
  속성(snake_case)·원문 키(camelCase)·변환 키 세 방식이 같은 값을 가리킨다.

### `SessionClosedError: 닫힌 문서 핸들입니다 (doc-1)`

- `with` 블록을 빠져나온 뒤 `doc.text()` 를 불렀다. 세션·문서 모두 컨텍스트
  매니저이고 `close()` 는 멱등이다.
- **자원 정리 주의**: 세션은 자식 프로세스를 띄운다. 남으면 다음 작업이 파일을
  못 연다. `with` 를 쓰면 예외로 빠져나가도 닫힌다.

### `RhwpRuntimeError: 알 수 없는 종료 코드입니다 (N) — rhwp 와 바인딩 버전이 어긋났을 수 있습니다`

- **버전 불일치의 정식 증상**이다(근거: `bindings/python/src/rhwp/errors.py:214`).
  rhwp 에 새 종료 코드가 생겼는데 바인딩이 모르면, **조용히 통과시키지 않고**
  여기서 멈춘다.
- **처방**: `rhwp --version` 과 바인딩 버전을 맞춘다. `ProtocolError`(stdout 이
  JSON 이 아님)도 같은 원인일 수 있다 — 그쪽은 "도구 버그이거나 버전 불일치".

---

## 6. 계획 실행기 `run` — 선검증 실패 vs 실행 중 실패

### 선검증 실패 — `invalid[]` + exit 2 (디스크 무변경)

전 step 을 먼저 검사하고 **위반을 전부 모아 한 번에** 보고한다. 하나 고치면 다음
위반이 나오는 두더지잡기를 막기 위한 설계다. 실측 원문:

| `invalid[].reason` | 원인 |
|---|---|
| `action 이 필요합니다` | step 의 키가 `action` 이다. **`op` 가 아니다** — 가장 흔한 조립 실수 |
| `오류: 본문 최상위 표 9 번이 없습니다 (최상위 표 3개; 중첩 표는 v1 범위 밖).` | 표 번호가 범위 밖이거나 중첩 표를 지목 |
| `오류: (0,2) 는 병합으로 덮인 칸입니다 — 앵커 (0,0) 를 지정하세요.` | 병합 아래 숨은 칸. 앵커 좌표가 안내된다 |
| `'<문자열>' 일치 0건 — 치환할 곳이 없습니다` | 문서 표기와 다름(전각/반각·공백·줄바꿈). `search` 로 실제 표기를 먼저 본다 |
| `필드 '<이름>' 이(가) 없거나 순번이 범위 밖입니다 (동명 0개)` | 누름틀 이름 오류. `fields --json` 으로 목록을 먼저 받는다 |

계획 자체의 모양 오류(전부 exit 2 + 봉투의 `error`):

| 문자열 | 원인 |
|---|---|
| `planVersion "1.0" 이 필요합니다` | `planVersion` 누락 또는 다른 값 |
| `input (원본 문서 경로)이 필요합니다` | `input` 누락 |
| `output (산출 경로)이 필요합니다` | `output` 누락 |
| `steps 는 비어 있지 않은 배열이어야 합니다` | `steps` 누락·빈 배열 |

### 실행/입력 단계 실패

| 증상 | exit | stdout |
|---|---:|---|
| `오류: 계획 파일을 읽을 수 없습니다 - <경로>: …` | 1 | 0 B (stderr) |
| `오류: 계획 JSON 파싱 실패 - key must be a string at line 1 column 3` | 2 | 0 B (stderr) |
| `{"error":"입력을 읽을 수 없습니다 - <문서>: …"}` | 1 | **봉투 200 B** |
| `{"error":"HWP 파싱 실패 - …"}` | 1 | 봉투 (근거: `src/main.rs:13385`) |

### 단언 실패 — exit 3 (판정)

`assertions.verify` 를 켰는데 저장본 재파싱이 원본과 다르면 exit 3 이다.
**오류가 아니라 판정**이므로 저널(`steps[]`·`verify`)을 읽고 판단한다.
`--dry-run` 은 `preview[]` 에 각 step 의 `currentText`→`newText` 를 준다(480 B).

---

## 7. 진입로를 바꾸면 사라지는 실패

| 증상 | 진입로를 이것으로 |
|---|---|
| 다단계 편집 중간에 실패해 산출물이 반쯤 고쳐짐 | `run` — 실패 시 디스크 무변경 |
| `edit` 가 exit 0 인데 아무것도 안 바뀜 | `run` — 같은 상황이 선검증 exit 2 |
| 대형 문서 조회를 반복하다 느려짐 | MCP 세션 — 재파싱 0회 |
| 파일 N개 루프가 오래 걸림 | `batch` — 기동 1회 |
| 봉투 키 오타가 조용히 `None` 이 됨 | 바인딩 — `AttributeError` |
| 도구 목록만 40 KB | `--profile` — 6~10 KB |
| 실행 파일을 놓을 수 없는 환경 | **현재 없음** — [#3869](https://github.com/edwardkim/rhwp/issues/3869) |

---

## 8. 그래도 안 풀리면

1. **층을 먼저 정한다**(§0). 프로토콜·도구·봉투 중 어디서 깨졌는지.
2. `rhwp capabilities` 로 명령 표면·`flags`·`json`·`available` 을 재확인한다(추측 금지).
3. 같은 입력으로 **CLI 사람용 모드**(`--json` 없이)를 돌려 stderr 안내를 읽는다.
   MCP·바인딩 경유 실패의 대부분은 CLI 에서 그대로 재현된다.
4. 바인딩이면 예외의 `.command` 를, MCP 면 도구 선언의 `cli.args` 를 꺼내
   같은 명령을 셸에서 직접 돌린다.
5. `info` → `dump`/`diag` 순으로 입력 문서 자체의 이상을 좁힌다
   ([문서 진단 도구](../../manual/document_diagnostics_tool_manual.md)).
6. 재현 명령·stderr·종료 코드로 이슈를 연다. **증상 문자열을 제목에 그대로 넣으면**
   다음 사람이 이 사전에서 찾는다.

---

## 인접 문서

- [entrypoint_decision.md](entrypoint_decision.md) — 어느 진입로를 고를 것인가
- [cost_model.md](cost_model.md) — 각 경로의 실측 비용
- [surface_spec.md](surface_spec.md) — WASM 표면 설계(canonical)
- [envelope_parity.md](envelope_parity.md) — 봉투 동등성 계약
- [README.md](README.md) — 축 지도
- [agent_troubleshooting_guide.md](../../manual/agent_troubleshooting_guide.md) — **진입로 무관 공통 실패**
- [cli_commands.md](../../manual/cli_commands.md) — 명령·플래그·종료 코드 권위
- [mcp_integration_guide.md](../../manual/mcp_integration_guide.md) — MCP 오류 의미론
- 공식 Python·Node 바인딩 철회: [#4655](https://github.com/edwardkim/rhwp/issues/4655)
- [agent_boundary_contract.md](../agent_boundary_contract.md) — 핸들·경로·자원 한계의 경계 계약
- 이슈 [#3869](https://github.com/edwardkim/rhwp/issues/3869)
