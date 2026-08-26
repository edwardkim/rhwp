---
kind: guide
status: active
canonical: gym/tutorial/README.md
last_verified: 2026-08-18
---

# 20. 첫날 체크리스트

휴게실을 한 바퀴 닫는 한 장이다. 항목은 기존 명령과 기존 과제다.
새 과제를 만들지 않는다.

돌아가기: [README.md](README.md) · 지도: [../PARK.md](../PARK.md)

## 빌드

- [ ] 저장소 루트에 있다
- [ ] `cargo build --bin rhwp` 가 성공했다 (Windows 는 `rhwp.exe`)
- [ ] `rhwp info samples/table-001.hwp --json` 이 JSON 을 냈다

## 입장 (`family`)

- [ ] `python gym/score.py --agent 나 --profile family`
- [ ] `gym/submissions/나/admission.json` 의 `verdict` 가 `allow`
- [ ] 프로파일 철자가 `family` 다 (`Family` 아님)

자세히: [01-admission.md](01-admission.md)

## 입문존 네 바퀴

- [ ] CR01 `pages` ← `info` 의 `pageCount` — [02-cr01-carousel.md](02-cr01-carousel.md)
- [ ] CR02 `paragraphs` ← `explain` 의 `paragraphCount` — [03-cr02-ferris.md](03-cr02-ferris.md)
- [ ] CR03 `tables` ← `export-tables` 의 `tableCount` — [04-cr03-circus.md](04-cr03-circus.md)
- [ ] CR04 `hits` ← `search -- 표` 의 `matchCount` — [05-cr04-ringtoss.md](05-cr04-ringtoss.md)
- [ ] 제출이 `gym/submissions/나/casual-rides/CR0n/answer.json` 이다
- [ ] 틀린 숫자를 한 번 넣어 떨어지는지 보고, 올바른 숫자로 되돌렸다

## 전당 (선택)

- [ ] `python gym/tools/leaderboard.py attest --agent 나`
- [ ] `python gym/tools/leaderboard.py verify`
- [ ] 비밀키를 커밋하지 않았다

[12-leaderboard.md](12-leaderboard.md)

## casual 바깥 한 걸음 (선택)

- [ ] 일곱 이름을 읽었다 — [06-profiles.md](06-profiles.md)
- [ ] `starter`: T01 쪽수 또는 SD01 명령 수 — [07-starter-path.md](07-starter-path.md)
- [ ] 다음 길이 숙제면 해당 페이지만 연다
      (`editor` / `publisher` / `operator` / `boss`)

## 친구 (선택)

- [ ] [../INVITE.md](../INVITE.md) 를 읽었다
- [ ] `invite --agent 친구이름` 으로 판 지문을 붙였다
- [ ] 부모님에게는 `--profile family` 만 권했다

[13-invite.md](13-invite.md)

## 하지 않은 일로 체크하지 말 것

- `gym/core/checks.py` 를 고쳤다 — 하면 안 된다
- 다른 PR 의 pack 과제 JSON 을 고쳤다 — 하면 안 된다
- 기준 풀이를 베껴 만점을 만들었다 — 타지 않은 것이다
- 원본 `samples/` 를 편집했다 — 되돌린다

정직 조항: [15-scoring-honesty.md](15-scoring-honesty.md)

## 기여자라면 (방문 다음)

이슈 #5263 같은 문서 작업의 로컬 확인은 이렇다.

```bash
python -m unittest scripts.tests.test_gym_tutorial
python gym/tools/audit.py
```

`audit.py` 는 pack 정합이다. 휴게실 문서가 pack JSON 을 안 만지면
devel 과 같은 결과가 나와야 한다. `cargo fmt --all` 은 Rust 가 없을
때 돌리지 않는다.
