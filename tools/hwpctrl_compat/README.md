# 웹한글컨트롤 호환 차등 하니스 (P0)

계획서: [`mydocs/plans/hwpctrl_ocx_full_compat.md`](../../mydocs/plans/hwpctrl_ocx_full_compat.md)

**판정자는 문서가 아니라 설치된 한글이다.** 같은 시나리오를 한글 COM 과 rhwp 양쪽에 돌려
반환값·문서 상태를 대조한다.

```
scenarios/*.json  ── runner_ocx.py   (설치된 한글, 프로세스 격리) ─┐
                  └─ runner_rhwp.mjs (rhwp WASM)                  ├→ compare.py → verdict/
                                                                   ┘        ↓
                                                       build_ledger.py --ingest → api_ledger.json
```

## 한 번에 돌리기

모든 OS에서 아래 명령을 실행할 수 있다.

```bash
npm --prefix npm/hwpctrl-ocx run gate
```

`npm run gate`는 신규 패키지 엔트리(`npm/hwpctrl-ocx/src/index.mjs`)를 대상으로 한다. Windows에서는
한글 2022 COM Oracle을 새로 실행해 전체 대조한다. macOS·Linux에서는 COM을 실행하지 않고 같은
등록 시나리오의 호출 순서·API 오류·`SaveAs` 산출물을 WASM 구현에서 검사한다. 이 결과는 **WASM
자체 회귀 검증**이며 새 Hancom Oracle 통과 근거는 아니다.

한글 2022로 수집해 검토·고정한 fixture가 있는 경우에는 어느 OS에서나 읽기 전용 대조를 실행할 수
있다. fixture에는 `<scenario>.returns.json`과 저장 시나리오의 HWP 산출물이 함께 있어야 하며, 각
반환 JSON의 `oracle.version`은 major `12`여야 한다.

```bash
# 저장소에 고정한 fixture를 사용할 때
node tools/hwpctrl_compat/python_runner.mjs run_gate.py \
  --impl npm/hwpctrl-ocx/src/index.mjs --fixture

# 별도 보관 fixture를 사용할 때
node tools/hwpctrl_compat/python_runner.mjs run_gate.py \
  --impl npm/hwpctrl-ocx/src/index.mjs --oracle-dir /path/to/hancom2022-fixture
```

`--fixture`의 표준 위치는 `tools/hwpctrl_compat/fixtures/hancom2022/`다. fixture는 Windows의
정상 COM 재수집 결과를 검토한 뒤에만 갱신한다. macOS·Linux에서 fixture를 새로 만들거나
`oracle.version`을 임의로 고치면 안 된다. `--impl legacy`는 기존 studio 층을 대상으로 하는
**하니스 자체 검증용**이며, 패키지의 호환 통과 근거로 사용하지 않는다.

## Windows 실행 전 조건

- 한글 2022 COM major 버전 `12`와 `pywin32`가 필요하다.
  `python -c "import win32com"`이 실패하면 먼저 준비한다.
- Node.js, `wasm-pack`, `rhwp-studio/node_modules/`가 있어야 한다.
- 시작 전에 `Hwp.exe` 또는 `HwpFrame.exe`가 실행 중이면 게이트는 `OCCUPIED`로 중단한다.
  다른 사용자의 한글 프로세스를 종료하지 않는다.
- `GetTextFile("TEXT")`와 `SetTextFile(..., "TEXT")`는 Windows 시스템 ANSI code page의
  영향을 받는다. ACP 65001에서는 한컴 COM이 한글을 U+FFFD로 돌려주거나 삽입 위치를 CP949
  byte 수로 옮기는 것이 확인됐다. 따라서 본문 완전일치와 편집 시나리오는 공식
  `UNICODE` 형식을 사용하고, `TEXT`의 CP949 수치 참조 규칙은 Rust·npm 계약 테스트로 고정한다.
- macOS·Linux에서는 `npm run gate`가 WASM 자체 시나리오 검증을 수행한다. 한글 2022 fixture를
  명시한 읽기 전용 대조는 가능하지만, live COM Oracle 수집·fixture 갱신은 Windows에서만 한다.

`run_gate.py`의 `--cleanup-spawned`는 전용 Windows 계정에서 시간 초과 뒤 생긴 PID를 정리할 때만
명시적으로 사용한다. 일반 개발·검토 환경에서는 지정하지 않는다.

## 스펙 재생성

정답지는 한컴 공식 문서 3종이다. 추측으로 채우지 않는다.

```bash
node tools/hwpctrl_compat/python_runner.mjs extract_spec.py   # samples/hwpctl_*.hwp → npm/hwpctrl-ocx/spec/*.json
node tools/hwpctrl_compat/python_runner.mjs build_ledger.py   # 스펙 → 원장 골격(기존 상태 보존)
node tools/hwpctrl_compat/python_runner.mjs build_ledger.py --check   # CI 게이트: 스펙↔원장 불일치 시 exit 1
```

추출 규모: API 122(속성 18·메서드 67·이벤트 3·객체 34) · Action 312 · ParameterSet 50/Item 521
= **원장 484 항목**.

## 시나리오 쓰는 법

```json
{ "id": "field-read",
  "ledger": ["HwpCtrl.method.GetFieldText"],   // 이 시나리오가 검증하려는 원장 항목
  "open": "samples/....hwp",
  "calls": [["GetFieldText", ["pt_nm"]]],
  "saveAs": "field-read.hwp" }
```

규칙 여섯 가지.

1. **`ledger` 를 반드시 적는다.** 원장은 시나리오 단위로 올라간다 — 선언이 없으면 무엇을
   검증했는지 알 수 없고, 그 실행은 진척으로 세지 않는다.
2. **바꾼 뒤에는 반드시 읽는다.** 반환값만 보면 아무 일도 하지 않는 구현이 통과한다.
   실제로 `MovePos` 는 `true` 만 돌려주고 커서를 옮기지 않아도 반환값 대조를 통과했다.
   뒤따르는 `GetPos` 가 그것을 잡았다.
3. **객체가 돌아오면 점을 찍어 들어간다.** 서식(`CharShape`)·개체 속성은 ParameterSet
   **객체**로 오는데, 러너는 객체를 `{__type: …}` 으로 줄인다. `["CharShape.Item", ["Height"]]`
   처럼 적어야 값이 대조된다 — 안 그러면 빈 셋을 돌려주는 구현도 통과한다.
4. **경로를 박지 않는다.** `paths` 에 이름으로 적고 인자에는 `{"$path": "이름"}` 을 쓴다.
   Windows 절대 경로를 그대로 박으면 Linux 에서 그것은 "못 여는 경로"가 아니라 **그냥 그런
   이름의 상대 경로**라, 없는 폴더를 재려던 자리가 성공하고 작업본에 쓰레기가 남는다.
5. **일부러 죽는 호출은 미리 선언한다.** `expectError` 없이 죽으면 게이트는 붉어진다 —
   그 규칙을 무르게 하면 진짜 오류도 함께 통과하기 때문이다.
6. **문자열 완전일치는 `UNICODE`로 잰다.** `TEXT`는 시스템 ACP에 영향을 받는 한컴의
   ANSI 형식이므로 서로 다른 Windows host의 live Oracle 기준으로 쓰지 않는다. `TEXT`
   자체 규칙을 고칠 때는 별도 단위·패키지 계약 테스트를 함께 갱신한다.

계약을 적는 자리는 호출의 **세 번째 칸**이다. 규칙 전문과 이유는
[`scenario_spec.py`](scenario_spec.py) 가 갖는다.

```jsonc
{ "paths": {
    "picture": { "win": "C:\\Users\\...\\s1.jpg", "posix": "{repo}/samples/s1.jpg" },
    "out":     { "win": "C:\\Temp\\a.bmp",        "posix": "{out}/a.bmp" } },
  "calls": [
    ["InsertPicture",   [{"$path": "picture"}, true, 0]],
    ["CreatePageImage", [{"$path": "out"}, 9], {"expect": false}],
    ["SetCurFieldName", ["새이름"], {"expectError": {
        "rhwp": "필수 매개 변수입니다",   // rhwp 오류가 이 문구를 담아야 한다
        "ocx":  null,                     // 오라클 문구 미측정 — "죽는가"까지만 본다
        "why":  "실물은 인자 넷을 다 요구한다(§4.54·§4.57)" }}]
  ] }
```

- `expect` 는 Linux 자체 검사와 Windows 오라클 대조가 **같은 한 값**을 보게 한다. 경로가
  플랫폼마다 갈리면 인자는 더 이상 공통 닻이 아니라, 이 자리가 유일한 닻이다.
- `expectError` 는 면제가 아니라 계약이다. 안 죽어도 실패, 딴 문구로 죽어도 실패,
  `MissingApi` 로 죽으면 선언했든 말든 실패다. 선언 없이 양쪽이 죽은 자리는 `MATCH` 가 아니라
  `ERROR_UNDECLARED` 로 센다.
- 오라클 문구를 안 쟀으면 `null` 로 **남겨 둔다.** 안 잰 것을 지어 적으면 그 초록이 거짓이 된다.
  Windows 에서 재서 채우면 그때부터 문구까지 대조된다.

`ledger` 를 고를 때는 **그 시나리오가 증거가 되는 항목만** 적는다. 축이 섞이면 무관한 실패가
이미 증명된 API 를 영원히 막는다(`RenameField` 가 `MovePos` 부재에 막혀 있었다).

## COM 규약 — 어기면 오판이 난다

- **문서 하나당 프로세스 하나.** 한 프로세스에서 `Hwp()` 두 번은 `com_error` 로 죽는다.
- **동시 실행 금지.** 서로의 `Hwp.exe` 를 죽여 "무응답" 오판을 만든다. `run_gate.py` 는 직렬이다.
- **시간 제한과 정리.** 시작 시 한글 프로세스가 하나라도 있으면 `OCCUPIED`로 중단하며, 종료하지
  않는다. `com.Quit()` 뒤 최대 10초 동안 자연 종료를 기다린 뒤에도 남은 PID만 `LEFTOVER`로
  실패시켜 자동 종료하지 않는다. 전용 Windows 계정에서만 명시적으로 `--cleanup-spawned`를 주면
  그 실행 뒤 새 PID를 종료할 수 있다.
- **오라클은 한글2022(major 12)로 고정한다.** `12, 0, 0, 4547`과 `12.0.0.4547` 표기를 모두
  12로 판정한다. 버전이 어긋나면 **시나리오를 아예 돌리지 않고** 이전 `returns.json`과 저장본을
  지운 뒤 `<id>.rejected.json`만 남긴다. `--skip-ocx`도 기존 정답지의 버전을 다시 검사해 다른
  버전 또는 손상된 산출물을 비교에 사용하지 않는다.
  이 머신에는 2024(13.x)도 깔려 있고, 지금은 ProgID 가 2024 로 붙는다 — 전환 방법은
  계획서 §4.5.1(관리자 권한 `/regserver` + `gen_py` 캐시 삭제 + 정답지 재수집).
  2024 로 수집했던 산출물은 `output/poc/hwpctrl/ocx-2024-quarantine/` 에 격리해 두었다.

## 3자 차등 대조 — 오라클 이원화 (계획서 §6.3.3·§9-6 — PR #4470)

기준은 **웹한글 기안기**(실물)이고 COM 한글2022 는 대량·회귀 프록시다. `runner_webhwp.mjs` 가
헤드리스 브라우저로 기안기(기본: 한컴 공개 데모)를 몰아 같은 시나리오를 실행하고, `compare3.py`
가 COM·기안기·rhwp 세 산출물을 한 표에 놓는다.

```bash
# 소수 시나리오 권장 — 저빈도 수동 전용, CI 에 물리지 않는다
python tools/hwpctrl_compat/run_3way.py --only doc-basic --only field-read

# 자가 호스팅 웹한글 서버가 있으면
python tools/hwpctrl_compat/run_3way.py --url https://hwp.example.go.kr/webhwpctrl/
```

3자 판정 코드 — **어느 둘이 같은지가 곧 판정이다**:

| 코드 | 뜻 |
|---|---|
| `ALL_AGREE` | 셋이 같다 — COM 이 유효한 프록시라는 증명. 기존 `verified` 가 그대로 선다 |
| `COM_DRIFT` | 기안기 = rhwp ≠ COM — 프록시의 한계. rhwp 는 이미 제품과 맞다 |
| `IMPL_GAP` | 기안기 = COM ≠ rhwp — 두 오라클이 함께 확인한 실 결함 |
| `WEB_DIVERGES` | COM = rhwp ≠ 기안기 — 웹 계약이 갈리는 지점. **기안기가 이긴다** — 재검증 대상 |
| `ALL_DIFFER` | 셋 다 다르다 |

오류는 종류 무관하게 "죽었다"로만 묶는다 — 문구는 러너·플랫폼마다 달라 러너 차이가 판정을
오염시킨다.

**Windows 에서 3자를 한 번에 닫는 순서** — COM 정답지를 만드는 그 기계에서:

```bash
npm --prefix npm/hwpctrl-ocx run gate        # ① live COM 정답지(ocx/) + rhwp 산출물
python tools/hwpctrl_compat/run_3way.py      # ② 기안기 측정(webhwp/) + 3자 판정(verdict3/)
```

Chrome 은 표준 설치 경로를 자동으로 찾는다(Edge 폴백 포함). 다른 곳에 있으면 `CHROME_PATH` 로
지정한다.

규율 넷. (1) **저빈도 수동 전용** — 기본 URL 이 한컴 공개 데모라 반복 폭주로 몰지 않는다.
(2) **버전 스탬프 강제** — 러너가 URL·측정 시각·`HwpCtrl.Version` 을 남기고 `compare3.py` 는
스탬프 없는 산출물을 거부한다. 데모의 버전이 곧 현장 버전이 아니기 때문이다. (3) `SaveAs` 는
브라우저 다운로드 경로라 이 축에서 태우지 않는다(L3 제외). `$path` 인자는 posix 갈래로 푼다 —
실물에 로컬 파일계가 없으니 그 호출이 어떻게 답하는지 자체가 관측이다. (4) **업로드 채널
부산물은 판정 잣대 밖** — 기안기 `Open` 봉투의 `fileName` 은 서버 부여 난수라 `result` 만
비교한다(`compare3.WEB_ENVELOPE_PROJECTIONS`). 봉투 전체는 returns.json 에 남는다.

첫 3자 실측(2026-08-10)이 드러낸 것: **웹 계약 형태가 곧 기준**이라 COM 반환의 정규화 구멍
넷이 함께 드러났다 — `GetText` 는 `{result, text}` 객체, `GetSelectedPos` 는 `result` 없는
여섯 키, `SetTextFile` 은 bool, `GetTextFile` 은 `&#N;` escape 없는 원문이 실물 계약이다.
`runner_ocx.py` 의 `ADAPTERS` 와 impl 이 같이 그 형태를 따른다.

러너 자체 검증은 목으로 한다(네트워크·데모 불필요):

```bash
node tools/hwpctrl_compat/runner_webhwp.mjs <시나리오> --out <출력> \
  --url "file://$PWD/tools/hwpctrl_compat/fixtures/webhwp_mock/mock.html"
```

## 판정 코드

`MATCH` / `MISSING_API`(rhwp 에 그 API 없음) / `VALUE_DIFF` / `ERROR_DIFF` / `OCX_ERROR`.
L3(문서 상태)는 저장본을 같은 파서로 읽어 쪽수·필드값을 대조한다. L4(픽셀)는 시각에 영향을
주는 축(P4~P5)에서 붙인다.

## 산출물

`output/poc/hwpctrl/` 아래 — `ocx/`(Windows에서 재사용할 로컬 정답지) · `rhwp/`(구현물) ·
`verdict/`(판정) · `legacy-cjs/`(기존 층 트랜스파일 산출물, 재생성 가능). 공유·재현 가능한
정답지는 `tools/hwpctrl_compat/fixtures/hancom2022/`에만 넣으며, 출력 디렉터리를 Git에 추가하지
않는다.
