# PR #6702 검토 기록

## 최종 판정: 메인터너 보정 됨 수용 가능

## 대상과 기준

- 원 PR: [#6702](https://github.com/edwardkim/rhwp/pull/6702) (`planet6897`)
- 원본 head: `a64d1e5fd83959279cff1c4328f421fa985856d3`
- 통합 기준: `upstream/devel` `d1831146587b1ac2346f9ed1216a64c2943a02f9`
- 통합 반영: 원 code/report `e984794`와 test/baseline/evidence `a64d1e`를 `-x`로 적용한
  `8b5f20349`, `32cc8113a`
- 사전 reviewer: `jangster77` 지정 완료

## 변경과 검토 결과

- 셀의 host 문단에 block table이 들어 있을 때 host text를 그리지 않아 표제
  `<향후 10년간 폐농업용 지게차 해체 수익 계산>`이 사라지던 renderer 경로를 보완한다.
- 원 PR의 source head CI, CodeQL, Render Diff, Adapter inter-diff, Proptest, CI Impact Policy는
  모두 성공했다. 실제 run은 [CI](https://github.com/edwardkim/rhwp/actions/runs/33880196302),
  [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/33880196257),
  [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/33880195987),
  [Adapter](https://github.com/edwardkim/rhwp/actions/runs/33880196106),
  [Proptest](https://github.com/edwardkim/rhwp/actions/runs/33880196099),
  [Policy](https://github.com/edwardkim/rhwp/actions/runs/33880192772)다.
- 다만 원 regression test는 `RHWP_ISSUE6697_SAMPLE` 또는 기여자 Windows 개인 corpus에서만
  문서를 찾아, 자료가 없으면 성공으로 반환했다. 공개 CI가 실제 회귀 조건을 실행하지 않는 결함이므로
  그대로는 수용하지 않았다.
- 메인터너 보정으로 repository 정식 fixture
  `samples/issue6697/80550-agricultural-machinery-act-amendment.hwpx`를 `include_bytes!`로
  필수화했다. private path와 환경 변수 fallback을 제거해 fixture 부재를 성공으로 숨기지 않는다.
  fixture SHA-256은 `e7b147f7cea66c97bed79085a3d89c2656037e0f711232f659ed3c7344984f62`다.
- 통합 candidate에서 formatter와 all-target clippy가 성공했고, 공개 fixture를 읽는 focused
  `issue_6697_cell_host_paragraph_text_is_drawn` regression도 통과했다.

## 증적 범위

- native-skia 후보 export의 논리 page 30에서 목표 표제가 표 아래에 표시됨을 직접 확인했다.
  후보 PNG는 [여기](../assets/pr_6702_6732_planet6897_integration_20260904/visual-6702/candidate-p30/80550-agricultural-machinery-act-amendment.png)에
  있으며 SHA-256은 `c4b0078b7c3cb4cdda10ac365689f04e648a11484e72394e68019e98a554ff64`다.
- 기여자 before/after/oracle PNG와 범위·한계는
  [통합 시각 증적](pr_6702_6732_planet6897_visual_sweep.md)에 고정했다.
- 이 확인은 표제의 존재에 한정한다. native export의 폰트 대체와 기존 `rhwp-studio` 차이가 있으므로
  전체 페이지 pixel equivalence나 Studio 동작의 대체 증거로 사용하지 않는다.

## 통합 전 조건

- 메인터너 보정과 review/evidence/오늘할일을 포함한 통합 head의 required CI가 성공해야 한다.
- 작업지시자의 통합 PR 생성 승인을 받은 뒤에만 upstream temporary branch push와 PR 생성을 한다.
- 원 PR comment/close와 contributor branch 정리는 통합 merge 및 실제 `devel` CI 성공 뒤
  `post_merge.md` 절차에서 한 번만 처리한다.
