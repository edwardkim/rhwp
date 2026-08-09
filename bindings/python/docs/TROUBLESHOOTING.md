# 문제 해결 — 증상으로 찾는 원인과 처방

증상 문자열로 검색해서 쓰는 문서다. 각 항목은 **왜 그렇게 설계됐는지**까지 적는다 —
이유를 모르면 같은 문제를 다른 방식으로 다시 만든다.

---

## 설치·실행

### `BinaryNotFoundError: rhwp 실행 파일을 찾지 못했습니다`

바인딩은 rhwp 실행 파일을 세 곳에서 찾는다. 메시지에 시도한 위치가 전부 적혀 있다.

```
1. RHWP_BIN (미설정)
2. 패키지 동봉 (/.../rhwp/_bin/rhwp)
3. PATH (rhwp 없음)
```

**처방**: 셋 중 하나를 만족시킨다.

```bash
export RHWP_BIN=/path/to/rhwp          # 가장 확실
# 또는 PATH 에 두기
```

**왜 자동 설치가 없나**: 휠에 플랫폼별 바이너리를 동봉하는 것은 M18 이후 과제다.
지금은 rhwp 를 따로 설치하는 대신, 어느 바이너리가 실행되는지 항상 명확하다.

### `BinaryNotFoundError: RHWP_BIN 가 가리키는 실행 파일을 쓸 수 없습니다`

환경변수를 **줬는데** 그 경로가 없거나, 파일이 아니거나, 실행 권한이 없다.

**왜 조용히 넘어가지 않나**: 사용자는 그 바이너리를 쓰고 있다고 믿는데 다른 게
실행되면 "왜 내 수정이 반영 안 되지"라는 진단 불가 상황이 된다. 탐색 순서가
계약인 이유다.

```bash
ls -l "$RHWP_BIN"        # 존재·권한 확인
chmod +x "$RHWP_BIN"     # 유닉스
```

### `RhwpError: rhwp 실행에 실패했습니다: [Errno 8] Exec format error`

플랫폼이 맞지 않는 바이너리다(예: 리눅스 빌드를 macOS 에서).

---

## 종료 코드·예외

### `UsageError: 호출 인자가 올바르지 않습니다`

**exit 2 — 호출 조립이 틀렸다. 우리 쪽(또는 호출자) 버그다.** 재시도해도 같은
결과가 나오므로 인자를 고쳐야 한다.

도구가 교정 단서를 줬으면 꺼내 쓴다.

```python
try:
    ...
except rhwp.UsageError as exc:
    print(exc.suggestion)   # "가장 가까운 명령은 'export-svg' 입니다"
    print(exc.command)      # 재현 가능한 명령 문자열
```

흔한 원인:

- 없는 누름틀 이름 → `rhwp.fields(path)` 로 실존 이름 확인
- 범위 밖 표·셀 좌표 → `rhwp.export_tables(path)` 로 확인
- 범위 밖 쪽 번호 → `rhwp.info(path).page_count` 확인

### `RhwpRuntimeError: 문서 처리에 실패했습니다`

**exit 1 — 읽기·파싱·렌더·쓰기가 실패했다.** 인자를 고쳐도 안 풀리며 입력 자체를
봐야 한다.

```python
except rhwp.RhwpRuntimeError as exc:
    print(exc.stderr)    # 도구가 남긴 진단 원문
```

흔한 원인: 파일 없음 · 손상된 문서 · 암호 필요 · 디스크 쓰기 권한 없음.

### 검증이 실패했는데 예외가 안 난다

**의도된 동작이다.** `--verify` 불일치나 회귀 검출은 **도구가 정상 동작한 결과**다.
판정은 반환값으로 읽는다.

```python
result = rhwp.export_hwpx("a.hwp", out="b.hwpx", verify=True)
if not result.verify.identical:
    print(f"차이 {result.verify.diff_count}건")
```

예외를 원하면 명시한다.

```python
rhwp.export_hwpx("a.hwp", verify=True, raise_on_verdict=True)   # VerdictFailed
```

**왜 기본이 아닌가**: 예외로 올리면 호출자가 `try/except` 로 "고장"처럼 다루게 되고,
정작 봉투에 담긴 판정 근거를 읽지 않는다.

### `RhwpRuntimeError: 알 수 없는 종료 코드입니다 (N)`

rhwp 가 사전에 없는 코드를 냈다 — 본체와 바인딩 버전이 어긋났을 가능성이 높다.

**왜 조용히 통과시키지 않나**: 모르는 코드를 성공으로 취급하면 실패한 작업이
성공으로 보고된다.

```python
print(rhwp.capabilities()["exitCodes"])   # 이 rhwp 가 아는 코드
```

---

## 봉투·필드

### `AttributeError: 봉투에 'xxx' 필드가 없습니다`

오타이거나, 그 명령이 그 필드를 내지 않는다. 메시지에 **있는 필드가 함께** 나온다.

**왜 `None` 이 아닌가**: 없는 필드가 조용히 `None` 이 되면, 이름을 잘못 쓴 코드가
"값이 없네"로 흘러가 가장 찾기 어려운 버그가 된다.

```python
print(sorted(result.raw))                  # 실제 필드 목록
print(rhwp.capabilities().raw["commands"]) # 선언된 recordFields
```

### `result.verify` 가 `None` 인데 실패로 읽힌다

`None` 은 **"검증 안 함"**이지 "검증 실패"가 아니다.

```python
if result.verify is None:
    print("verify=True 를 주지 않았습니다")
elif result.verify.identical:
    print("통과")
else:
    print("실패")
```

### `changed_pages` 가 `None` 이다

**확정할 수 없다는 뜻**이다. `[]`(바뀐 쪽 없음)과 다르다.

변경 문단 중 하나라도 조판 커버리지 밖이면 rhwp 는 부분 목록 대신 `null` 을 낸다 —
빠뜨린 쪽이 있는 목록은 거짓 통과를 만들기 때문이다.

```python
pages = result.changed_pages
if pages is None:
    print("전체 확인이 필요합니다")
elif not pages:
    print("바뀐 쪽이 없습니다")
```

### `AttributeError: 봉투는 읽기 전용입니다`

도구가 내놓은 판정을 호출자가 고치려 했다. 값을 바꾸고 싶으면 `result.raw` 사본을
쓴다(원본 봉투는 그대로 남는다).

---

## 세션

### `SessionClosedError: 닫힌 문서 핸들입니다`

`with` 블록을 벗어난 뒤 `doc` 을 다시 썼다.

```python
with rhwp.open("a.hwp") as doc:
    doc.info()
doc.info()          # ← 여기서 실패
```

세션을 유지하고 싶으면 명시적으로 관리한다.

```python
session = rhwp.Session()
try:
    doc = rhwp.open("a.hwp", session=session)
    doc.close()                              # 문서만 닫힘, 세션은 유지
    other = rhwp.open("b.hwp", session=session)
finally:
    session.close()
```

### `ProtocolError: mcp-serve 가 응답 없이 종료했습니다`

서버가 죽었다. `exc.stderr` 에 사유가 있다.

```python
except rhwp.ProtocolError as exc:
    print(exc.stderr)
```

### 프로그램이 끝났는데 프로세스가 남는다

`with` 없이 세션을 열고 닫지 않았다. **서버가 남으면 파일을 잡고 있어 다음 작업이
막힌다.**

```python
with rhwp.open("a.hwp") as doc:   # 예외로 빠져나가도 정리된다
    ...
```

### `UsageError: hwp_doc_xxx 호출이 거부됐습니다`

도구가 `isError` 를 세웠다. 서버가 교정 단서를 실어 보내면 `exc.envelope` 에 있다.

```python
except rhwp.UsageError as exc:
    print(exc.envelope)   # {"error": "...", "nextCall": {"name": "hwp_open", ...}}
```

`nextCall` 은 **기계가 그대로 따라할 수 있는 교정 호출**이다.

---

## 계획

### `plan.check()` 가 통과했는데 `run()` 이 실패한다

원칙적으로 없어야 한다 — 검사와 실행이 **같은 판정자**를 쓰기 때문이다. 발생하면
버그이므로 이슈를 열어 달라.

단, 검사와 실행 사이에 **문서가 바뀌면** 결과가 달라질 수 있다.

### `ValueError: step 이 하나도 없는 계획은 실행할 수 없습니다`

빌더에 `fill_fields` 등을 하나도 부르지 않았다.

### `ValueError: 셀 값에 줄바꿈·탭은 넣을 수 없습니다`

셀은 한 줄 값이다. 여러 줄이 필요하면 세션 API 로 셀 안 문단을 직접 다뤄야 한다.

### 위반이 났는데 예외가 아니다

**의도된 동작이다.** 위반은 결과이고, 계획을 고쳐 다시 검사하는 것이 정상 흐름이다.

```python
result = plan.check()
if not result.ok:
    print(result.describe_violations())
```

### `plan.check()` 가 항상 실행 모드처럼 동작한다

rhwp 본체가 계획 `--dry-run` 을 아직 지원하지 않는 버전이다.

```python
caps = rhwp.capabilities()
run_cmd = next(c for c in caps.raw["commands"] if c["name"] == "run")
print("--dry-run" in (run_cmd.get("flags") or []))
```

---

## 인코딩

### 한글이 깨진다 / `UnicodeDecodeError`

바인딩은 stdout·stderr 를 **UTF-8** 로 읽는다. 실물 rhwp(Rust)는 콘솔 코드페이지와
무관하게 항상 UTF-8 을 내보내므로 정상 경로에서는 문제가 없다.

깨진다면 **rhwp 가 아닌 것**이 실행되고 있을 가능성이 높다.

```python
print(rhwp.find_binary())    # 무엇이 실행되는지 확인
```

**테스트에서 가짜 바이너리를 쓸 때**는 픽스처가 플랫폼 기본 인코딩을 따르므로
명시적으로 UTF-8 을 씌워야 한다(윈도우 cp949 에서만 깨져 오인하기 쉽다).

```python
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", newline="\n")
sys.stdin = io.TextIOWrapper(sys.stdin.buffer, encoding="utf-8")
```

---

## 성능

### 같은 문서에 여러 번 호출하니 느리다

1층은 호출마다 프로세스를 띄우고 문서를 다시 파싱한다. **2층 세션**을 쓴다.

```python
with rhwp.open("큰문서.hwp") as doc:   # 한 번만 파싱
    doc.info(); doc.fields(); doc.tables()
```

### `TimeoutError: 제한 시간 300초를 초과했습니다`

대형 문서 렌더·변환이 오래 걸린다.

```python
rhwp.export_pdf("큰문서.hwp", out="a.pdf", timeout=1800)
rhwp.export_pdf("큰문서.hwp", out="a.pdf", timeout=None)   # 무제한
```

`batch` 는 기본이 이미 무제한이다.

### 대량 처리에서 메모리가 늘어난다

`batch` 는 전 레코드를 모은다. 스트리밍이 필요하면 저수준 API 를 쓴다.

```python
import rhwp

for record in rhwp.iter_ndjson(["batch", "info", "--json"], stdin=paths_text):
    handle(record)     # 나오는 대로 처리
```

---

## 버전 불일치

### 새 rhwp 명령을 바인딩이 모른다

계약 패리티 가드가 CI 에서 잡지만, 로컬에서는 저수준 API 로 우회할 수 있다.

```python
envelope = rhwp.run_json(["새명령", "문서.hwp", "--json"])
```

그다음 `commands.py` 에 래퍼를 추가하는 PR 을 열어 달라.

### IR 모델이 스키마와 어긋난다

```bash
python tools/gen_models.py -o src/rhwp/ir.py --check    # 검사
python tools/gen_models.py -o src/rhwp/ir.py            # 재생성
```

---

## 그래도 안 되면

이슈를 열 때 아래를 함께 붙여 달라 — 재현이 절반이다.

```python
import rhwp
print("바인딩:", rhwp.__version__)
print("바이너리:", rhwp.find_binary())
print("rhwp:", rhwp.capabilities().version)
```

예외가 났다면 `exc.command` 가 그대로 재현 명령이다.
