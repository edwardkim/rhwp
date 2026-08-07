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

```bash
python tools/hwpctrl_compat/run_gate.py --impl legacy
```

`--impl legacy` 는 기존 studio 층을 대상으로 하는 **하니스 자체 검증용**이다. P1 부터는
신규 패키지 엔트리 경로를 준다(`--impl npm/hwpctrl-ocx/src/index.mjs`).

## 스펙 재생성

정답지는 한컴 공식 문서 3종이다. 추측으로 채우지 않는다.

```bash
python tools/hwpctrl_compat/extract_spec.py   # samples/hwpctl_*.hwp → npm/hwpctrl-ocx/spec/*.json
python tools/hwpctrl_compat/build_ledger.py   # 스펙 → 원장 골격(기존 상태 보존)
python tools/hwpctrl_compat/build_ledger.py --check   # CI 게이트: 스펙↔원장 불일치 시 exit 1
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

규칙 두 가지.

1. **`ledger` 를 반드시 적는다.** 원장은 시나리오 단위로 올라간다 — 선언이 없으면 무엇을
   검증했는지 알 수 없고, 그 실행은 진척으로 세지 않는다.
2. **바꾼 뒤에는 반드시 읽는다.** 반환값만 보면 아무 일도 하지 않는 구현이 통과한다.
   실제로 `MovePos` 는 `true` 만 돌려주고 커서를 옮기지 않아도 반환값 대조를 통과했다.
   뒤따르는 `GetPos` 가 그것을 잡았다.

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

`output/poc/hwpctrl/` 아래 — `ocx/`(정답지) · `rhwp/`(구현물) · `verdict/`(판정) ·
`legacy-cjs/`(기존 층 트랜스파일 산출물, 재생성 가능).
