# 25 — 신선도 --check

갈래: **유지보수**. 장: `README.md`. gym 아님. 새 CLI 아님.

권위는 생성 장이 아니라, 생성 장을 **읽는 순서**다. 표본 JSON 을 손대지 않는다.

## 요청

`대전이 낡은 것 같아.`

## 명령

```bash
cargo build --bin rhwp
python tools/gen_agent_codex.py --check
```

차이면 **exit 3**. 그것은 DATA(C1) 다. 생성 장을 손으로 맞추지 말고
`python tools/gen_agent_codex.py` 로 재생성한다.
