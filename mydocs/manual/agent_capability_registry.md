---
kind: canonical
status: active
canonical: mydocs/manual/agent_capability_registry.md
last_verified: 2026-07-26
---

# 에이전트 capability 카탈로그

이 문서는 Claude와 Codex에서 재사용하는 **프로젝트 capability의 유일한 등록부**다.
파일 하나가 아니라 사용자에게 제공하는 책임 단위로 등록한다. 따라서 같은 capability의 Claude
에이전트와 Codex Skill은 별도 항목이 아니라 하나의 항목에 연결한다.

## 식별번호 생성 규칙

새 capability의 등록 식별번호는 **`CAP-<GitHub Issue 번호>`**다. 예를 들어 Issue #3398에서
등록하는 `bug-hunter`는 `CAP-3398`이다. 별도의 `CAP-001` 순번을 만들지 않는다. 동시 PR의
작성자가 다음 번호를 추측하면 충돌하므로, GitHub가 원자적으로 발급하는 **Issue** 번호만 숫자
키로 쓴다. PR 번호는 Issue와 번호 공간을 공유해도 PR 자체가 작업 계약이 아니므로 사용하지 않는다.

1. 새 capability에는 먼저 전용 GitHub Issue를 만들고, GitHub가 부여한 번호로 `CAP-<N>`을
   생성한다. 사람이 번호를 임의 지정하거나 재사용하지 않는다.
2. runtime 파일명과 표의 `capability ID`는 별도의 소문자 kebab-case 슬러그다. 예를 들어
   등록 식별번호 `CAP-3398`, capability ID `bug-hunter`, Codex Skill 이름 `bug-hunter`는 각각
   안정 식별·사람용 이름·런타임 이름의 역할을 가진다.
3. `deprecated`가 된 `CAP-<N>`도 삭제·재사용하지 않는다. 대체 capability의 새 Issue에서 새
   `CAP-<N>`을 발급하고, 이전 항목에 대체 ID를 남긴다.
4. 이 등록부보다 먼저 존재해 전용 Issue를 확정할 수 없는 항목만
   `LEGACY-<최초 도입 commit의 9자리 SHA>`를 쓴다. 이는 이관 한정 형식이며 새 항목에는 쓸 수 없다.
   전용 등록 Issue를 나중에 만들더라도 legacy 식별번호는 바꾸지 않고 그 Issue를 근거로 추가한다.

## 등록부

| 등록 식별번호 | capability ID | 책임과 비범위 | 권위 문서 | Claude 진입점 | Codex 진입점 | 상태·소유 |
| --- | --- | --- | --- | --- | --- | --- |
| `LEGACY-d86c935bc` | `rhwp-cli` | HWP/HWPX 분석·내보내기·진단. 구현 변경·한컴 최종 판정은 책임 밖 | [CLI 명령어 매뉴얼](cli_commands.md) | [Skill](../../.claude/skills/rhwp-cli/SKILL.md) | — | active · rhwp maintainers |
| `CAP-660` | `rhwp-exam-ingest` | 시험지 자료를 HWPX로 변환. 일반 문서 양식 생성은 책임 밖 | [ingest 명령](cli_commands.md#build-from-ingest) | [Skill](../../.claude/skills/rhwp-exam-ingest/SKILL.md) | — | active · rhwp maintainers |
| `CAP-4561` | `rhwp-contributor` | 기여 1건의 공식 절차 안내(이슈→분석→구현→검증 게이트→증빙→PR). 리뷰·머지 판단은 책임 밖 | [CONTRIBUTING](../../CONTRIBUTING.md) | [Skill](../../.claude/skills/rhwp-contributor/SKILL.md) | — | active · rhwp maintainers |
| `CAP-3398` | `bug-hunter` | 실사례 여정과 정답지 대조로 재현 가능한 결함을 발굴. 수정 구현은 요청 시 별도 작업 | [버그 헌팅 playbook](bug_hunting_playbook.md) | [에이전트](../../.claude/agents/bug-hunter.md) | [Skill](../../.agents/skills/bug-hunter/SKILL.md) | active · rhwp maintainers |

`—`는 해당 런타임용 어댑터가 아직 없다는 뜻이며, capability 자체가 없다는 뜻은 아니다.

## 등록·변경 규칙

새 Claude 에이전트·Claude Skill·Codex Skill을 만들거나 없애기 전에 이 등록부와 열린 Issue/PR을
확인한다. 이름이 아니라 **사용자 산출물, 권위 문서, 비범위**가 겹치는지를 기준으로 판단한다.

1. 같은 산출물과 같은 권위 문서를 쓰면 새 capability를 만들지 않는다. 기존 capability ID에 해당 런타임의
   어댑터 경로만 추가한다.
2. 기존 산출물의 범위만 넓히면 기존 ID와 권위 문서를 갱신한다. 독립된 산출물·판정 기준·책임이
   생겼을 때만 새 ID를 등록한다.
3. 새 capability에는 `CAP-<N>`, capability ID, 책임, 명시적 비범위, 권위 문서, 상태, 소유 maintainer를
   반드시 정한다. 상세 절차와
   트리거 문구는 여기서 복제하지 않고 authority와 각 진입점에 둔다.
4. 진입점을 추가·이동·제거하거나 capability를 폐기하면 **같은 PR에서** 이 표를 갱신한다.
   폐기 항목은 지우지 않고 `deprecated` 상태와 대체 capability를 남긴다.
5. capability의 책임·권위 문서·진입점 변경은 rhwp maintainer가 중복 여부와 이 표의 정확성을
   검토한다. 구현과 무관한 개인 프롬프트·일회성 조사 지침은 등록 대상이 아니다.

## 검증

PR 준비 시 변경한 등록 행의 authority와 진입점이 실제 파일을 가리키는지 확인한다. 문서 경로를
옮기거나 이 등록부의 구조를 바꾼 경우에는 다음 검사를 수행한다.

```bash
python3 scripts/check_markdown_links.py --changed-from upstream/devel
python3 scripts/check_document_metadata.py
```

등록부 검사기 자체를 수정했다면 형식·Markdown 표 파싱 회귀 테스트도 실행한다.

```bash
python3 -m unittest discover -s scripts/tests -p 'test_*.py'
```

Codex Skill을 추가·수정했다면 Skill Creator의 `quick_validate.py`도 실행한다. 활성 Python이
externally managed 환경이라 전역 또는 `--user` 설치를 거부하면, 검증 의존성은 사용자 홈 아래 Codex
전용 가상환경에 유지한다.

macOS/Linux POSIX 셸:

```bash
# 최초 1회: 가상환경 생성과 PyYAML 설치
python3 -m venv "$HOME/.codex/venvs/skill-creator"
"$HOME/.codex/venvs/skill-creator/bin/pip" install PyYAML

# Skill마다 실행
"$HOME/.codex/venvs/skill-creator/bin/python" \
  "$HOME/.codex/skills/.system/skill-creator/scripts/quick_validate.py" \
  .agents/skills/<skill-name>
```

Windows PowerShell:

```powershell
# 최초 1회: 가상환경 생성과 PyYAML 설치
py -m venv "$HOME\.codex\venvs\skill-creator"
& "$HOME\.codex\venvs\skill-creator\Scripts\pip.exe" install PyYAML

# Skill마다 실행
& "$HOME\.codex\venvs\skill-creator\Scripts\python.exe" `
  "$HOME\.codex\skills\.system\skill-creator\scripts\quick_validate.py" `
  .agents/skills/<skill-name>
```

가상환경은 저장소 밖의 로컬 도구 의존성이므로 Git에 추가하지 않는다. 다른 환경에서는 해당 Codex
설치 경로 아래에 같은 용도의 가상환경을 만들고, 저장소의 Skill 정의에는 환경별 절대 경로를 기록하지 않는다.

`scripts/check_markdown_links.py`는 등록부의 ID 형식·중복, active 항목의 authority와 runtime 진입점의
실파일 해석, 서로 다른 capability가 같은 runtime 진입점을 가리키는 경우를 자동으로 검출한다. authority
문서는 여러 capability가 공유할 수 있으므로 중복 오류로 다루지 않는다. 산출물·권위·비범위가 실질적으로
중복되는지는 기계적으로 판정하지 않고 maintainer 리뷰로 결정한다.
