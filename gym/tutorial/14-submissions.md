---
kind: guide
status: active
canonical: gym/tutorial/README.md
last_verified: 2026-08-18
---

# 14. 제출 폴더 — 어디에 무엇을 놓나

채점기는 파일을 찾는다. 폴더 이름이 틀리면 그 과제는 "제출 없음"으로
떨어진다. 이 페이지는 `gym/core/runner.py` 가 이미 하는 탐색 순서를
방문어로 옮긴다. 탐색 순서를 바꾸지 않는다.

돌아가기: [README.md](README.md)

## 기본 자리

```text
gym/submissions/<에이전트이름>/
├── scorecard.json
├── report.md
├── admission.json
├── casual-rides/
│   ├── CR01/answer.json
│   ├── CR02/answer.json
│   ├── CR03/answer.json
│   └── CR04/answer.json
├── core-cli/
│   └── T01/answer.json
└── text-editing/
    └── TE01/edited.hwp
```

`<에이전트이름>` 은 `--agent` 와 같다. 채점 명령과 제출 폴더가
어긋나면 빈 점수가 나온다.

## pack 우선, 평면은 하위 호환

`score_pack` 은 먼저 `submissions/<이름>/<packid>/` 를 본다. 그
폴더가 없으면 `submissions/<이름>/` 을 과제 id 로 직접 훑는다.
옛날 배치를 깨지 않으려는 하위 호환이다. 새로 탈 때는 pack 아래가
맞다. 입문존과 본식 존 과제 id 가 한 폴더에 섞이지 않는다.

## 제출 종류

`gym/README.md` 제출 형식 절과 같다.

| `submit.kind` | 놓는 것 | 입문 예 |
|---|---|---|
| `answer` | `answer.json` (과제가 요구한 키만) | CR01 `pages` |
| `artifact` | 지정된 파일 이름 | TE01 `edited.hwp` |
| `pair` | 산출 2개 + 계획서 | core-cli T10 |

`answer` 에 여분 키를 넣어도 보통 채점은 필요한 키만 본다. 그래도
과제가 요구한 키만 넣는 편이 실수가 적다. `artifact` 는 파일 이름이
과제 `submit.files` 와 같아야 한다. `edited.hwp` 를 `out.hwp` 로
내면 그 과제는 파일을 못 찾는다.

## 원본을 제출 자리에 복사하지 마라

`samples/` 는 입력이다. 제출이 아니다. 편집 과제는 `-o` 로 새
파일을 만든다. `differs_from_input` 이 있는 과제는 입력 바이트와
같은 산출을 거절한다.

산출 `.hwp`/`.hwpx` 는 보통 커밋하지 않는다. 재실행으로 다시 만들
수 있어야 한다는 것이 이 저장소의 검증 문화다.

## 채점 산출물은 제출이 아니다

`scorecard.json` · `report.md` · `admission.json` 은 채점기가 쓴다.
이 세 파일을 손으로 고쳐 점수를 올리는 것은 전당 사슬이 막는다.
로컬 파일을 고치는 것 자체는 가능하지만, `attest` 이후 `verify` 가
해시 불일치를 폭로한다.

## Windows 경로

PowerShell 에서 `나` 같은 한글 에이전트 이름은 된다. 콘솔 코드
페이지가 CP949 면 `echo` 로 쓴 JSON 이 깨질 수 있다.
[19-windows.md](19-windows.md) 의 UTF-8 without BOM 을 쓴다.

슬래시와 백슬래시는 Python 쪽 `os.path.join` 이 맞춘다. 제출 폴더를
문서에 쓸 때는 저장소 관례대로 `/` 를 쓴다.
