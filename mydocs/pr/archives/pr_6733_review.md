# PR #6733 검토 기록

## 최종 판정: 수용

## 대상과 기준

- 원 PR: [#6733](https://github.com/edwardkim/rhwp/pull/6733) (`planet6897`)
- 원본 head: `13450dc20969ae7a386d4254c323a0ef2dbfca88`
- 통합 기준: `upstream/devel` `d1831146587b1ac2346f9ed1216a64c2943a02f9`
- 통합 반영: conflict 없는 `-x` cherry-pick `734528ca3`
- 사전 reviewer: `jangster77` 지정 완료

## 변경과 로컬 검증

- #6721에 일부만 반영되어 유실된 #6718의 `pi=62` sub-pixel ladder fit 보정을 복구한다.
  `LADDER_FIT_EPSILON_PX = 1.0`을 바깥 fit gate와 `ladder_page_is_full`에 동일 적용하고,
  10쪽 `pi=62`의 0.8px 반올림 경계가 사다리 분기를 건너뛰지 않도록 한다.
- 공개 canonical fixture `samples/issue6718/27469-child-allowance-retroactive-support.hwp`를 반드시
  읽는 2쪽·4쪽·10쪽 regression 3건을 포함한다. fixture SHA-256은
  `f619b8745d179562755e767307a3728dc7f8952fe70d376fbfe28aaf77ff66d7`다.
- 통합 candidate에서 `cargo fmt --all -- --check`,
  `cargo clippy --all-targets -- -D warnings`가 성공했고, #6702·#6732·#6733 focused nextest는
  5 passed, 0 failed였다. #6733의 2쪽·4쪽·10쪽 assertion도 모두 통과했다.
- candidate `rhwp info --json`은 canonical HWP의 12 logical pages를 보고했고, source regression의
  page-count 계약과 일치한다.

## 원격 CI 상태

- source PR의 [CI](https://github.com/edwardkim/rhwp/actions/runs/33886696932),
  [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/33886696916),
  [Adapter](https://github.com/edwardkim/rhwp/actions/runs/33886696999),
  [Proptest](https://github.com/edwardkim/rhwp/actions/runs/33886697299),
  [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/33886696441),
  [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/33886697037)는 모두 success다.
- 보류 중이던 [Rust CodeQL worker](https://github.com/edwardkim/rhwp/actions/runs/33886697037/job/101068237856)도
  완료 후 success로 확정됐다. 동일 source head는 `MERGEABLE`, `CLEAN`이다.

## 증적 범위

- native-skia `export-png --profile print`의 logical page 10은 [candidate p10](../assets/pr_6702_6732_planet6897_integration_20260904/visual-6733/candidate-p10/27469-child-allowance-retroactive-support.png)이며,
  SHA-256은 `e76972af16f901044d42bef7f8d6b03cd808edf1ec90827b3942c4a10d085cde`다.
- page footer `- 10 -`과 본문이 출력되는 것을 직접 확인했다. 환경의 한글 glyph 대체가 있으므로
  이 자료는 page 10의 body-overflow regression 범위만 뒷받침하며, Hancom/Studio full fidelity 또는
  pixel match의 증거는 아니다.
- 상세 해시와 한계는 [#6733 시각 증적](pr_6733_planet6897_visual_sweep.md)에 고정했다.

## 통합 PR CI 기록

- 통합 PR #6734의 code candidate `a239517aa32519ad8dce938768e63d3e45593ea7`에서
  CI, CodeQL Rust worker, Render Diff, Adapter inter-diff, Proptest, CI Impact Policy가 모두
  성공했다. Rust CodeQL worker 실행 시간은 12분 29초였다.
- 이 기록을 포함한 documentation-only trailing head는 별도 CI 성공 뒤에만 merge한다.

## 통합 전 조건

- 세 PR을 포함한 통합 candidate의 최신 commit으로 temporary upstream branch를 push하고 PR을 만든다.
- 통합 PR의 latest head required CI가 success 또는 policy expected skip이고 `MERGEABLE`, `CLEAN`인
  것을 확인한 뒤 일반 merge commit 방식으로 병합한다.
- source PR comment/close와 contributor branch 정리는 통합 merge 및 실제 `devel` CI 성공 뒤
  `post_merge.md` 절차에서 한 번만 처리한다.
