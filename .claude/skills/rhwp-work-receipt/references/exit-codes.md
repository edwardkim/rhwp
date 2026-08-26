# 종료 코드 — 판정은 데이터다

권위: #2707, `cmd_replay` / `cmd_audit` / `cmd_lineage`,
`mydocs/manual/cli_commands.md` §종료 코드.

이 스킬이 배선하는 세 명령의 숫자는 같다. **exit 3 은 도구 크래시가 아니다.**

## 1. 표

| exit | 이름 | 언제 | stdout |
|-----:|------|------|--------|
| 0 | 성공 | attest 완료, verify 일치, audit 전건 재현, lineage valid | JSON 봉투 (`--json`) |
| 1 | IO | 계획/캡슐/폴더/부모 파일을 열 수 없음 | **0바이트** |
| 2 | 사용법 | 인자 부족, 미지 옵션, 빈 감사 폴더, 비hex 해시, 같은 파일 parent | **0바이트** |
| 3 | 판정 | verify 불일치, audit 실패 ≥1, lineage invalid | **봉투** (근거가 여기 있다) |

`run` 의 exit 2 가 `invalid[]` 봉투를 남기는 것과 다르다. `replay` /
`audit` / `lineage` 의 1·2 는 stdout 이 비어 있다. stderr 만 읽는다.

## 2. exit 3 을 읽는 법

| 명령 | 봉투 신호 | 다음 행동 |
|------|-----------|-----------|
| `replay` verify | `reproduced: false` + 두 해시 | 주장 기각. 같은 argv 재시도 금지 |
| `audit` | `failed[]` 비지 않음, `reproducedRate < 1` | 실패 캡슐만 개별 추적 |
| `lineage` | `valid: false`, `brokenAt` | 그 링크의 축을 읽는다 |

재시도가 도움이 되는 경우는 **exit 1** (경로·권한) 뿐이다.
exit 2 는 호출 조립을 고친다. exit 3 은 데이터를 사용자에게 보여 준다.

## 3. 명령별 사용법 표본

`replay`

- 계획 없음 → 사용법
- `--expect-output-sha256` 뒤에 값 없음 → 사용법
- 값이 64 hex 가 아님 → 사용법
- `--sign-key` 만 있고 `--capsule` 없음 → 사용법
- `--capsule` == `--parent` (기존 파일) → 사용법
- 계획 파일 읽기 실패 → **IO (1)**
- 부모 캡슐 읽기 실패 → **IO (1)**
- 계획 JSON 파싱 실패 / `input` 없음 → 사용법
- verify 해시 불일치 → **판정 (3)**

`audit`

- 폴더 인자 없음 → 사용법
- 폴더 열기 실패 → IO
- 직속 `*.capsule.json` 0개 → 사용법 (빈 폴더)
- 실패 ≥1 → 판정

`lineage`

- 머리 인자 없음 → 사용법
- 머리 파일 없음 → **IO (1)** (링크가 아직 없을 때)
- 중간 부모 없음 / JSON / 축 실패 → **판정 (3)**

## 4. 의사코드

```
code = spawn(rhwp, argv)
if code == 0:
    read_envelope(stdout)
elif code == 3:
    read_envelope(stdout)   # 판정. 예외로 raise 하지 않는다
    stop_or_show(brokenAt or failed or reproduced)
elif code == 2:
    assert stdout == b""
    fix_argv()              # 재시도 금지
elif code == 1:
    assert stdout == b""
    fix_path_or_permissions()
else:
    unexpected()
```

바인딩도 같다. exit 3 을 예외로 올리면 호출자가 `reproduced` /
`reproducedRate` / `brokenAt` 을 읽지 못하게 된다.

## 5. 이 스킬이 내지 않는 숫자

exit 4 는 `convert` / `export-hwpx --verify-pages` 전용이다.
`replay` / `audit` / `lineage` 기본 경로는 4 를 내지 않는다.
