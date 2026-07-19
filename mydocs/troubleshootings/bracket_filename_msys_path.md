---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# 대괄호 파일명 "읽기 실패" — Git Bash/MSYS 경로 변환 quirk (rhwp 버그 아님)

## 증상

Git Bash 에서 대괄호(`[...]`) 포함 파일을 `/c/...` Unix 경로로 rhwp CLI 에 넘기면 실패:

```
$ ./target/release/rhwp.exe dump '/c/Users/.../36383351_..._[관악산] ... 보고.hwpx' -s 0 -p 0
오류: 파일을 읽을 수 없습니다 - /c/Users/.../[관악산] ...: 지정된 경로를 찾을 수 없습니다. (os error 3)
```

## 원인 — MSYS 경로 변환 스킵 (rhwp 무관)

Git Bash(MSYS)는 Windows .exe 에 인자를 넘길 때 `/c/Users/...` → `C:\Users\...` 자동 변환한다.
그러나 **경로에 `[...]` 가 있으면 글롭 패턴으로 간주해 변환을 스킵**, 변환 안 된 `/c/...` 를
rhwp 에 그대로 전달한다. rhwp 는 `fs::read("/c/...")` 를 호출하나 Windows 에서 `/c/Users` 는
유효 경로가 아니라 실패(os error 3).

## 검증 (rhwp 는 대괄호 정상 처리)

| 경로 형식 | 대괄호 파일 | 결과 |
|-----------|-----------|------|
| `C:\Users\...[관악산]...` (PowerShell/cmd) | ✅ 정상 |
| `C:/Users/...[관악산]...` (슬래시 Windows) | ✅ 정상 |
| `/c/Users/...[관악산]...` (MSYS Unix) | ❌ 실패 |
| `/c/Users/...비대괄호...` (MSYS Unix) | ✅ 정상(MSYS 변환됨) |

**fidelity14 배치: 대괄호 파일 937건 전수 정상 처리**(935 PASS + 2 PARSE_FAIL=손상 다운로드).
→ rhwp 실사용(배치 rglob, PowerShell, Python Path)에 **영향 0**.

## 해결 (코드 수정 불요)

- **PowerShell/cmd 또는 Windows 경로(`C:\` / `C:/`)** 사용.
- Git Bash 에서 부득이 `/c/` 경로를 쓸 땐 `MSYS_NO_PATHCONV=1` 환경변수 + Windows 경로,
  또는 `cygpath -w` 로 변환.

## rhwp 코드 수정 부적절 사유

`/c/...` 를 드라이브 C: 로 해석하도록 rhwp 에 추가하면 **Linux/WASM 빌드에서 `/c/Users` 가
정상 절대경로인 것과 충돌**한다. `/c/...` 는 MSYS 관례일 뿐 표준 경로가 아니므로 Windows
프로그램이 파싱할 의무가 없다. rhwp 는 모든 표준 경로 형식을 대괄호 포함 정상 처리한다.
