---
kind: guide
status: active
canonical: gym/tutorial/README.md
last_verified: 2026-08-18
---

# 17. 자주 묻는 것

돌아가기: [README.md](README.md) · 막힘: [18-troubleshooting.md](18-troubleshooting.md)

## 입장과 프로파일

**Q. 프로파일 이름에 대문자를 써도 되나?**
안 된다. 파일 이름이 `family.json` 이다. `Family` 나 `FAMILY` 는
다른 경로다.

**Q. `casual` 프로파일이 있나?**
없다. 입문존을 고르는 이름은 `family` 다. pack 을 직접 고르려면
`--pack casual-rides`.

**Q. `beginner` / `expert` / `guest` 는?**
프로파일 id 가 아니다. 일곱 이름은
`family` · `starter` · `editor` · `publisher` · `operator` ·
`boss` · `maintainer` 뿐이다. [06-profiles.md](06-profiles.md).

**Q. 만점이어야 들어가나?**
아니다. pack 을 하나라도 유효하게 채점하면 `verdict: allow` 다.
[01-admission.md](01-admission.md).

## 제출과 채점

**Q. 기준 풀이를 봐도 되나?**
봐도 채점은 정직하게 돈다. 측정되는 능력만 달라진다.
[15-scoring-honesty.md](15-scoring-honesty.md).

**Q. 예시의 `{"pages": 3}` 을 그대로 내면?**
네 `rhwp info` 가 3 을 줄 때만 통과한다. 예시 숫자는 설명용이다.
라이브 오라클이 정답이다.

**Q. CR01 과 T01 이 둘 다 쪽수면 답을 재사용하나?**
입력 문서가 다르다. 숫자도 다를 수 있다. 폴더도 `casual-rides/CR01`
과 `core-cli/T01` 로 갈린다.

**Q. 원본 샘플을 고치면?**
하지 마라. 제출은 `gym/submissions/` 아래 새 파일이다. 커밋된
픽스처를 바꾸면 다른 과제와 시각 검증까지 깨진다.

**Q. 이 안내가 채점 점수를 바꾸나?**
바꾸지 않는다. `gym/core/checks.py` 는 이 작업의 범위 밖이다.

## 리더보드와 초대

**Q. 초대장이 있어야 등재되나?**
아니다. 문은 열려 있다. 초대장은 안내와 판 지문이다.
[13-invite.md](13-invite.md) · [../INVITE.md](../INVITE.md).

**Q. 비밀키를 PR 에 올려야 하나?**
안 된다. `gym/leaderboard/keys/` 는 `.gitignore` 다. 공개키·서명·
스코어카드만 오른다.

**Q. 같은 점수를 두 번 등재하면?**
원장이 거부한다. 전역 유일성(P3).

## 환경

**Q. 오프라인에서도 되나?**
된다. 채점과 봉인은 로컬이다.

**Q. Windows 인데 문서의 bash 가 안 된다.**
[19-windows.md](19-windows.md).

**Q. 파이썬 버전이?**
3.8+ 이면 채점기·감사기가 돈다. 표준 라이브러리만 쓴다.

**Q. `audit.py` 는 방문자가 돌리나?**
기여자가 pack 을 손댈 때 필수다. 입문 방문자는 안 돌려도 탄다.
이 작업(#5263)은 pack JSON 을 안 만지므로, 감사기는 회귀 확인용이다.
