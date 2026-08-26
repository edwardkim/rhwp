# 14 — 예외: venv 없음 (F09)

시스템 Python 으로 하네스를 치면 보통 이렇게 끝난다.

```
pypdf가 필요합니다: python -m pip install pypdf
```

시트 모드면 `pypdfium2가 필요합니다`. 종료 코드 2.

메시지 그대로 시스템에서 `python -m pip install pypdf` 를 실행하지
않는다. `--break-system-packages` 도 붙이지 않는다 (F15).

## 처방

POSIX:

```bash
test -x venv/bin/python || python3.12 -m venv venv
venv/bin/python -m pip install pypdf pypdfium2 pillow
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 0 \
  --text-only --out-dir /tmp/rhwp-fidelity-smoke
```

Windows:

```powershell
if (-not (Test-Path venv\Scripts\python.exe)) { py -3.12 -m venv venv }
venv\Scripts\python.exe -m pip install pypdf pypdfium2 pillow
venv\Scripts\python.exe tools\fidelity_compare\fidelity_compare.py plan 0 0 `
  --text-only --out-dir $env:TEMP\rhwp-fidelity-smoke
```

`venv` 디렉터리가 있는데 패키지가 없으면 **같은 venv 에 install**
한다. 새 venv 를 홈에 만들지 않는다. 재현이 저장소 루트 `venv/` 에
묶인다.

## 다른 실패와 구분

| 증상 | 정지 | 이 장? |
| --- | --- | --- |
| ImportError pypdf/pypdfium2 | F09 | 예 |
| `venv/bin/python` 없음 (Windows) | F09 + 03장 | 예 |
| Chrome 없음 | F10 | 아니오 |
| rhwp 바이너리 없음 | F12 계열 | 아니오. `RHWP_BIN` |
| 끝 쪽 overflow | exit 2, 범위 | 아니오 |

`rhwp 실행 파일을 찾을 수 없습니다` 는 venv 가 아니다.
`cargo build --profile release-test` 또는 `RHWP_BIN` 이다.

## PEP 668

일부 배포판은 시스템 pip 를 막는다. 그게 venv 를 쓰라는 신호다.
`--break-system-packages` 는 그 막을 부수는 일이라 이 저장소에서
거절한다. 에이전트가 "이게 제일 빠르다"고 제안하면 F15 로 끊는다.

## 에이전트 점검표

- [ ] 호출 argv 의 0번이 `venv/bin/python` 또는 `venv\Scripts\python.exe` 인가
- [ ] 그 바이너리의 `sys.prefix` 가 저장소 `venv` 인가
- [ ] freeze 에 pypdf 가 있는가
- [ ] 실패 로그를 "도구가 깨졌다"가 아니라 "환경" 으로 보고했는가
