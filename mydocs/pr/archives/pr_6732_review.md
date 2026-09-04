# PR #6732 검토 기록

## 최종 판정: 승인

## 대상과 기준

- 원 PR: [#6732](https://github.com/edwardkim/rhwp/pull/6732) (`planet6897`)
- 원본 head: `37d3dc888a6d819f2ec381a09a7e2c4268ebaa98`
- 통합 기준: `upstream/devel` `d1831146587b1ac2346f9ed1216a64c2943a02f9`
- 통합 반영: `-x` cherry-pick `53d7c9864`
- 사전 reviewer: `jangster77` 지정 완료

## 변경과 검토 결과

- saved tail이 body 끝 좌석에 실제로 걸린 경우만 fit 재배치를 막도록 `typeset` 조건을 좁힌다.
  이로써 일반적인 saved-tail overflow가 불필요하게 다음 페이지로 넘어가는 회귀를 방지한다.
- 원 PR의 source head CI, CodeQL, Render Diff, Adapter inter-diff, Proptest, CI Impact Policy는
  모두 성공했다. 실제 run은 [CI](https://github.com/edwardkim/rhwp/actions/runs/33884058000),
  [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/33884057975),
  [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/33884057691),
  [Adapter](https://github.com/edwardkim/rhwp/actions/runs/33884057974),
  [Proptest](https://github.com/edwardkim/rhwp/actions/runs/33884059342),
  [Policy](https://github.com/edwardkim/rhwp/actions/runs/33884819135)다.
- 공개 canonical fixture `samples/issue5941/1490000-201600081_roadmap_research.hwp`
  (`a06f46ec3f175c7cfa84eb3178b8b3fbdf78e94f71b31d7d87f3417a2617dae9`)에서
  현재 devel native binary는 305 logical pages, 통합 후보 native binary는 304 logical pages를
  보고했다. 이는 PR의 regression expectation과 일치한다.
- 통합 candidate에서 formatter와 all-target clippy가 성공했고, focused
  `issue_5941_tail_overflow_drift_gate` regression도 통과했다.

## 증적 범위

- 후보 native-skia export의 마지막 logical page 304가 비어 있지 않고 표의 마지막 행까지 나타남을
  직접 확인했다. PNG는 [여기](../assets/pr_6702_6732_planet6897_integration_20260904/visual-6732/candidate-p304/1490000-201600081_roadmap_research.png)에
  있으며 SHA-256은 `5d5ebc292cb8d6af044469b7fd61b5a953e7f05f50f1669c43f16b5be65ee62d`다.
- PDF oracle 또는 pixel overlay를 이번 통합 후보에서 다시 수행하지 않았으므로, 이 증적은 마지막
  페이지의 비공백·육안상 잘림 부재와 page-count 변화에 한정한다. source PR 본문의 Hancom count를
  독립 재현한 결과로 기록하지 않는다.
- 상세 범위와 한계는 [통합 시각 증적](pr_6702_6732_planet6897_visual_sweep.md)에 남겼다.

## 통합 전 조건

- 이 판정은 source 변경을 그대로 적용한 통합 후보에 대한 것이다. 통합 review/evidence/오늘할일
  trailing head의 required CI 성공과 작업지시자의 PR 생성 승인이 필요하다.
- 원 PR comment/close와 contributor branch 정리는 통합 merge 및 실제 `devel` CI 성공 뒤
  `post_merge.md` 절차에서 한 번만 처리한다.
