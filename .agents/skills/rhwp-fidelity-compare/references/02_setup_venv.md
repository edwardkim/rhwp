# 02 — venv 와 pypdf / pypdfium2 / pillow

하네스는 저장소 로컬 `venv/` 의 Python 으로만 돌린다. 시스템 패키지
설치와 `--break-system-packages` 는 금지다 (F15).

정본: `tools/fidelity_compare/README.md` 요구사항 절,
개발 환경 가이드의 venv Git 제외 계약.

## 왜 venv 인가

CI 와 기여자 머신에 시스템 Python 이 이미 jinja/ansible/ydoc 으로
묶여 있다. 하네스가 `pypdfium2` 를 전역에 깔면 그 환경이 깨진다.
저장소 `venv/` 는 `.gitignore` 되어 있고, 재현 기록에
`venv/bin/python -m pip freeze` 를 남길 수 있다.

에이전트가 "빠르게 돌리려고" `pip install pypdf` 를 시스템에서 실행하면
F09/F15 위반이다. 메시지에 적힌 `python -m pip install pypdf` 는
**어떤 인터프리터인지** 를 생략한 힌트일 뿐, 시스템 설치 허가가가 아니다.

## 최초 1회 (POSIX)

```bash
python3.12 -m venv venv
venv/bin/python -m pip install -U pip
venv/bin/python -m pip install pypdf pypdfium2 pillow
venv/bin/python -c "import pypdf, pypdfium2, PIL; print('ok', pypdf.__version__)"
```

`--text-only` 만 쓸 계획이면 `pypdf` 만으로 충분하다. 그래도 pillow 와
pypdfium2 를 같이 넣어 두면 시트 모드로 올라갈 때 ImportError 가 없다.

Python 3.11 도 동작하는 편이지만 정본 예시는 3.12 다. 3.13 에서
wheel 이 없으면 소스 빌드로 넘어가지 말고 3.12 venv 를 다시 만든다.

## 최초 1회 (Windows)

```powershell
py -3.12 -m venv venv
venv\Scripts\python.exe -m pip install -U pip
venv\Scripts\python.exe -m pip install pypdf pypdfium2 pillow
venv\Scripts\python.exe -c "import pypdf, pypdfium2, PIL; print('ok')"
```

`python` 이 Microsoft Store 별칭이거나 3.11 이면 `py -3.12` 를 고정한다.
실행 정책이 막으면 `venv\Scripts\python.exe -m pip` 만 쓰고
`Activate.ps1` 은 생략해도 된다. 활성화는 필수 가 아니다. **경로를 명시한
venv 파이썬** 이 필수다.

## 호출

```bash
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 2 --text-only \
  --out-dir /tmp/rhwp-fidelity-smoke
```

`python3 tools/fidelity_compare/...` 로 시스템 인터프리터를 찌르지 않는다.
빠져 있으면 stderr:

```
pypdf가 필요합니다: python -m pip install pypdf
```

또는 시트 모드에서:

```
pypdfium2가 필요합니다: python -m pip install pypdfium2
```

종료 코드 2. F09 — 저장소 venv 를 고친다.

## 의존성 역할

| 패키지 | 모드 | 역할 |
| --- | --- | --- |
| pypdf | `--text-only` | `PdfReader` 로 텍스트층, `len(pages)` |
| pypdfium2 | 시트 모드 | PDF 래스터 + `get_text_range` |
| pillow | 시트 모드 | 비교 PNG 합성, diff% |

Chrome 은 pip 패키지가 아니다. 13장. rhwp 바이너리도 pip 가 아니다.
`find_rhwp()` 가 `target/release-test` → `target/release` → PATH →
`RHWP_BIN` 순이다.

## 재현 기록에 남길 것

```
python: venv/bin/python  (3.12.x)
pypdf: <version>
pypdfium2: <version>
pillow: <version>
rhwp: target/pr-review/release-test/rhwp  (git SHA)
```

`pip freeze` 전체를 붙일 필요는 없다. 세 패키지와 인터프리터 경로면
충분하다.

## 에이전트 점검표

- [ ] `venv/bin/python` 또는 `venv\Scripts\python.exe` 가 존재하는가
- [ ] 그 바이너리로 `import pypdf` 가 되는가
- [ ] 시트 모드면 `pypdfium2` 와 `PIL` 도 되는가
- [ ] `pip install --break-system-packages` 를 제안하지 않았는가
- [ ] `sudo apt install python3-pypdf` 같은 시스템 패키지로 우회하지 않았는가

실패 시 [14_missing_venv.md](14_missing_venv.md).
Windows 세션이면 다음 장.
