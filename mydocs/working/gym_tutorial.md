---
kind: working
status: active
canonical: mydocs/working/gym_tutorial.md
last_verified: 2026-08-18
---

# gym 휴게실 · PARK 입문 문서 보강

Issue: #5263
Branch: `feat/gym-tutorial-park-docs`
Date: 2026-08-18

## 1. 결론

`gym/tutorial/` 을 5분 안내 한 장에서 입문 동선 세트로 늘렸다.
`gym/docs/tutorial.md` 가 프로파일 이름·상대 링크·채점 불가침을
규약으로 고정하고, `scripts/tests/test_gym_tutorial.py` 가 그 규약을
기계로 잠근다. `gym/PARK.md` 와 `gym/INVITE.md` 에는 첫 방문 동선·
일곱 프로파일·초대 손님별 첫 줄처럼 **실제로 온보딩에 쓰는 절**만
보탰다.

하지 않은 것 (이 기둥이 고치지 않는다):

- `gym/core/checks.py` 채점 논리
- 다른 열린 PR 의 pack 과제 JSON
- 새 pack · 새 과제 · 새 연산자
- `cargo fmt --all`

검증:

- `python -m unittest scripts.tests.test_gym_tutorial`
- `python gym/tools/audit.py`
- `cargo fmt --all` 은 실행하지 않음 (Python/문서만, 사용자 지시)

## 2. 배경

이슈 #5263:

> gym/tutorial 보강, 필요하면 casual 외 입문 문서와 시험. 채점 논리
> 변경 금지. 열린 pack PR 과 파일 충돌 금지.
> DoD: additions >= 3000. audit.py.

devel 의 휴게실은 `gym/tutorial/README.md` 93줄이었다. PARK 와
INVITE 는 테마파크 은유와 초대 3줄이 있었지만, casual 바깥으로
가는 동선·Windows 번역·프로파일 철자 계약·링크 가드가 없었다.

같은 날 열린 가지들이 pack 과제 JSON 을 크게 늘리고 있다
(casual-rides CR05+, extraction, batch-ops, text-editing 등).
그 파일에 손대면 충돌하고, 이슈가 명시한 금지를 깬다. 그래서
문서·시험만 만진다.

## 3. 한 일

### 3.1 휴게실 (`gym/tutorial/`)

| 파일 | 내용 |
|---|---|
| `README.md` | 5분 안내 유지, 01~20 색인, 일곱 프로파일 표 |
| `01-admission.md` | 입장 봉투, allow ≠ 만점 |
| `02`~`05` | CR01~CR04 손타기, 답 키와 오라클 경로 |
| `06-profiles.md` | family/starter/editor/publisher/operator/boss/maintainer |
| `07-starter-path.md` | T01 · T02 · SD01 (기존 과제만) |
| `08-editor-path.md` | TE01 · TB01 · OM01 |
| `09-publisher-path.md` | SR01 · LR01 · SE01 |
| `10-operator-path.md` | CD01 · AU01 |
| `11-boss-path.md` | XC01 과 보스 표 |
| `12-leaderboard.md` | attest / verify / render, 봉인 범위 |
| `13-invite.md` | 사람·에이전트·CI |
| `14-submissions.md` | pack 우선 제출 자리 |
| `15-scoring-honesty.md` | REGISTRY 열일곱, 불가침 |
| `16-unavailable.md` | 부재 ≠ 0점 |
| `17-faq.md` | 이름·제출·전당·환경 |
| `18-troubleshooting.md` | 막힘 10칸 |
| `19-windows.md` | PowerShell, UTF-8 without BOM |
| `20-checklist.md` | 첫날 한 장 |

모든 페이지가 `gym/tutorial/README.md` 를 canonical 로 가리킨다.
규약 정본은 `gym/docs/tutorial.md`.

### 3.2 PARK · INVITE

PARK 에 보탠 것:

- 휴게실이 한 장이 되었다는 포인터와 `docs/tutorial.md`
- 첫 방문 동선 다섯 걸음
- 일곱 프로파일 표
- 제출 자리 한 줄
- 존별 첫 놀이기구 (기존 1번 과제만)
- 막혔을 때 표
- 이 지도가 바꾸지 않는 것 (checks.py · pack JSON · verdict)

INVITE 에 보탠 것:

- 사람 / 에이전트 / CI 첫 줄
- 판 지문을 손으로 맞추는 순서
- 흔한 초대 실수 다섯
- 가족과 같이 탈 때 이름이 갈린다는 것
- 초대장이 바꾸지 않는 것

은유와 정직 조항의 원래 문장은 유지했다. 채점 의미를 뒤집는 문장은
넣지 않았다.

### 3.3 규약과 기록

- `gym/docs/tutorial.md` — 링크·프로파일·CR 키·REGISTRY 스냅샷·
  금지 변경
- `mydocs/working/gym_tutorial.md` — 이 기록

### 3.4 시험

`scripts/tests/test_gym_tutorial.py`

- 필수 파일이 있다
- 일곱 프로파일 JSON 의 id·packs 가 안내와 같다
- 상대 링크가 저장소 안에서 풀린다
- CR01~CR04 키와 명령이 휴게실에 있다
- `REGISTRY` 가 열일곱 이름 그대로다
- 안내가 checks.py 를 고친다고 말하지 않는다
- PARK/INVITE/휴게실이 서로를 가리킨다
- CI 워크플로가 이 시험을 호출한다
- 링크를 지운 텍스트는 검사 함수가 문제를 낸다 (음성 회귀)

### 3.5 CI

`.github/workflows/ci.yml` 의 "Validate gym scorer contracts" 에
`python3 -m unittest scripts/tests/test_gym_tutorial.py` 한 줄을
더했다. 다른 gym 도구 시험 파일을 고치지 않았다.

## 4. 의도적으로 하지 않은 일

- casual-rides 에 CR05 를 만들지 않았다. 그 자리의 열린 PR 과
  싸운다.
- `gym/README.md` 의 만점 표(12 pack · 221점)를 고치지 않았다.
  실제 pack 수는 그보다 많다. 숫자 갱신은 pack 확장 PR 의 일이다.
- `maintainer.json` 을 고치지 않았다. 이미 전 pack 을 가리킨다.
- `audit.py` 는 실행만 했고 수정하지 않았다.
- 시각 검증·Rust 테스트는 해당 파일이 없다.

## 5. 위험과 남은 구멍

1. **예시 숫자.** CR01 문서의 `{"pages": 3}` 은 설명용이라고
   여러 번 적었다. 그래도 붙여 넣는 방문자가 있을 수 있다. 라이브
   오라클이 거절한다. 안내가 골든을 만들지는 않는다.
2. **열린 pack PR.** 병합 후 입문존 과제 수가 4 가 아닐 수 있다.
   휴게실은 CR01~CR04 만 잠근다. CR05+ 는 그 PR 의 README 가
   맡는다.
3. **프로파일에 없는 pack.** `table-csv` 등은 `--pack` 으로만
   고른다. 이 기둥이 프로파일에 끼워 넣지 않았다.
4. **링크 검사 범위.** 시험은 상대 경로 파일 존재만 본다. 헤딩
   앵커와 외부 URL 은 보지 않는다. `check_markdown_links.py` 와
   같은 범위다.
5. **CI 한 줄.** 다른 가지가 같은 블록을 고치면 충돌할 수 있다.
   충돌 시 우리 줄(`test_gym_tutorial.py`)만 남기면 된다.

## 6. 재현

격리 worktree: `C:\Users\swsz9\rhwp-gym-tutorial-park-docs`
기준: `upstream/devel`
브랜치: `feat/gym-tutorial-park-docs`

```bash
python -m unittest scripts.tests.test_gym_tutorial
python gym/tools/audit.py
git diff --shortstat upstream/devel
```

## 7. 시험 목록 (클래스)

| 클래스 | 고정하는 것 |
|---|---|
| `RequiredFilesTests` | 규약이 적은 문서가 디스크에 있다 |
| `ProfileNameTests` | 일곱 id, 별명 금지, packs 매핑 |
| `TutorialLinkTests` | 상대 링크가 풀린다, 허브가 서로를 가리킨다 |
| `CasualRideContractTests` | CR01~CR04 명령·키·입력 |
| `ScoringUntouchedTests` | REGISTRY · GLOBAL_SCAN_OPS · checks.py 불가침 문장 |
| `HonestyClauseTests` | pack 별 점수, 라이브 오라클, unavailable |
| `AdmissionContractTests` | allow ≠ 만점, packsScored |
| `CiWiringTests` | ci.yml 이 이 시험을 호출한다 |
| `NegativeGuardTests` | 깨진 링크·빠진 프로파일은 반드시 걸린다 |

## 8. 크기 게이트

이슈 DoD: `additions >= 3000` vs `upstream/devel`.

허용에 가까운 경로:

- `gym/tutorial/**`
- `gym/docs/tutorial.md`
- `mydocs/working/gym_tutorial.md`
- `gym/PARK.md`
- `gym/INVITE.md`
- `scripts/tests/test_gym_tutorial.py`
- `.github/workflows/ci.yml` (한 줄)

`git add -A` 금지. 위 경로만 스테이징한다. 워킹트리에 다른 열린
PR 의 pack JSON 이 나타나도 커밋에 넣지 않는다.

## 9. 설계에서 버린 대안

1. **casual-rides 에 읽기 과제 추가.** 이슈가 문서·시험을 말하고
   열린 pack PR 과의 충돌을 금지한다. 과제는 그 PR 에 맡긴다.
2. **프로파일 JSON 에 table-csv 등을 추가.** maintainer 가 이미
   전 pack 을 가리킨다. family/starter 를 넓히면 입문 의미가
   흐려진다.
3. **checks.py 에 주석만 추가.** 이슈가 채점 논리 변경 금지를
   걸었다. 주석도 그 파일의 책임 범위라 안 만진다.
4. **gym/README.md 만점 표 갱신.** pack 확장 PR 과 숫자가 싸운다.
5. **영문 휴게실.** 저장소 입문 표면은 한글이다. 번역은 후속.

## 10. 로컬 명령 기록

격리 worktree 에서 실행한 것:

```
python -m unittest scripts.tests.test_gym_tutorial
python gym/tools/audit.py
```

`cargo fmt --all` 과 `cargo fmt --all -- --check` 는 돌리지 않았다.
Rust 파일이 없다. 사용자 지시가 명시적으로 금지했다.

`audit.py` 는 devel 의 pack 정합을 그대로 통과해야 한다. pack JSON
을 이 가지가 만지지 않으므로 audit 실패는 회귀가 아니라 작업 트리
오염이다.

PR 본문은 한글, base 는 `devel`, `closes #5263`, `--body-file` 로
보낸다.

닫는 문장: 휴게실이 두꺼워져도 채점은 얇다. 안내가 연산자를 늘리지
않는다.

이상.

(이 기록은 작업 메모다. 규약 정본은 `gym/docs/tutorial.md`.)
