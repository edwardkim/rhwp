# #4586 Stage 3 완료보고서 — 기준선·생성 교본·문서 정합

- **Issue**: [#4586](https://github.com/edwardkim/rhwp/issues/4586)
- **브랜치**: `task/4586-gym-t12-hwpx`
- **Stage 2 커밋**: `1ac75c0aad1056195c34cac2cc036d2943c5f99d`

## 1. T12 focused 기준 실행

전체 1호 선수의 gitignore 산출물이 저장소에 없으므로 전체 14과제를 다시 실행했다고 가장하지 않았다.
`samples/field-01.hwp`를 `export-hwpx`로 실제 HWPX로 만들고, 수정된 `gym.score.score_task`로 T12만
focused 재실행했다.

```text
T12 변환 자기검증: pass, 2/2 checks
- HWPX 형식 확인: expected hwpx / actual hwpx
- 변환물 IR 대조: expected false / actual false
```

기준 실행 식별자:

| 항목 | 값 |
| --- | --- |
| rhwp version | `rhwp v0.8.2` |
| rhwp source commit | `1ac75c0aad1056195c34cac2cc036d2943c5f99d` |
| capabilities SHA-256 | `62aea3df8bc40dd679247c044093e41fc54d1d80396c2b4b5b445ec843ffe27c` |
| source SHA-256 | `518cb939079e6e0640a5f813597f744e2528a17ca52ee418929f1c8f4b5380c0` |
| HWPX SHA-256 | `41c07ba1a1f00b356b0ca6ccbef986747ae177cfee10f243a6905d82ad698617` |

`gym/baselines/claude-fable-5/T12/verification.json`에 위 식별자와 실제 판정 결과를 고정했다.
`answer.json`, T12 scorecard slice와 report 행도 `identical:false`·형식 검사 계약으로 보정했다.
바이너리 산출물은 `gym/.gitignore` 정책대로 커밋하지 않는다.

로컬 원문은 `output/4586/baseline/README.md`에 있다.

## 2. 생성 교본 정합

`tools/gen_agent_codex.py`의 `convert` 표본을 `conv.hwpx`에서 `conv.hwp`로 바꾸고 설명도 HWP5 변환으로
명시했다. 생성된 `40_변환과_렌더.md`는 실제 `format:hwp5` 봉투와 `.hwp` 경로가 일치한다.

재생성 중 `50_검증_사다리.md`의 `planSha256`이 checkout 절대경로에 따라 달라지는 기존 결함을 발견했다.
생성기가 "어느 기계에서도 같은 본문"을 약속하므로 다음처럼 함께 보정했다.

- 계획 output을 절대 `{tmp}` 치환 대신 저장소 상대 `target/codex-tmp/plan_out.hwp`로 고정
- 표시 단계에서 상대 경로도 `<tmp>`로 정규화
- 새 상대 plan의 SHA-256을 검증 사다리에 재생성

연속 `python3 tools/gen_agent_codex.py --check`에서 변경 0을 확인했다.

## 3. 사용자 문서 정합

- `agent_codex/01_판단트리.md`: HWP→HWPX는 `export-hwpx`, HWPX/배포용→HWP5는 `convert`로 분리
- `mydocs/manual/cli_commands.md`: `.hwp` 외 출력은 입력 IO 전 exit 2, 산출 없음, `export-hwpx` 안내
- `gym/README.md`: 단일 `expect_exit`과 복수 판정 `expect_exits` 의미 구분
- 잘못된 `convert ... conv.hwpx`와 `HWP↔HWPX → convert` 문구 검색 결과 0건

## 4. 검증

```text
python3 -m unittest scripts/tests/test_gym_score.py
  5 passed / 0 failed

python3 tools/gen_agent_codex.py --check
  명령 83 · 실측 18 · 계약만 65 · 변경 0

cargo test --test agent_codex_contract
  2 passed / 0 failed

cargo clippy --all-targets -- -D warnings
  통과, 경고 0

cargo fmt --all -- --check
git diff --check
  통과
```

렌더러·레이아웃·WASM 변경이 없어 시각 증적과 WASM 빌드는 적용하지 않는다.

## 5. 다음 게이트

Rust CLI 변경이므로 `mydocs/manual/pr_review/local_validation.md` §4.3에 따라 release-test 전체 회귀를
남겼다. 장시간 전체 회귀와 PR CI는 별도 작업지시자 승인을 받은 뒤 수행한다. 전체 회귀 뒤 최종 보고서와
PR 준비 자료를 작성한다.
