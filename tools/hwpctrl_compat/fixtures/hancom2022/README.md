# Hancom 2022 Oracle Fixture

이 디렉터리는 Windows 한글 2022 COM(major `12`)에서 정상 수집하고 검토한 WebHwpCtrl 차등
정답지를 보관하는 위치다. fixture에는 시나리오별 `<id>.returns.json`과, `saveAs` 시나리오가
만든 HWP 산출물을 함께 넣는다.

macOS·Linux에서는 다음 명령으로 이 fixture를 **읽기 전용** 대조할 수 있다.

```bash
node tools/hwpctrl_compat/python_runner.mjs run_gate.py \
  --impl npm/hwpctrl-ocx/src/index.mjs --fixture
```

fixture 수집·갱신은 Windows에서 한글 2022 COM 버전이 `12`인지 확인한 뒤에만 한다. Windows
live 실행의 기본 출력 `output/poc/hwpctrl/ocx/`을 바로 Git에 추가하지 않는다. 결과의 버전,
시나리오 수, 저장 문서까지 검토한 뒤 이 디렉터리로 선별 복사하고 차등 gate를 다시 통과시킨다.

현재 이 파일만 있는 것은 검증된 fixture가 아직 수집되지 않았기 때문이다. 빈 fixture에서
`--fixture`를 실행하면 `NO_ORACLE`로 실패해야 하며, 임의 값으로 통과시키면 안 된다.
