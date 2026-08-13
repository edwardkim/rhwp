# 처리 결과 — core-cli 기준 풀이 완결 (#4689)

## 배경

PR #4656 통합 검토 기록([pr_4656_review.md](../pr/archives/pr_4656_review.md))이 남긴 **유일한 추적 항목**:

> `core-cli`의 14개 legacy 과제는 현재 추적된 `reference/`와 과거 baseline 제출물이 완전하지 않아 … 과거 scorecard의 전체 수치가 현재 저장소만으로 완전히 재현된다고 주장하지 않는다.

다른 11개 pack 은 전부 `reference/` 기준 풀이가 있어 저장소만으로 "풀 수 있음"이 실측된다. core-cli 만 예외였다.

## 무엇을 했나

core-cli 14과제(T01~T14) 전건에 `reference/` 기준 풀이를 저작 — **저장소 단독으로 scorecard 재현 가능**.

| 유형 | 과제 | 저작 방식 |
|---|---|---|
| answer 6 | T01·T02·T03·T04·T05·T11 | 봉투 값 라이브 재계산(`answer` 스텝, 개수는 `len`) |
| 편집 산출 5 | T06·T07·T08·T09·T10 | `run`/`edit` 산출(치환·서식채움·셀교정·2단계·결정론쌍) |
| 변환 1 | T12 | `export-hwpx` + ir-diff identical 라이브 기록 |
| 사다리 2 | T13·T14 | 하네스 작업장(2캡슐)·서명+앵커 게이트 통과 |

## T13 깨진 명령 수리 (부수 발견)

devel 의 T13 체크가 **존재하지 않는 `harness status`(두 단어)** 를 불렀다 — v0.8.4 에서
`harness` 는 `init|wrap` 하위명령만 있고, 판정 명령은 `harness-status`(하이픈, capabilities 실재)다.
그래서 T13 은 채점되지 못하고 있었다(이것이 core-cli 13/14 의 실패 1건).

- `gym/packs/core-cli/tasks/T13.json`: 체크 `["harness","status"]` → `["harness-status"]`.
- `test_gym_score.py` 의 명령 실재성 가드가 `harness-status` 를 **이름으로 금지**하고 있었다 — 이는
  옛 명령 표면 기준이었다. 하드코딩 금지 대신 **실제 capabilities 와 대조**하도록 고쳤다(바이너리가
  단일 출처; 없으면 이름 꼴 검사까지만 하고 skip).

## 검증

```
core-cli 기준 풀이 왕복:  성공 14 · 실패 0
core-cli 채점:            32/32 (14/14 과제)
전 12 pack 왕복:          성공 100 · 실패 0
전 pack 채점:            221/221 (12 pack)
gym 계약 테스트:          62 passed, 1 skipped
```

- `test_gym_packs.py`: core-cli 면제 제거 — 12 pack 전체가 같은 완결성 기준(reference 필수).
- `git diff --check`: 통과. 루트 산출물 오염 없음(plan output 을 `{sub:}` 로).

## 트랩 (정직 기록)

1. `harness status`(두 단어)는 v0.8.4 에 없다 — `harness-status`(하이픈)가 정식. 옛 가드가 이를 거꾸로 금지하고 있었다.
2. `convert` 는 `.hwp` 전용(`-o` 없음, 위치 인자) — HWPX 는 `export-hwpx <입력> <출력.hwpx>`.
3. `fill-fields` 는 `edit fill-fields --data <JSON|@파일>`(필드명→값 맵). 첫 필드명은 콘솔에서 깨지므로 생성기가 실측으로 읽어 박음.
4. 다세대 plan 의 `output` 이 상대명(`o.hwp`)이면 저장소 루트로 흘러나온다 — `{sub:}` 로 제출 폴더에 가둠.
5. 크로스플랫폼: 메인테이너가 추가한 `test_built_submission_is_scored_in_its_pack_directory` 가 `os.path.join` 결과를 리터럴 `"/"` 와 비교해 **Windows 에서만** 깨졌다(내 변경 전 devel 에서도 실패 실측). 기대값을 같은 `os.path.join` 으로 고침.

## 시각 증거

`mydocs/report/edit_demo_4689/t08_before_after.png` — T08 기준 풀이가 실제로 issue2007 문서의 표 (0,0) 을 '짐검증'으로 교정한 전/후(rhwp export-svg 렌더, 자가 검증).

## 정직

기준 풀이는 정답 노출이므로 `reference/` 로 분리(에이전트는 보지 않는 규칙). 채점·판정 로직은 불변 — 완결성만 채웠다. legacy `gym/baselines/` 는 길잡이로만 참고했다.
