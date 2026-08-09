---
kind: report
status: active
canonical: mydocs/report/task_m100_3914_queue_preflight.md
last_verified: 2026-08-06
---

# 처리 결과 — 선검사 큐 규율 검사 3종 (#3914 수용 기준 2)

병렬 세션 규약(`mydocs/tech/autonomous_maintenance/parallel_session_protocol.md`)
§8-6 이 "제안이고 구현되지 않았다"로 남겨 둔 큐 검사를 `tools/agent_preflight.py`
에 구현했다.

## 분석 — 스펙은 이미 있었다

§8-6 이 형태를 규정한다: ① 기존 `Report.fail/ok/skip` 구조에 얹되 ② 실패 의미가
"계약 위반"이 아니라 "규율 이탈(판단 필요)"이므로 경고 전용 ③ 네트워크가 없으면
조용히 건너뛴다(헛울리는 검사기는 곧 무시당한다) ④ 검사 항목은 잔량·동일 이슈
중복·미할당 착수 셋.

구현 중 규약 문서와 달라진 판단 하나: §8-5 의 중복 감지 예시는 본문 전체의
`#N` 을 긁지만, 실제로는 조망 이슈(#3907 등) 참조가 거의 모든 PR 본문에 있어
헛울린다. **선언된 대상만**(제목의 `#N` + 본문의 `closes/fixes/resolves/refs #N`
행) 세도록 좁혔다 — 실사고 두 건(#3902/#3903 은 제목·본문에 #3885, #3897/#3904 는
#3884)은 이 좁힌 신호로도 전부 잡힌다.

## 변경

- `tools/agent_preflight.py`
  - `Report.warn()` 신설 — 경고 전용 채널, 종료 코드 무관.
  - `check_queue_discipline()` 신설 — gh 인증 확인 후 열린 PR 1회 조회로
    잔량(캡 10 내외)·동일 이슈 중복을 보고, 현재 브랜치가 `task/<n>-`·`wip/fix-<n>-`
    형태면 이슈 #n 의 잠금(assignee **또는 착수 코멘트** — 외부 기여자는 assignee
    편집이 거부되므로, §5-1 실측)을 확인한다. gh 부재·미인증·네트워크 실패는 전부
    `skip`.
  - `--no-network` 플래그, 자기 stdout UTF-8 고정(cp949 콘솔에서 보고 도중
    UnicodeEncodeError 로 죽던 경로 — 경고 출력이 이 경로를 늘려서 이 PR 범위),
    ReDoS 검사의 자기스캔 제외(이 파일이 변경 집합에 들면 검출기 리터럴을 잡는
    오탐 3건 실측).
- `mydocs/manual/agent_preflight_guide.md` — "큐 규율 검사" 절 신설.

## 실측 (재현: 저장소 루트에서 실행)

① 정상 경로 — `py tools/agent_preflight.py --static-only` (브랜치 `task/3914-…`):

```
  통과   doc 주석 오배치
  통과   ReDoS — 백트래킹 폭발 정규식
  통과   큐 규율 (열린 PR 6건; #3914 잠금=착수 코멘트)
  건너뜀 오염 — 커밋 범위 밖 파일 — staged 파일 없음 — `git add` 뒤에 다시 돌려라
  건너뜀 rustfmt — 변경된 .rs 없음

전부 통과.
exit=0
```

② 미할당 착수 경고 — 브랜치를 `task/4056-…` 로 바꿔 실행(이슈 #4056 은
assignee·착수 코멘트가 없다):

```
경고 1건 — 차단 아님, 판단 필요 (종료 코드 무관)
  ! [큐 규율] 브랜치가 이슈 #4056 를 가리키는데 assignee 도 착수 코멘트도 없다 —
    선점하라: gh issue comment 4056 --repo edwardkim/rhwp --body "착수합니다 — <범위>"
    (외부 기여자는 assignee 편집이 거부된다, protocol §5-1 실측)
```

③ `--no-network` — 큐 규율 검사가 출력에 아예 나타나지 않음(0줄) 확인.

경고는 세 실행 모두 종료 코드에 반영되지 않았다(①·③ exit 0; ② 는 경고만으로
exit 0 유지).

## 남긴 판단

- 캡 상수는 `QUEUE_CAP = 10`(§8-1 의 세 출처가 말하는 값). 숫자 변경은 처리량
  실측이 선행해야 한다는 §8-1 의 단서를 그대로 승계한다.
- 이슈 코멘트의 착수 판별은 `"착수"` 포함 여부다 — 형식이 굳으면
  (`"착수합니다 — <범위>"`) 더 좁힐 수 있다.
