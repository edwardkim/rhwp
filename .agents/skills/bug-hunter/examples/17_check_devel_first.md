# 예제 — 상신 전 devel 확인

이슈 #5324. playbook 함정 2. gym 아님. F14.

## 정답지

현재 `upstream/devel` 의 파일:라인. 이슈 본문이나 CLOSED PR
제목이 아니다.

## 명령

```bash
git fetch upstream devel
git grep -n "<원인 함수>" $(git rev-parse upstream/devel)
# Read 로 해당 줄이 패치 후 상태인지 확인
```

## 읽는 법

53건 중 17건이 이미 고쳐져 있던 전례. CLOSED PR 이 cherry-pick
되어 반영된 사례 #2927/#2943 → #3205. 커밋 메시지 검색만으로
"아직 있다"고 쓰지 않는다.
