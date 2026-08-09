# @rhwp/hwpctrl - 웹한글컨트롤 호환 층

한컴 웹한글컨트롤(WebHwpCtrl) API v2.4를 rhwp WASM 위에서 호출 호환으로 구현한다.
현재는 P1~P3 개발자 미리보기이며, 원장 기준 193/484 항목이 한글 2022 COM Oracle과 대조됐다.
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

문서 I/O(`Open`, `SaveAs`), 필드 읽기·쓰기·이름 변경, 커서·선택 이동, 문서 속성,
글자·문단 모양, 블록과 `Run` Action을 지원한다. 정확한 지원·보류·대체 항목은 원장에서 확인한다.
`Version`과 `IsModified`는 COM 값이 아니라 웹 호환 계약에 따른 `substituted` 항목이다.

## 기존 studio 층과의 관계

`rhwp-studio/src/hwpctl/`은 별개이고 P6까지 동결한다. 이 패키지가 원장 100%에 도달하면
P7에서 그 층을 철거하고 studio를 이쪽으로 이관한다(계획서 §6.2).

## 개발과 검증

공통 개발 절차는
[`웹한글컨트롤 호환 개발 가이드`](../../mydocs/manual/webhwpctrl_compat_development.md)를 따른다.
Oracle 하니스의 Windows 전용 준비와 실행 규칙은
[`tools/hwpctrl_compat/README.md`](../../tools/hwpctrl_compat/README.md)에 있다.

```bash
# 공개 패키지 진입점과 생성자 계약만 빠르게 검사한다.
npm --prefix npm/hwpctrl-ocx run test:contract

# Windows에서는 Hancom 2022 COM Oracle, macOS/Linux에서는 WASM 자체 시나리오를 검사한다.
npm --prefix npm/hwpctrl-ocx run gate
```

`gate`는 새 패키지 구현(`npm/hwpctrl-ocx/src/index.mjs`)을 대상으로 실행한다. 기존 studio
구현(`legacy`)은 하니스 자체 검증 전용이며 패키지의 통과 근거가 아니다. macOS/Linux의 기본
결과는 호출·저장 회귀 검증이다. Windows Hancom 2022로 수집·검토한 fixture가 있으면 모든 OS에서
정적 Oracle 대조를 수행할 수 있으며, 정확한 명령과 fixture 갱신 권한은
[`tools/hwpctrl_compat/README.md`](../../tools/hwpctrl_compat/README.md)를 따른다.

## 파일 역할

| 파일 | 역할 |
| --- | --- |
| `src/index.mjs` | ESM 공개 진입점: `createHwpCtrl`, `HwpCtrl`, `ParameterSet` |
| `test/package_contract.test.mjs` | 패키지 self-reference와 생성자 계약 검사 |
| `spec/webhwpctrl_api.json` | API 122항목 (속성 18, 메서드 67, 이벤트 3, 객체 34) |
| `spec/actions.json` | Action 312개와 ParameterSet |
| `spec/parameter_sets.json` | ParameterSet 50종과 Item 521개 |
| `spec/api_ledger.json` | 원장 484항목과 Oracle 근거 |

`spec/`는 손으로 고치지 않는다. 재생성 절차는
[`tools/hwpctrl_compat/README.md`](../../tools/hwpctrl_compat/README.md)를 따른다.
