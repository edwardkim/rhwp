# 재생성과 신선도

## 명령

```bash
cargo build --bin rhwp
python tools/gen_agent_codex.py          # 재생성 (표본 재실행)
python tools/gen_agent_codex.py --check  # 신선도 — 차이면 exit 3
```

`--check` 의 exit 3 은 C1 과 같은 **DATA** 다. 문서가 바이너리와 어긋났다는
판정이다. CI 가 이 코드를 실패로 읽도록 설계됐다.

## 누가 무엇을 판정하나

| 장치 | 질문 | 실패 |
|---|---|---|
| `gen_agent_codex.py --check` | 생성 장 본문이 지금 바이너리·픽스처와 같은가 | exit 3 |
| `tests/agent_codex_contract.rs` | 자기서술의 모든 명령이 `### \`이름\`` 장을 갖는가 | 테스트 실패 |
| 스킬 표류 가드 | 스킬이 죽은 명령을 가리키는가 | 테스트 실패 |
| 이 스킬 계약 시험 | 4규약·트리·경계·85 금지가 문서에 닫혀 있는가 | 테스트 실패 |

## 새 명령이 생기면

1. CLI 구현 (이 스킬의 범위 밖)
2. `cargo build --bin rhwp`
3. `python tools/gen_agent_codex.py`
4. 90_미분류.md 가 생기면 생성기 `FAMILIES` 표를 갱신하고 재생성
5. 생성 장을 손으로 채워 넣지 말 것

## 표본 문서가 바뀌면

생성 장의 JSON 이 바뀌는 것이 정상이다. 대전은 스냅샷이 아니라 현재형이다.
스킬 픽스처 전사본은 `gen_skill_pack.py` 로 다시 뽑는다.

## 하지 말 것

- 생성 장의 JSON 을 예쁘게 손보기
- `--check` 를 무시하고 날짜만 고치기
- 이 스킬에서 생성기를 포크해 다른 절단 규칙을 만들기
- 새 CLI 로 신선도를 대체하기
