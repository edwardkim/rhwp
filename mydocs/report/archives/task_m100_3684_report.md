---
kind: report
status: active
canonical: mydocs/plans/task_m100_3684.md
last_verified: 2026-08-02
---

# Task #3684 최종 보고 — Actions cache quota 구조적 해법

- Issue: [#3684](https://github.com/edwardkim/rhwp/issues/3684) (M100) —
  [#2431](https://github.com/edwardkim/rhwp/issues/2431) 후속
- 2026-08-02 완결 / 단계 기록: `mydocs/working/task_m100_3684_stage{1,2}.md`

## 결과

| 지표 | 정리 전 | 정리 후 |
|---|---:|---:|
| 캐시 개수 | 42개 | **24개** |
| 총량 | 10.13GB (한도 **초과**) | **4.73GB** |
| 무료 한도(10GB) 대비 | 101% | **47%** |

매일 18:23 UTC cron 이 세대를 자동 정리하므로 재발하지 않는다.

## 조사 결론이 이슈 전제를 뒤집었다

이슈는 **"Blacksmith 등 캐시 계층 이전"** 을 전제로 과제 1(비용)·2(sticky disk)·
3(재분류) 순으로 열렸다. **과제 3 을 먼저 수행하자 근인이 드러나 1·2 가 불필요해졌다.**

**근인**: `Swatinem/rust-cache`·CodeQL·frontend-wasm 이 `Cargo.lock`/SHA 해시별로 새
캐시를 만들고, **구 세대가 GitHub 의 7일 미사용 만료까지 자리를 차지한다.**
`v0-rust-lint` 는 7세대가 동시 생존했고 각 세대는 하루 이틀만 쓰였다.

캐시 계층을 옮기는 문제가 아니라 **키 세대 관리 부재**였다.

### #2431 이 실패한 이유도 설명된다

| ref | 개수 |
|---|---:|
| `refs/heads/devel` | **31** |
| `refs/pull/*/merge` | 7 |

#2431 의 A안은 **PR-ref 정리**를 겨눴는데 실제 축적은 devel 이 압도적이었다. A·B 를
다 해도 46% 를 차지하는 세대 누적은 그대로 남는다. 정리 자동화 자체가 틀린 게 아니라
**겨눈 대상이 근인이 아니었다.**

## 후보 검증 — 3개 중 2개 기각

- **`save-if` 축소 → 기각**: 해시당 캐시 수 = 그 해시로 돌아간 job 수(중복 저장 없음).
  devel push 가 7일간 498회였지만 세대는 7개뿐이고 `Cargo.lock` 커밋 9회와 정합.
  **좁혀도 줄어들 것이 없다.**
- **CodeQL·wasm 별도 처리 → 불요**: 같은 세대 누적 패턴이라 전역 정리로 흡수.
- **세대 상한 → 채택.**

## 구현

`.github/workflows/cache-generation-sweep.yml` — 그룹별(키에서 후행 해시 제거 + ref)
최신 **2세대** 유지, 매일 cron + 수동 dispatch(`dry_run` 기본 true).

**KEEP=2 근거**: 1세대만 남기면 2.53GB 로 더 줄지만 진행 중 job 이 방금 만든 캐시를
스윕이 지울 수 있고, lock 변경 직후 PR 이 전부 cold 가 된다.

**안전 경계**: `actions: write` 권한 하나, checkout·PR 코드 실행 없음, **열린 PR ref
캐시 제외**(실행 시 24건 보호 확인), 삭제 실패는 경고 처리, 정리 전/후 총량을 job
summary 로 기록.

## 검증 — 3단계가 모두 일치

| 단계 | 결과 |
|---|---|
| 로컬 시뮬레이션 (실제 캐시 스냅샷) | 18개 5.40GB 정리 → 잔여 4.73GB 예측 |
| dry-run (실제 워크플로) | **42개 10.13GB / 정리 대상 18개 5.40GB** — 예측과 일치 |
| 실행 | **18개 전부 삭제, 실패 0건** → **4.73GB** |

## Blacksmith·sticky disk — 불요 판정

이 해법으로 한도의 47% 로 내려가므로 유료 서비스 도입도, 2026-07-25 self-hosted 종료
판정 재검토도 불필요하다. 이슈의 과제 1·2 는 이 해법이 실패했을 때의 대안으로 남긴다.

### openclaw 대조에서 배운 것

openclaw 는 **GitHub 캐시 정리 워크플로가 아예 없다**(`sticky-disk-cleanup.yml` 은
Blacksmith 디스크용). 정리가 필요 없는 구조이기 때문이다 — `rust-cache` 대신
`actions/cache` 로 키를 직접 관리하고(`...-build-all-v4-${hashFiles(...)}` 처럼 명시적
세대 번호), 무거운 것은 sticky disk 로 빼 GitHub 캐시에 쌓일 물량 자체가 적다.

**더 근본적인 대안**으로 rust-cache 를 버리고 키를 직접 관리하는 길이 있으나, cargo
캐시 경로·restore 전략을 직접 짜야 해 비용이 크다. 지금은 스윕으로 충분하다.

## 겪은 함정 2가지

1. **등록형 워크플로는 기본 브랜치 기준** — 신규 워크플로를 devel 에만 두니
   `workflow_dispatch` 가 HTTP 404. main 반영(#3812) 후 등록됐다. 계획서 §5 에
   "트리거 자체를 바꾸지 않으므로 위험이 낮다"고 적었으나, **신규 워크플로 파일 추가가
   곧 트리거 등록**이라는 점을 놓쳤다. `feedback_workflow_trigger_default_branch`
   메모리의 함정을 다른 형태로 다시 밟았다.
2. **`dry_run` 기본값이 버그를 잡았다** — 첫 실행에서
   `res.data.actions_caches is not iterable`(paginate 가 배열을 평탄화해 주는데 원시
   응답 형태를 가정) 로 실패. 곧바로 실삭제였다면 "일부 삭제 후 중단"이라는 애매한
   상태가 됐을 것이다. #3813(devel)·#3814(main) 로 정정.

## 관련 PR

| PR | 내용 |
|---|---|
| [#3810](https://github.com/edwardkim/rhwp/pull/3810) | 스윕 워크플로 신설 (devel) |
| [#3812](https://github.com/edwardkim/rhwp/pull/3812) | main 등록 — dispatch·schedule 발동 조건 |
| [#3813](https://github.com/edwardkim/rhwp/pull/3813) | 페이지네이션 정정 (devel) |
| [#3814](https://github.com/edwardkim/rhwp/pull/3814) | 페이지네이션 정정 (main) |
