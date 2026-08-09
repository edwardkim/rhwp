---
kind: guide
status: active
canonical: mydocs/manual/webhwpctrl_compat_development.md
last_verified: 2026-08-09
---

# 웹한글컨트롤 호환 개발 가이드

`@rhwp/hwpctrl`은 한컴 웹한글컨트롤(WebHwpCtrl) API v2.4를 rhwp WASM으로 대체하려는
개발자용 호환 층이다. 이 문서는 패키지 연결, 코드 수정, Oracle 대조의 경계를 정한다.
호환 범위의 유일한 기준은
[`npm/hwpctrl-ocx/spec/api_ledger.json`](../../npm/hwpctrl-ocx/spec/api_ledger.json)이다.

## 현재 배포 계약

패키지는 P1~P3 개발자 미리보기이며 아직 `private` 상태다. npm 레지스트리 배포물이나 전역
`HwpCtrl`을 자동으로 만드는 단독 스크립트는 제공하지 않는다. 앱은 ESM으로
`createHwpCtrl({ wasm, onSave, version })`를 가져오고, rhwp WASM 초기화가 끝난 뒤 호출한다.

실제 연결 코드와 `Open`/`SaveAs` 예시는
[`npm/hwpctrl-ocx/README.md`](../../npm/hwpctrl-ocx/README.md)를 따른다. 기존 페이지가
전역 `HwpCtrl` 이름을 사용하면 앱 bootstrap에서 생성한 객체를
`globalThis.HwpCtrl`에 명시적으로 연결한다. 자동 전역·기존 `<script>` 교체가 구현된 것으로
가정하면 안 된다.

## 공통 준비

저장소 루트에서 일반 rhwp 개발 도구를 준비한다.

```bash
wasm-pack --version
node --version
npm --version
python3 --version
```

Rust/WASM과 Studio 준비는 [개발 환경 가이드](dev_environment_guide.md)를 따른다. Rust 또는
WASM 경계를 바꿨다면 다음 순서로 새 WASM을 만든다.

```bash
wasm-pack build --target web --out-dir pkg
npm --prefix npm/hwpctrl-ocx run test:contract
```

`test:contract`는 `package.json`의 공개 `exports` 경로와 생성자 계약만 검사한다. 문서·API
목록이 맞아도 한글 호환성이 증명되는 것은 아니다.

## OS별 시나리오 검증과 Windows Oracle

`npm --prefix npm/hwpctrl-ocx run gate`는 세 OS에서 실행할 수 있다. macOS·Linux에서는 rhwp WASM에
등록된 모든 시나리오를 실행해 호출 순서·호출 오류·`SaveAs` 산출물을 확인한다. 이는 플랫폼 공통의
자체 회귀 검증이다.

행동 호환성의 기준값을 **새로 수집하거나 갱신**하는 작업은 Windows의 한글 2022(major 12) COM
Oracle에서만 한다. macOS·Linux는 검토·고정된 fixture를 `--fixture` 또는 `--oracle-dir`로 읽어
차등 비교할 수 있지만, 그 환경에서 새 fixture를 만들거나 Oracle 통과 수치를 올리면 안 된다.
fixture 대조와 WASM 자체 검증의 결과는 `output/poc/hwpctrl/verdict/run_status.json`의 `oracleMode`
(`live`, `fixture`, `wasm-self-check`)로 구분한다.

Windows에서 필요한 조건은 다음과 같다.

- 한글 2022가 COM으로 등록되어 있고 major 버전이 `12`
- Python에서 `import win32com`이 성공하는 `pywin32`
- `wasm-pack`, Node.js, 저장소의 `rhwp-studio/node_modules/`
- 시작 전에 실행 중인 `Hwp.exe`/`HwpFrame.exe`가 없음

정확한 사전 점검, 프로세스 정리 금지 원칙, 정답지 갱신 규칙과 실행 명령은
[`tools/hwpctrl_compat/README.md`](../../tools/hwpctrl_compat/README.md)를 따른다.
기본 검증은 다음과 같다.

```bash
npm --prefix npm/hwpctrl-ocx run gate
```

Windows live 모드에서는 새 패키지 구현을 대상으로 시나리오를 **직렬** 실행한다. COM Oracle은
문서마다 별도 한글 프로세스를 쓰므로 임의 병렬 실행하거나 일반 개발 PC에서
`--cleanup-spawned`를 지정하면 안 된다. macOS·Linux의 WASM 자체 검증에는 COM 프로세스 제약이
없지만, fixture 대조의 비교 범위와 결과 표기는 Windows live 모드와 구분한다.

## 변경 후 갱신할 자료

새 API 또는 시나리오를 추가할 때는 다음 세 층을 함께 갱신한다.

1. 소비자 계약: `npm/hwpctrl-ocx/README.md`의 지원 상태·초기화·저장 예시
2. Oracle 운영: `tools/hwpctrl_compat/README.md`와 시나리오·원장 근거
3. 진행 계획: `mydocs/plans/hwpctrl_ocx_full_compat.md`의 현재 단계와 완료 수치

`spec/`와 Oracle 산출물은 사람이 추측으로 고치지 않는다. 공식 문서 추출, Oracle 재수집,
비교 결과 반영의 순서는 도구 README에 있는 절차만 사용한다.
