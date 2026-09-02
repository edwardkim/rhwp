# @rhwp/hwpctrl - 웹한글컨트롤 호환 층

한컴 웹한글컨트롤(WebHwpCtrl) API v2.4를 rhwp WASM 위에서 호출 호환으로 구현한다.
현재는 P1~P5 개발자 미리보기이며, 원장 기준 **312/484** 항목이 올라 있다 — 308 은 한글 2022
COM Oracle과 0 diff 로 대조됐고, 넷(음·양력)은 **오라클이 판정자가 될 수 없어** 사유를 적고
계약을 바꾼 것이다(아래 참고). 남은 **172** 는 이 하니스로 구조적으로 못 재는 것들이라
(대화상자·안 끝남·관측창 없음·숨은 상태·글자 폭 정밀도 따위) **도달 가능한 상한 312 에
닿았다**.

음·양력 넷은 한국천문연구원이 펴낸 공공 자료로 구현했다. 그 표는 한글의 표와
**1841~2043 의 2,051일 표본 중 35일(1.71%)** 이 어긋난다 — 스물둘은 한글이 29일짜리 달의
"30일"을 말하는, 그 달력에 없는 날이고 넷은 2033년 윤달 배치가 갈린 자리다. 어긋난 목록은
양쪽 답과 함께 [`spec/lunar_divergence.json`](spec/lunar_divergence.json) 에 있다.

갈래와 그 이유는 `tools/hwpctrl_compat/classify_remaining.py` 가 낸다. 다만 그 분류는
**Windows 액션 스윕 산출물이 있어야** 재현된다 — "안 끝남"·"대화상자" 를 가리는 자료가
`output/` 아래에 있고 Git 에 담기지 않는다. 없는 기계에서 돌리면 그 항목들이 "아직 안 잼"
으로 새어 상한이 부풀고, 분류기는 그 수를 확정값이라 부르지 않고 비영 종료 코드로 알린다.
위 172·312 는 스윕이 있는 Windows 기계에서 센 값이다(계획서 §4.57).
지원 범위와 보류 항목의 권위 자료는 [`spec/api_ledger.json`](spec/api_ledger.json)이다.

이 패키지는 아직 `private` 상태다. npm 레지스트리에서 설치하거나 단독 `<script>` 파일을
내려받는 방식은 제공하지 않는다. 앱은 ESM으로 패키지를 가져오고, 먼저 rhwp WASM을 초기화해야 한다.

## 앱에 연결하기

소스 트리에서 시험할 때는 앱의 `package.json`이 있는 디렉터리에서 로컬 패키지를 연결한다.

```bash
npm install /path/to/rhwp/npm/hwpctrl-ocx
```

`@rhwp/core`의 WASM 초기화가 완료된 뒤 생성자를 호출한다. WASM 파일 배치 경로는 앱의 번들러와
배포 방식에 맞춰 바꾼다.

```js
import initRhwp, * as rhwpWasm from '@rhwp/core';
import { createHwpCtrl } from '@rhwp/hwpctrl';

await initRhwp({ module_or_path: '/assets/rhwp_bg.wasm' });

const HwpCtrl = createHwpCtrl({
  wasm: rhwpWasm,
  onSave(bytes, fileName) {
    const blob = new Blob([bytes], { type: 'application/x-hwp' });
    const url = URL.createObjectURL(blob);
    const link = Object.assign(document.createElement('a'), { href: url, download: fileName });
    link.click();
    URL.revokeObjectURL(url);
  },
});

globalThis.HwpCtrl = HwpCtrl;
```

`Open`은 브라우저의 `File` 또는 `Uint8Array`/`ArrayBuffer`를 받는다. `File`은 비동기이므로
성공 여부를 콜백에서 확인한다.

```js
HwpCtrl.Open(fileInput.files[0], '', '', (ok) => {
  if (!ok) return;
  HwpCtrl.PutFieldText('기안자', '홍길동');
  HwpCtrl.SaveAs('기안문.hwp', 'Hwp', '');
});
```

현재 패키지는 전역을 자동 생성하지 않는다. 기존 코드가 전역 객체 이름을 기대할 때만 위처럼
앱 bootstrap에서 명시적으로 연결한다.

## 현재 지원 범위

문서 I/O(`Open`, `SaveAs`, `SetTextFile`/`GetTextFile`), 필드 읽기·쓰기·이름 변경, 커서·선택
이동, 문서 훑기(`InitScan`/`GetText`), 문서 속성, 글자·문단 모양, 블록, 표 셀 이동·블록·편집,
개체 고르기·옮기기·크기 조절·묶음 풀기·캡션·글상자, 되돌리기, 파라미터셋과 `Run` Action을
지원한다. 정확한 지원·보류·대체 항목은 원장에서 확인한다. `Version`과 `IsModified`는 COM 값이
아니라 웹 호환 계약에 따른 `substituted` 항목이다.

`GetTextFile("TEXT", ...)`는 한글의 CP949 계약처럼 인코딩할 수 없는 문자를 `&#N;`으로
바꾼다. `GetTextFile("UNICODE", ...)`는 같은 문서 순서를 원문 Unicode로 돌려준다.
JavaScript 문자열을 받는 `SetTextFile`은 두 형식 모두 Unicode 입력을 보존한다. Windows live
Oracle 시나리오는 시스템 ANSI code page에 따라 `TEXT` 결과가 달라지지 않도록 `UNICODE`를 쓴다.

**호스트 고리.** 규격이 **경로**를 받는 API 는 호스트가 거들어야 한다 —
`createHwpCtrl({ wasm, onSave, onReadFile, onCreatePageImage })`. 고리가 없으면 그 API 는
아무 일도 하지 않는다. `CreatePageImage` 는 코어가 그린 쪽 SVG 를 호스트에 넘기고 픽셀로
앉히는 것은 호스트 몫이라, **파일 갈래와 픽셀은 이 층이 약속하지 않는다**.

**물려받는 한계.** 줄·쪽 단위 API(`MoveLine*`·`MovePage*`·`DeleteLine*`)는 저장된 줄 나눔과
rhwp 조판기에 기대므로 조판 정밀도를 그대로 물려받는다. 조판이 한글과 갈리는 문서에서는 이
값들도 갈린다.

## 두 실행 모드 — studio 없이도, studio 와 함께도

이 패키지는 **혼자서도 문서를 다룬다**. 브라우저·Node·워커 어디서든 위 예시 그대로 동작하고,
studio 를 참조하지 않는다(`test/independence.test.mjs` 가 그 성질을 잠근다).

rhwp-studio 안에서는 **플러그인**으로 얹힌다 — `@rhwp/hwpctrl/studio-plugin`.

| | standalone | plugin |
| --- | --- | --- |
| 진입점 | `createHwpCtrl({ wasm, ... })` | `hwpctrlStudioPlugin` (studio 가 `plugins.load('hwpctrl')`) |
| 문서 소유 | 이 층이 만든다 | studio 가 소유하고 **빌려준다**(복사 없음) |
| 화면 | 없음 — 호스트가 그린다 | studio 가 그린다 |
| undo | 자체 스냅샷 | studio 히스토리 (배치 1건 = undo 1스텝) |
| `Open`/`Clear` | 자기 문서 교체 | studio 열기 경로에 위임 |

두 모드는 **같은 구현을 공유**한다. 갈라 두면 원장 수치가 모드마다 달라지므로,
`test/adapter_parity.test.mjs` 가 같은 시나리오의 산출 바이트가 동일함을 검사한다.

플러그인 모드에서 `Undo`/`Redo` 는 studio 히스토리로 간다 — 사용자가 보는 undo 스택과 두 갈래가
되면 안 되기 때문이다.

## 기존 studio 층과의 관계

`rhwp-studio/src/hwpctl/`은 별개이고 아직 동결 상태다. 이 패키지가 원장 100%에 도달하면
그 층을 철거하고 studio 를 이쪽으로 이관한다(계획서 §6.2). 그 이관에 필요한 배선(플러그인 호스트·
트랜잭션·좌표 변환)은 이미 서 있다 —
[`rhwp_studio_hwpctrl_plugin.md`](../../mydocs/plans/rhwp_studio_hwpctrl_plugin.md).

## 개발과 검증

공통 개발 절차는
[`웹한글컨트롤 호환 개발 가이드`](../../mydocs/manual/webhwpctrl_compat_development.md)를 따른다.
Oracle 하니스의 Windows 전용 준비와 실행 규칙은
[`tools/hwpctrl_compat/README.md`](../../tools/hwpctrl_compat/README.md)에 있다.

```bash
# 공개 패키지 진입점과 TEXT/UNICODE dispatch 계약을 빠르게 검사한다.
npm --prefix npm/hwpctrl-ocx run test:contract

# 좌표 변환·모드 동등성·독립성 검사 (pkg/ WASM 이 있어야 한다)
cd npm/hwpctrl-ocx && node --test test/*.test.mjs

# Windows에서는 Hancom 2022 COM Oracle, macOS/Linux에서는 WASM 자체 시나리오를 검사한다.
npm --prefix npm/hwpctrl-ocx run gate
```

`gate`는 새 패키지 구현(`npm/hwpctrl-ocx/src/index.mjs`)을 대상으로 실행한다. 기존 studio
구현(`legacy`)은 하니스 자체 검증 전용이며 패키지의 통과 근거가 아니다. macOS/Linux의 기본
결과는 호출·저장 회귀 검증이다. Windows live gate는 `Quit()` 뒤 남는 **이번 실행의** Hancom
프로세스만 정리한다. 시작 시 이미 열려 있던 Hancom 프로세스는 종료하지 않고 `OCCUPIED`로
거부하므로, 전용 검증 계정에서 실행한다. Windows Hancom 2022로 수집·검토한 fixture가 있으면
모든 OS에서 정적 Oracle 대조를 수행할 수 있으며, 정확한 명령과 fixture 갱신 권한은
[`tools/hwpctrl_compat/README.md`](../../tools/hwpctrl_compat/README.md)를 따른다.

## 파일 역할

| 파일 | 역할 |
| --- | --- |
| `src/index.mjs` | ESM 공개 진입점: `createHwpCtrl`, `HwpCtrl`, `ParameterSet` |
| `src/studio-plugin.mjs` | rhwp-studio 플러그인 진입점 (`@rhwp/hwpctrl/studio-plugin`) |
| `src/adapter.mjs` | 모드 어댑터 — 읽기/쓰기 분류와 문서 교체 위임 |
| `src/cursor-map.mjs` | 한글 `{list,para,pos}` ↔ studio `{sectionIndex,cellPath}` 변환 |
| `test/package_contract.test.mjs` | 패키지 self-reference와 생성자 계약 검사 |
| `spec/webhwpctrl_api.json` | API 122항목 (속성 18, 메서드 67, 이벤트 3, 객체 34) |
| `spec/actions.json` | Action 312개와 ParameterSet |
| `spec/parameter_sets.json` | ParameterSet 50종과 Item 521개 |
| `spec/api_ledger.json` | 원장 484항목과 Oracle 근거 |

`spec/`는 손으로 고치지 않는다. 재생성 절차는
[`tools/hwpctrl_compat/README.md`](../../tools/hwpctrl_compat/README.md)를 따른다.
