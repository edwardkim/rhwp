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

규칙 세 가지.

1. **`ledger` 를 반드시 적는다.** 원장은 시나리오 단위로 올라간다 — 선언이 없으면 무엇을
   검증했는지 알 수 없고, 그 실행은 진척으로 세지 않는다.
2. **바꾼 뒤에는 반드시 읽는다.** 반환값만 보면 아무 일도 하지 않는 구현이 통과한다.
   실제로 `MovePos` 는 `true` 만 돌려주고 커서를 옮기지 않아도 반환값 대조를 통과했다.
   뒤따르는 `GetPos` 가 그것을 잡았다.
3. **객체가 돌아오면 점을 찍어 들어간다.** 서식(`CharShape`)·개체 속성은 ParameterSet
   **객체**로 오는데, 러너는 객체를 `{__type: …}` 으로 줄인다. `["CharShape.Item", ["Height"]]`
   처럼 적어야 값이 대조된다 — 안 그러면 빈 셋을 돌려주는 구현도 통과한다.

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

## 판정 코드

`MATCH` / `MISSING_API`(rhwp 에 그 API 없음) / `VALUE_DIFF` / `ERROR_DIFF` / `OCX_ERROR`.
L3(문서 상태)는 저장본을 같은 파서로 읽어 쪽수·필드값을 대조한다. L4(픽셀)는 시각에 영향을
주는 축(P4~P5)에서 붙인다.

## 산출물

`output/poc/hwpctrl/` 아래 — `ocx/`(Windows에서 재사용할 로컬 정답지) · `rhwp/`(구현물) ·
`verdict/`(판정) · `legacy-cjs/`(기존 층 트랜스파일 산출물, 재생성 가능). 공유·재현 가능한
정답지는 `tools/hwpctrl_compat/fixtures/hancom2022/`에만 넣으며, 출력 디렉터리를 Git에 추가하지
않는다.
