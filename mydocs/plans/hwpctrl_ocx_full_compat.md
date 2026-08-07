---
kind: plan
status: draft
canonical: mydocs/plans/hwpctrl_ocx_full_compat.md
last_verified: 2026-08-07
---

# 수행계획서 — HwpCtrl OCX 100% 호환 대체 층

- **이슈**: 미발행 (착수 승인 시 발행)
- **브랜치**: `feat/hwpctrl-ocx-compat` (Phase 별 하위 브랜치)
- **기록 시각**: 2026-08-07 KST
- **절차 상태**: P0 착수 완료(2026-08-07) — 하니스·스펙·원장 구축, §4.5 참조
- **배포 형태**: 독립 npm 패키지 `@rhwp/hwpctrl` (작업지시자 결정, 2026-08-07)
- **목표**: 웹에서 쓰는 한컴 `HwpCtrl` ActiveX 컨트롤을 rhwp WASM 으로 **호출 호환**
  대체한다. 기존 통합 페이지의 스크립트를 **한 줄도 고치지 않고** 동작시키는 것이 합격선이다.
- **판정자**: 이 머신에 설치된 한글(오라클 = 한글2022, `hangul-oracle-version-2022`)의 OCX.
  구현물과 **같은 스크립트를 양쪽에 돌려 대조**한다.

---

## 0. 결론 먼저

1. **기존 `rhwp-studio/src/hwpctl/` 은 신규 층이 완성될 때까지 손대지 않는다.** 그 층은
   "studio 자동화 표면"이라는 **다른 계약**을 갖는다(예: `Undo` 를 정책상 미지원으로 판정 —
   `actions/clipboard.ts:60`). 100% 호환 층은 **별도 패키지**로 새로 세운다.
   **보존은 목적이 아니라 순서다** — 신규 층이 원장 100% 에 도달하면 P7 에서 **기존 층을 철거하고
   studio 를 신규 층으로 이관**한다(§6.2).
2. **먼저 짓는 것은 API 가 아니라 오라클이다.** 312개 액션·50종 ParameterSet 의 정확한 동작은
   문서가 아니라 **설치된 OCX 만** 안다. Phase 0 은 코드 변경 0 으로, 차등 테스트 하니스와
   기계 판독 가능한 **API 원장**을 만든다.
3. **"100%" 는 정의해야 측정된다.** §1 에서 4계층 계약으로 쪼개고, 물리적으로 불가능한 축
   (로컬 파일시스템 경로, 인쇄 드라이버, 프로세스/윈도우 객체)은 **명시적 대체 계약**으로 고정한다.
   여기를 흐리면 "100%" 가 영원히 판정 불가가 된다.
4. **진척은 원장 숫자로만 보고한다.** `api_ledger.json` 이 유일한 진실이고, 각 항목의 상태는
   사람 판단이 아니라 **오라클 대조 결과**가 채운다.

---

## 1. "100% 호환"의 정의 — 4계층 계약

| 계층 | 무엇을 맞추나 | 판정 방법 |
|---|---|---|
| **L1 표면** | 객체·메서드·속성의 **존재**, 인자 개수, 반환 타입 | OCX 타입 라이브러리 열거 ↔ 구현 열거 diff |
| **L2 반환값** | 같은 입력에 대한 **반환값** (Boolean/Integer/String/ParameterSet 아이템) | 시나리오 실행 로그 JSON diff |
| **L3 문서 상태** | 호출 뒤 **문서가 같아지는가** | 양쪽 저장본을 rhwp 로 파싱해 IR/레코드 대조 |
| **L4 시각** | 저장본을 조판했을 때 **같이 보이는가** | 한글 PDF ↔ rhwp `export-png` 픽셀 대조 |

**합격선**: L1·L2 는 전 항목 0 diff. L3 는 대상 API 가 건드리는 레코드에 한해 0 diff.
L4 는 기존 시각 게이트 기준선(`visual-sweep-oracle`, 정상 ~92-93%) 이하로 떨어지지 않을 것.

### 1.1 물리적으로 대체 불가한 축 — 명시적 계약

브라우저에는 없는 것들이다. **"미지원"이 아니라 "대체 계약"으로 문서화**하고, 원장에
`substituted` 상태로 기록한다.

> **갱신(P0)**: 아래 표의 대부분은 우리가 정할 일이 아니었다. 웹한글컨트롤 규격 v2.4 §2.2 가
> 이미 정해 놓았다(§4.4). 규격과 어긋나는 자체 계약을 만들지 말 것 — 규격이 먼저다.

| OCX 동작 | 브라우저 대체 계약 |
|---|---|
| `Open("C:\...\a.hwp")` 로컬 경로 | 경로 문자열을 **가상 FS 키/URL** 로 해석. 호스트가 등록한 리졸버 경유 |
| `SaveAs(path)` 로컬 저장 | 다운로드 트리거 또는 호스트 콜백(`onSave(bytes, path)`) |
| `PrintDocument` 프린터 드라이버 | PDF 생성 후 브라우저 인쇄 대화상자 (`rhwp-studio/src/ui/pdf-print-dialog.ts` 재사용) |
| `ShowToolBar`/`ShowRibbon` 등 UI 7종 | **no-op + `true` 반환** (OCX 도 반환값을 안 쓰는 호출) |
| `XHwpWindows`/프로세스 객체 | 단일 문서 인스턴스로 고정, 열거 시 1개짜리 컬렉션 |
| `RegisterModule`(보안 모듈) | no-op — 브라우저에는 파일 접근 승인 다이얼로그가 없다 |

이 표에 없는 API 가 "브라우저라 못 한다"는 이유로 미구현되면 **계획 위반**이다.

---

## 2. 현재 지점 (2026-08-07 실측)

정답지는 `mydocs/plans/archives/task_43_feature_def.md` §2 (Properties 18 / Methods 67 /
Action 312 / ParameterSet 50). 같은 문서 §3 의 매핑표는 낡았으므로 **현재 코드로 다시 셌다**.

| 축 | OCX 총수 | 기존 hwpctl 이름 호환 | 비고 |
|---|---|---|---|
| Properties | 18 | 1 (`PageCount()`) | 메서드 형태라 L1 부적합 |
| Methods | 67 | 17 (실동작 14) | `SetCurFieldName`·`RenameField` 는 stub (`index.ts:388,394`) |
| Actions | 312 | 30 등록 / 28 실행 | `Undo`/`Redo` 정책 미지원 |
| ParameterSet | 50종 | 0종 (범용 key-value 1개) | `parameter-set.ts` 73줄 |

**있는 자산**: rhwp WASM 366 함수(`src/wasm_api.rs`), studio 명령 156개(12 stub).
표·서식·검색치환·필드·페이지·머리말꼬리말은 **기능 등가가 이미 있다** — 새로 만들 것은
대부분 "능력"이 아니라 **OCX 모양의 배선**이다.

**등가물이 아예 없는 축** (신규 구현 필요): 맞춤법 검사, 음/양력 변환 4종, 하이퍼링크 삽입·수정,
개체 정렬 11종·캡션 부착, 쪽 배경 그림, 명령 잠금 3종, 스캔 커서(`InitScan`/`GetText`/`ReleaseScan`),
이벤트 발화.

---

## 3. 아키텍처 — 기존 보존 + 코어 공유

```
  기존 (건드리지 않음)                    신규
  ─────────────────────                  ──────────────────────────────
  rhwp-studio/src/hwpctl/                npm/hwpctrl-ocx/            ← 신규 패키지
    index.ts  (HwpCtrl, studio 계약)       src/hwpctrl.ts   (window.HwpCtrl, 67 methods/18 props)
    action.ts / action-registry.ts         src/actions/**   (312 action)
    actions/**  (30 action)                src/sets/**      (50 ParameterSet 스키마)
    parameter-set.ts                       src/objects/**   (Action/CtrlCode/ParameterSet/ParameterArray)
           │                                        │
           └──────────────┬─────────────────────────┘
                          ▼
              rhwp-studio/src/hwpctl-core/   ← 신규 공용 코어 (문서 상태기)
                cursor.ts · selection.ts · fields.ts · ctrl-walk.ts
                          ▼
                    WASM  HwpDocument  (src/wasm_api.rs)
```

**규칙**

1. 기존 `hwpctl/` 파일은 P0~P6 동안 **읽기 전용**으로 취급한다. 기존
   e2e(`rhwp-studio/e2e/hwpctl-basic.test.mjs`)는 그 기간 내내 녹색이어야 한다.
   **최종 상태는 단일 층이다** — P7 에서 기존 층은 삭제되고 studio 는 신규 층만 쓴다.
   따라서 신규 층은 처음부터 **studio 가 실제로 쓰는 호출**(§6.2 표)을 지원 범위에 포함시켜
   설계한다. 나중에 얹는 것이 아니다.
2. 신규 층은 `window.HwpCtrl` 전역을 심어 **기존 통합 스크립트를 무수정 실행**한다.
3. OCX 는 **동기** API 다. 신규 층도 동기로 만든다 — WASM 호출이 이미 동기다
   (`rhwp-studio/src/core/wasm-bridge.ts`). 비동기가 불가피한 축(URL 로딩)만 OCX 원래의
   `callback` 인자를 쓴다.
4. **코어에 없는 능력은 WASM 에 추가**한다(예: 하이퍼링크 삽입). 호환 층 안에서 임시 구현하지 않는다 —
   그러면 studio 와 동작이 갈라진다.

---

## 4. 오라클 하니스 — 이 계획의 심장

### 4.1 구조

```
scenario/*.json   ← 호출 시퀀스 (선언형 DSL)
      │
      ├── runner_ocx.py     설치된 한글 OCX (pyhwpx, 프로세스 격리)
      │       → returns.json · out.hwp · oracle.pdf
      │
      └── runner_rhwp.mjs   node + rhwp WASM (신규 호환 층)
              → returns.json · out.hwp · out/*.png
      ▼
compare.py  → L2 반환값 diff · L3 IR/레코드 diff · L4 픽셀 diff → verdict.tsv
```

시나리오 DSL 예 (한 파일 = 한 판정 단위):

```json
{ "id": "field-put-basic",
  "open": "samples/gian/기안문.hwp",
  "calls": [
    ["MoveToField", ["기안자", false, true, false]],
    ["PutFieldText", ["기안자", "홍길동"]],
    ["GetFieldText", ["기안자"]],
    ["GetPos", []]
  ],
  "saveAs": "out.hwp" }
```

### 4.2 재사용할 자산

- `tools/hwp_open_bisect/hangul_com.py` — 프로세스 격리 규약(문서당 프로세스 1개, 인스턴스
  재생성 금지)이 이미 문서화돼 있다. runner 의 뼈대로 쓴다.
- `tools/hangul_pdf_baseline.py`, 시각 스윕 자산 — L4 판정에 그대로 쓴다.

### 4.3 함정 목록 (하니스가 **먼저** 방어할 것)

과거 캠페인에서 실제로 오판을 만든 것들이다. 하니스 v1 에 전부 넣는다.

1. **COM hang** — stall-watchdog 필수. 시작 시 기존 한글 프로세스가 있으면 종료하지 않고
   `OCCUPIED`로 중단한다. `com.Quit()` 뒤 자연 종료를 최대 10초 기다린 뒤에도 남은 PID만
   `LEFTOVER`로 보고하고 자동 종료하지 않는다. 전용 Windows 계정에서만 `--cleanup-spawned`를
   명시해 새 PID를 정리할 수 있다.
2. **동시 실행 금지** — COM 판정은 직렬. 병렬은 서로의 인스턴스를 죽인다.
3. **보안 다이얼로그** — `FilePathCheckerModule.dll` 을 `HKCU\Software\HNC\HwpAutomation\Modules`
   에 등록(`hwp_com_automation` 메모).
4. **버전 고정** — 한글2022. 2024 는 조판은 같아도 계측이 불안정하다.
5. **정착(settle)** — 조판 전 `PageCount`/좌표를 읽으면 비결정 값이 나온다. 읽기 전 정착 게이트.
6. **PDF producer 확인** — `producer=Hancom PDF` 만 정답지.
7. **바이너리·산출물 신선도** — 왕복본·PNG·덤프까지 재생성. stale 산출물이 닫힌 결함을 다시 연다.

### 4.4 스펙 추출 — 추측이 아니라 공식 문서에서

착수하며 확인한 사실: **한컴 공식 규격서 3종이 이미 저장소에 있다.**

| 문서 | 내용 |
|---|---|
| `samples/hwpctl_API_v2.4.hwp` | 웹한글컨트롤 API v2.4 — 이 계획의 **대상 규격 그 자체** |
| `samples/hwpctl_Action_Table__v1.1.hwp` | Action 312개 + 각자의 ParameterSet ID |
| `samples/hwpctl_ParameterSetID_Item_v1.2.hwp` | ParameterSet 50종 / Item 521개 |

따라서 §4.4 의 원안(왕복 판별로 Item 을 추정한다)은 **불필요해졌다**. `extract_spec.py` 가
rhwp CLI 로 세 문서를 읽어 기계 판독 스펙을 만든다. 왕복 판별은 문서가 모호한 항목
(서명과 `Parameters N` 이 어긋나는 18건)에만 쓴다.

> **대상 규격이 ActiveX 가 아니라 웹한글컨트롤이다.** v2.4 §2.2 가 이미 브라우저 제약에 대한
> 답을 정해 놓았다 — `Open`/`GetTextFile` 은 콜백, `SaveAs`/`CreatePageImage` 는 다운로드,
> `InsertPicture` 는 업로드된 File, 이벤트는 `addEventListener`. §1.1 의 대체 계약표는
> **우리가 정하는 것이 아니라 규격이 정한 것**으로 바뀐다. 그만큼 "100%"의 판정이 쉬워졌다.

### 4.5 P0 실행 결과 (2026-08-07)

만든 것 — `tools/hwpctrl_compat/` (`README.md` 가 사용법):

| 파일 | 역할 |
|---|---|
| `extract_spec.py` | 공식 문서 3종 → `spec/*.json`. 개수가 문서 약속과 다르면 **실패한다** |
| `build_ledger.py` | 스펙 → 원장 484항목. `--check` = CI 게이트, `--ingest` = 판정 반영 |
| `runner_ocx.py` | 설치된 한글 COM 에 시나리오 실행 → 정답지 |
| `runner_rhwp.mjs` | 같은 시나리오를 rhwp WASM 위에서 실행 (`--impl legacy` = 자체 검증) |
| `compare.py` | L2 반환값 + L3 문서 상태 대조 → `verdict.tsv`/`verdict.json` |
| `run_gate.py` | 직렬 오케스트레이터 — 시간 제한·안전한 잔류 PID 보고·정답지 버전 재검증 |

**P0 완료 판정("기존 층을 돌려 알려진 diff 를 재현")을 충족했다.** 한글2022 정답지 기준
시나리오 3건 28호출 중 12 일치, 16 불일치가 잡혔고 그 내역이 전부 설명된다.

| 잡힌 것 | 내용 |
|---|---|
| `MISSING_API` 9 | `IsEmpty`·`IsModified`·`Version`·`EditMode`·`SelectionMode`·`FieldExist`·`GetCurFieldName` 등 |
| `VALUE_DIFF` — 반환 규약 | `PutFieldText` 는 규격상 void(오라클 `null`)인데 기존 층은 `true` |
| `VALUE_DIFF` — 자료 형태 | `GetFieldList` 는 규격상 `0x02` 구분 **문자열**, 기존 층은 객체 배열 |
| `VALUE_DIFF` — 커서 모델 | `GetPos().list` 는 OCX 가 서브리스트 id(292·8)를 주는데 rhwp 는 늘 0 |
| **L3 문서 결함** | `PutFieldText` 가 **인접 필드를 오염**시킨다(§4.6) |

#### 4.5.1 오라클 버전 — **한글2022 로 고정 완료** (2026-08-07)

작업지시자가 관리자 권한으로 `Hwp.exe /regserver`(Office 2022)를 실행해 전환했다. 이후
`gen_py` 캐시를 지우고 정답지를 **전량 재수집**했다 — 현재 `output/poc/hwpctrl/ocx/` 는 전부
`12, 0, 0, 4547` 산이다. 게이트 기본값이 `--expect-version 12` 이므로 `12, 0, 0, 4547`과
`12.0.0.4547`을 같은 2022 오라클로 판정하고, 다른 major로는 **시나리오가 실행되지 않는다**.
재실행은 이전 `returns.json`과 저장본을 먼저 제거한 뒤 `<id>.rejected.json`만 남겨 stale 정답지가
비교에 섞이지 않게 한다. `--skip-ocx`도 저장된 oracle version을 다시 검사한다.

**2022 와 2024 의 답은 같았다.** 격리해 둔 2024 산출물과 대조한 결과 28호출 중 다른 것은
`Version` 값 하나뿐이다(`12, 0, 0, 4547` ↔ `13, 0, 0, 564`). 즉 **API 축은 두 버전에 둔감**하다
— 계획서가 2022 로 고정하는 이유는 정확성이 아니라 **저장소의 다른 오라클(10k 레이아웃
캠페인)과 재현 조건을 맞추기 위해서**다. 레이아웃 축은 여전히 버전에 민감하다.

아래는 전환 과정 기록이다(같은 상황이 재발할 때를 위해 남긴다).

#### 4.5.1.1 전환이 왜 레지스트리만으로 안 됐나

이 머신에는 한글2022(`Office 2022/HOffice120`, 12.0.0.4547)와 2024(`HOffice130`)가 함께 있다.
`HWPFrame.HwpObject` 의 CLSID·ProgID·TypeLib GUID 는 **두 버전이 공유**하고, 등록은 2024 가
전부 가져갔다(2022 를 가리키는 CLSID 는 레지스트리에 하나도 없다).

`HKCU\Software\Classes` 에 CLSID·TypeLib 을 우선 등록하면(관리자 불필요) 레지스트리는 바뀐다
— `HKCR…\LocalServer32` 가 2022 exe 를 가리킨다. **그런데 실제 기동 프로세스는 여전히
`HOffice130\Bin\Hwp.exe`(13,0,0,564)였다.** 2022 바이너리가 2024 로 넘겨준다. 버전 중재는 CLSID
등록이 아니라 한컴 내부(공유 설치 정보)에서 일어난다.

따라서 전환 경로는 **관리자 권한 `/regserver`** 뿐이다. CLSID 뿐 아니라 공유 설치 정보까지
갱신한다.

```powershell
# 관리자 PowerShell
& 'C:\Program Files (x86)\Hnc\Office 2022\HOffice120\Bin\Hwp.exe' /regserver
# 되돌리기
& 'C:\Program Files (x86)\Hnc\Office 2024\HOffice130\Bin\Hwp.exe' /regserver
```

전환한 뒤 반드시 두 가지를 한다(이번에 실제로 했다).

1. `%LOCALAPPDATA%\Temp\gen_py\3.12` 삭제 — `win32com` 이 옛 타입라이브러리로 만든
   조기 바인딩 래퍼가 남아 있으면 서버만 바뀌고 인터페이스는 옛것을 쓴다.
2. **정답지를 다시 만든다.** 이전 버전으로 모은 산출물은 폐기한다
   (`output/poc/hwpctrl/ocx-2024-quarantine/` 에 격리해 두었다 — 비교용이며 정답지가 아니다).

#### 4.5.2 시나리오 설계 규칙 — 반환값만 보면 통과한다

첫 판정에서 `MovePos` 가 `verified` 로 올라갔다. `true` 만 돌려주고 **커서를 옮기지 않아도**
반환값 대조를 통과했기 때문이다. 뒤따르는 `GetPos` 가 그것을 잡았다. 그래서 원장 반영을
**시나리오 단위**로 바꿨다 — 시나리오가 통째로 통과해야 그 시나리오가 선언한 항목이 올라간다.
시나리오는 **바꾼 뒤 반드시 읽어야** 한다.

### 4.6 P0 이 찾아낸 결함 — `PutFieldText` 필드 오염

`samples/issue-986-receipt.hwp` 에서 `PutFieldText("med_str_dt", "2026-08-07")` 한 번 호출 후
저장본을 대조한 결과다.

| 필드 | 한글(정답) | rhwp |
|---|---|---|
| `med_str_dt` | `2026-08-07` | `2026-08-07부터 ` |
| `med_end_dt` | (빈 값) | `2026-08-07` |

같은 셀에 있는 **인접 필드까지 값이 번지고**, 기존 텍스트가 지워지지 않는다. 규격 §8.3.34 는
"현재 필드에 입력되어 있는 내용은 지워진다"고 못박는다.

**절반은 좁혀 놓았다(P1 조사, 2026-08-07).**

| 관측 | 결론 |
|---|---|
| 하니스 rhwp 측 `GetFieldText` 가 쓰기 직후 `"2026-08-07"` 를 정확히 돌려준다 | **메모리 상태는 옳다** — 쓰기 경로 무혐의 |
| CLI `edit fill-fields` 로 저장한 뒤 다시 읽으면 오염 | 저장·재적재 경계의 문제 |
| HWP5 저장본과 HWPX 저장본에서 **동일하게** 재현 | 특정 직렬화기 결함이 아니다 — 공통 경로(필드 범위 ↔ 컨트롤 짝짓기) |
| 두 필드는 **같은 셀·같은 문단**(cell 7, para 0)에 있고 원래 둘 다 빈 값 | 빈 필드 두 개가 한 문단에 있을 때 재적재가 범위를 잘못 짝지음 |
| 재적재된 `med_str_dt` 값이 뒤 필드의 **안내문("진료 종료일")까지 삼킨다** | 범위 경계가 다음 필드 시작을 넘어간다 |

`Paragraph::insert_text_at`/`delete_text_at` 는 `field_ranges` 를 이미 시프트한다
(`src/model/paragraph.rs` §5-1) — 그래서 메모리가 맞는다.

#### 4.6.1 근인 — `rebuild_char_offsets` 가 필드 시작 위치를 **추정**한다

`src/document_core/queries/field_query.rs:1429`:

```rust
let ctrls_before_text = if !para.char_offsets.is_empty() {
    para.char_offsets[0] as usize / 8      // ← 첫 갭 크기로 "앞선 컨트롤 수"를 추정
} else { para.controls.len() }.min(para.controls.len());

for fr in &para.field_ranges {
    if fr.control_idx >= ctrls_before_text {   // ← 이 조건이 거짓이면
        field_begin_at[fr.start_char_idx.min(text_len)] += 1;   // 시작 위치 갭을 안 만든다
    }
}
```

문제의 셀은 텍스트가 `'부터 까지'` 이고 **빈 필드 두 개**가 그 안에 있다. 첫 갭이 커서
`ctrls_before_text` 가 2 로 추정되고, 두 필드의 `control_idx` 는 0·1 이라 **둘 다 조건에서
탈락**한다. 그러면 두 FIELD_BEGIN 이 모두 "텍스트 앞 선행 컨트롤"로 취급돼 **위치 0 에 붙어**
저장된다.

파서는 스택(LIFO)으로 짝짓는다(`src/parser/body_text.rs:328`). 위치 0 에 BEGIN 두 개가 붙은
스트림을 되읽으면:

```
BEGIN(a) BEGIN(b) … END@10 → b 에 [0,10)   … END@13 → a 에 [0,13)
```

즉 `med_end_dt`(b) = `'2026-08-07'`, `med_str_dt`(a) = `'2026-08-07부터 '` — **관측값과 정확히
일치한다.** 텍스트 자체는 온전하다(`export-tables` 로 `'2026-08-07부터 까지'` 확인).

#### 4.6.2 수정 시도 2회 — **둘 다 게이트가 되돌렸다** (2026-08-07)

| 시도 | 내용 | 결과 |
|---|---|---|
| ① `rebuild_char_offsets` 교정 | FIELD_BEGIN 을 `start_char_idx` 에 놓고, 선행 갭 상한을 **필드가 아닌** 컨트롤 수로 | 재현 케이스 해결. 그러나 `issue_2007_saved_frame_tail_nested_table…` **회귀** |
| ②  ① 을 좁힘 | 시작이 0 인 필드는 선행 갭에 유지, 중간 시작 필드만 자기 자리에 + 예산 차감 | 재현 케이스 해결. 그러나 같은 파일의 **다른** 테스트(`…intra_paragraph_saved_frame_break…`) 회귀 |
| ③ 직렬화기 차단 | 갭 채우기 루프가 필드 시작 전에 FIELD_BEGIN 을 방출하지 못하게 | 재현 케이스 해결 + `issue_2007` 4건 **전부 통과**. 그러나 **필드 소실**(165→164) |

**①·② 가 조판을 흔든 이유**: 적재 시 `clear_initial_field_texts` 가 필드 쓰기 경로를 타고
`rebuild_char_offsets` 를 부른다. 즉 이 함수는 저장 전용이 아니라 **적재 정규화 경로**이며,
`char_offsets` 는 `char_shapes`/`line_segs` 와 같은 좌표계를 공유하므로 값을 바꾸면 줄바꿈이
움직인다. "조판과 무관한 수정"이라는 초기 판단은 **틀렸다.**

**③ 이 필드를 잃은 이유**: 방출을 막기만 하면 그 FIELD_BEGIN 이 갈 자리를 못 찾는다. 갭
예산은 텍스트 편집 뒤 실제 구조보다 크게 남아 있고, 뒤쪽 위치에는 여유가 없다.

#### 4.6.3 다음 시도의 설계 (미착수)

③ 에 **강제 방출**을 더해야 한다. 위치 `i` 에서 방출 순서를 명시적으로 정한다.

1. `end == i` 이고 **BEGIN 이 이미 나간**(`start < i`) 필드의 FIELD_END
2. `start == i` 인 필드의 FIELD_BEGIN — 갭 예산과 무관하게 **강제 방출**
3. `start == i && end == i` 인 빈 필드의 FIELD_END

파서가 스택(LIFO)으로 짝지으므로 이 순서를 어기면 범위가 뒤엉킨다(1 과 2 의 순서가 특히
중요하다 — 한 필드가 끝나는 자리에서 다음 필드가 시작하는 경우). 현재 `field_ends` 는
인덱스별 마커 목록이라 범위와 연결돼 있지 않으므로, **`field_ranges` 를 직접 훑는 구조로
바꿔야 한다.**

검증 게이트(3중): `cargo nextest --no-fail-fast`(편집 계약) + `ir_field_sweep`(필드 IR 스윕)
+ 본 하니스 `field-put-save` L3 MATCH. 추가로 **필드 개수 보존**(165→165)을 회귀 검사에 넣는다
— ③ 의 소실을 잡은 것이 바로 이 확인이었다.

#### 4.6.4 시도 ④ — **채택** (2026-08-07)

`src/serializer/body_text.rs` `serialize_para_text` 에 §4.6.3 순서를 구현했다. 빈 필드의
FIELD_END 를 별도 맵(`empty_field_ends`)으로 가르고, 필드를 여는 컨트롤은 `field_begin_pos`
로 자기 시작 전 방출을 막되 **시작 위치에서는 갭 예산과 무관하게 강제 방출**한다.

| 판정 | 결과 |
|---|---|
| 재현 케이스 | `med_str_dt='2026-08-07'`, `med_end_dt=''` — **오라클과 일치** |
| 필드 개수 | 165 → **165** (시도 ③ 의 소실 없음) |
| ① `cargo nextest --no-fail-fast` | 5,287 통과 / 1 실패 — **clean devel 과 동률**(같은 테스트) |
| ② `ir_field_sweep_baseline` | **통과** (2 passed, 600s) |
| ③ 하니스 `field-put-save` L3 | **DOC_DIFF → MATCH** |

잔여 실패 `issue_2833…inflated_row_count_does_not_slow_down_parsing` 은 500ms **벽시계 상한**
테스트이고 clean devel 전체 실행에서도 똑같이 실패한다 — 이 머신의 사전 실패다.

#### 4.6.5 이 조사에서 얻은 방법 교훈

`issue_2007` 조판 테스트가 "수정본 전체 실행"에서만 깨져 두 번 오귀속했다. 원인은 코드가
아니라 **부하**였다 — 그 실행 동안 `ir_field_sweep`(10분)·`wasm-pack`(4분)·COM 하니스가 같은
머신에서 돌았고, 대조군인 clean devel 실행에는 경쟁 작업이 없었다. 조판은 폰트 환경에 의존해
부하에 흔들린다.

**전체 스위트끼리 비교할 때는 부하 조건도 대조군에 맞춰야 한다.** 무부하로 다시 돌리자
수정본도 clean devel 과 같은 1건 실패로 수렴했다. 덧붙여 nextest 는 **Monitor 로** 돌린다 —
배경 Bash 10분 상한에 17분짜리 실행이 잘린다(이 조사에서 한 번 더 겪었다).

경로가 코어이므로 CLI `edit fill-fields` 와 studio 가 **같은 결함을 공유한다.**

---

## 5. API 원장과 게이트

`npm/hwpctrl-ocx/spec/api_ledger.json` — 항목당:

```json
{ "kind": "method", "name": "PutFieldText", "arity": 2, "returns": "void",
  "status": "verified",            // unimplemented | implemented | verified | substituted | by-design-noop
  "oracle": { "scenarios": ["field-put-basic", "field-put-multi"], "lastRun": "r1", "diff": 0 } }
```

- `status: verified` 는 **오라클 대조 0 diff 로만** 올라간다. 사람이 손으로 못 올린다.
- CI 게이트 ①: 원장에 없는 API 가 코드에 있거나, 코드에 없는 API 가 원장에 `implemented` 면 실패.
- CI 게이트 ②: 원장 커버리지가 직전 커밋보다 **줄면** 실패(래칫).
- `GetActionSupport` 를 신규 층에도 두되 원장에서 생성한다 — 손으로 관리하지 않는다.
- 진척 보고는 항상 `verified / 447` (18+67+312+50) 형태의 한 숫자로 한다.

---

## 6. 단계 계획

각 Phase 완료 판정 = **해당 축 원장 전 항목 `verified`** + studio 회귀 0 + 10k 쪽수/시각 게이트 회귀 0.

| Phase | 범위 | 규모 | 산출물 | 완료 판정 |
|---|---|---|---|---|
| **P0** 하니스 ✅ | 코드 변경 0 | — | runner 2종 · compare · 원장 484 · 공식 스펙 3종 추출 | **충족**(2026-08-07) — 28호출 중 16 diff 를 재현하고 전부 설명, §4.5 |
| **P1** 문서 I/O + 필드 | Methods 8+10, Actions 4 | 22 | `Open`/`SaveAs`/`Clear`/`Insert`, 필드 10종, `CreateField`, `FieldExist` | 기안문 시나리오(Open→PutFieldText→SaveAs) L1~L4 0 diff |
| **P2** 커서·선택 | Actions 51+36, Methods 9+4 | 100 | 공용 코어 `cursor.ts`/`selection.ts`, `MovePos` 28종 이동 타입, `KeyIndicator` | 이동/선택 시퀀스 후 `GetPos`·`GetSelectedPos` 전수 일치 |
| **P3** 텍스트·서식 | Actions 29+33+27 | 89 | `CharShape`/`ParaShape` 전 아이템 왕복, 번호/글머리표 | 서식 적용 후 L3 레코드 diff 0 |
| **P4** 표·셀 | Actions 50+6, Methods 2 | 58 | 셀 블록·이동·크기조절, `TableCellBlock*`, `CellZone*` | 표 편집 시퀀스 L3·L4 0 diff |
| **P5** 개체·페이지·뷰·편집제어 | Actions 46+4+3+1+3+10 | 67 | 개체 정렬 11종·캡션(**신규 WASM 필요**), 쪽 배경 그림, 명령 잠금 | 개체 조작 L4 0 diff |
| **P6** 잔여 | Methods 잔여 + 이벤트 | ~30 | 스캔 커서, 음/양력, 맞춤법(**신규**), 이벤트 발화, `GetTextFile`/`SetTextFile` 전 포맷 | 원장 100% |
| **P7** 이관·철거 | 기존 hwpctl 층 | 12 파일 | studio 를 신규 층으로 이관, `src/hwpctl/` 삭제 | §6.2 판정 |

**규모 총계 447 항목.** P1~P2 가 기존 통합 스크립트의 90% 를 차지한다(기안문 패턴) — 여기까지가
실사용 전환점이고, P3 이후는 완전 호환을 위한 꼬리다.

### 6.1 Phase 내부 작업 루프

1. 대상 API 의 시나리오를 **먼저** 쓴다 (OCX 만으로 돌려 정답지 확보).
2. OCX 반환값을 보고 계약을 확정한다 — 문서가 아니라 실측이 계약이다.
3. 구현 → 하니스 재실행 → diff 0 → 원장 `verified`.
4. PR 단위: 축 하나(예: "필드 10종"), fmt+clippy, studio e2e, 하니스 리포트 첨부.

### 6.2 P7 — 기존 층 철거와 studio 이관

최종 상태는 **단일 층**이다. 두 층을 영구히 병존시키면 같은 결함을 두 번 고치게 된다.

#### 6.2.1 실측한 이관 범위 (2026-08-07)

착수 전에 실제 소비자를 셌다. **`rhwp-studio` 본 편집기(`main.ts`)는 `src/hwpctl/` 을 쓰지 않는다.**
`grep -rn "hwpctl" rhwp-studio/src --include=*.ts` 결과에서 `src/hwpctl/` 자신을 빼면 남는 것은
주석 1건(`view/flow-image-url-cache.ts:87`)과 무관한 legacy 메시지 별칭
(`embed/runtime.ts:99` — `hwpctl-load` 는 `loadFile` 의 옛 이름일 뿐 `HwpCtrl` 클래스와 무관)뿐이다.

| 소비자 | 성격 | P7 처리 |
|---|---|---|
| `rhwp-studio/hwpctl-test.html` | 데모·수동 시험 페이지 (`./src/hwpctl/index.ts` 동적 import) | 신규 층 import 로 교체 |
| `rhwp-studio/e2e/hwpctl-basic.test.mjs` | e2e 회귀 | 신규 층 기준으로 재작성(§6.2.3) |
| `embed/runtime.ts:99` `hwpctl-load` | 이름만 같은 별칭 | **무관 — 건드리지 않는다** |
| studio 편집기 UI 156 명령 | `HwpCtrl` 경유 아님, WASM 직접 호출 | **무관 — 이관 대상 아님** |

즉 이관 비용은 **파일 2개**다. 이 사실이 "기존 층을 P6 까지 동결"하는 결정을 싸게 만든다 —
동결 비용이 거의 0 이므로 신규 층을 서두를 이유가 없다.

> 이 절은 계획 초안의 "studio 가 main.ts 경유로 실사용 중"이라는 서술을 실측으로 대체한 것이다.

#### 6.2.2 철거 순서

1. **동결 확인** — P6 종료 시점에 §6.2.1 표를 다시 실측한다(그 사이 새 소비자가 생겼을 수 있다).
2. **계약 차이 목록화** — 신규 층이 기존 층과 **다르게 답하는 지점**을 전수로 뽑는다. 알려진 것:
   - `Undo`/`Redo`: 기존 = `notSupportedByDesign`, 신규 = **OCX 와 동일하게 동작해야 함**.
     studio 이관 시 신규 층의 undo 가 studio 히스토리(`engine/history.ts`)와 **이중 관리**가 되지
     않도록, 신규 층 undo 를 studio 히스토리에 위임할지 여기서 결정한다.
   - 비표준 확장 3종(`SetCellText`/`GetCellText`/`EvaluateFormula`)과 `GetActionSupport`:
     OCX 에 없는 API 다. **신규 층에 같은 이름으로 남긴다**(OCX 표면 + rhwp 확장 표면 분리 문서화).
   - `MovePos(2)` 결함: 기존 층은 "문서 끝"에서 커서를 `(0,0)` 으로 놓는다(`index.ts:365-371`).
     신규 층은 OCX 대조로 올바르게 구현되므로 **동작이 바뀐다** — 이관 노트에 명시.
3. **교체** — `hwpctl-test.html` 의 import 를 신규 패키지로 돌리고, e2e 를 재작성한다.
4. **삭제** — `rhwp-studio/src/hwpctl/` 12 파일 제거. 같은 PR 에서 지운다(호환 shim 을 남기지 않는다).
5. **문서 갱신** — `mydocs/tech/` 및 studio 문서에서 기존 층을 가리키는 서술을 신규 층으로 옮긴다.

#### 6.2.3 완료 판정

- `grep -rn "src/hwpctl" rhwp-studio` **0건**
- 재작성한 e2e 가 기존 e2e 의 **모든 단언을 포함**하고 녹색
  (등록 액션 수 ≥ 30 → **≥ 312**, `GetActionSupport` 3상태, ParameterSet 왕복)
- 하니스 전 시나리오 0 diff 유지(철거 PR 이 회귀를 만들지 않았음)
- studio 수동 시험: 문서 열기 → 편집 → 저장 정상

#### 6.2.4 되돌리기

철거 PR 은 **단일 커밋 revert 로 복구 가능**해야 한다. 삭제와 이관을 한 커밋에 담고, 그 커밋
메시지에 §6.2.1 실측 결과를 남긴다.

---

## 7. 리스크

| 리스크 | 완화 |
|---|---|
| ParameterSet 아이템이 수천 개 — 전수 실측이 병목 | Set 단위 우선순위(CharShape/ParaShape/SecDef/Table 먼저), 나머지는 지연 실측 |
| OCX 버전별 동작 차이 | 기준선을 **한글2022 로 고정**하고 원장에 버전 스탬프. 다른 버전은 별도 축 |
| 동기 API 안에서 URL 로딩 | OCX 원래 `callback` 인자를 쓰고, 동기 경로는 사전 프리로드 계약으로 |
| 신규 층이 studio 를 회귀시킴 | 코어 공유는 P6 이후. 그 전까지는 **복제 허용**(중복 > 회귀) |
| "100%" 가 판정 불가로 흐려짐 | §1.1 대체 계약표에 없는 예외를 만들지 않는다. 새 예외는 계획 개정으로만 |
| 하니스가 오판(과거 캠페인 재발) | §4.3 함정 7종을 하니스 v1 에 내장, P0 완료 판정에 **알려진 diff 재현**을 넣음 |
| P7 철거가 studio 동작을 바꿈(`MovePos(2)` 교정, undo 위임 등) | 계약 차이를 §6.2.2-2 에서 **전수 목록화한 뒤** 철거. 철거 PR 은 단일 커밋 revert 로 복구 가능 |
| 병존 기간이 길어져 두 층에 같은 결함을 두 번 고침 | 기존 층은 **동결**(버그 수정도 하지 않음). 급한 결함은 신규 층에서만 고치고 P7 을 당김 |

---

## 8. 검증 게이트 (PR 마다)

1. 하니스 차등: 대상 축 시나리오 **0 diff**
2. `cargo fmt --check` + `clippy -D warnings` (WASM 변경 시)
3. `cargo nextest` (debug) — 편집 계약 회귀 검출
4. studio e2e (`hwpctl-basic.test.mjs` 포함) 녹색
5. 10k 모집단 쪽수·시각 게이트 회귀 0 (WASM 렌더/조판 건드린 PR 한정)

---

## 9. 결정 사항

1. ~~배포 형태~~ — **독립 npm 패키지 `@rhwp/hwpctrl`** 로 확정(2026-08-07). 골격은
   `npm/hwpctrl-ocx/`.
2. **이슈 발행 단위** — Phase 당 이슈 1개 + 축당 PR 을 가정.
3. **P7 시점의 undo 계약** — 신규 층이 규격대로 `Undo`/`Redo` 를 지원하면 studio 히스토리
   (`engine/history.ts`)와 이중 관리가 된다. (A) 신규 층 undo 를 studio 히스토리에 위임 /
   (B) studio 가 신규 층 히스토리를 쓰도록 역전. §6.2.2-2 에서 결정하되, **(A)** 를 권장한다 —
   studio UI 는 이미 히스토리 대화상자·IndexedDB 저장(`history/idb-store.ts`)까지 붙어 있다.
4. ~~오라클 버전~~ — **한글2022 로 통일 완료**(2026-08-07). 전환·재수집까지 끝났고 게이트가
   기본값으로 강제한다(§4.5.1).
5. **`PutFieldText` 결함(§4.6) 처리 순서** — P1 착수 전에 별도 이슈·PR 로 고칠지, P1 안에서
   함께 고칠지. 경로가 WASM 공용이라 **studio 도 같이 낫는다**.
4. **P7 시점의 undo 계약** — 신규 층이 OCX 처럼 `Undo`/`Redo` 를 지원하게 되면 studio 히스토리
   (`engine/history.ts`)와 이중 관리가 된다. (A) 신규 층 undo 를 studio 히스토리에 위임 /
   (B) studio 가 신규 층 히스토리를 쓰도록 역전. §6.2.2-2 에서 결정하되, **(A)** 를 권장한다 —
   studio UI 는 이미 히스토리 대화상자·IndexedDB 저장(`history/idb-store.ts`)까지 붙어 있다.
