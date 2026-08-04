# PR #2811 검토 기록

| 항목 | 내용 |
|---|---|
| PR | [#2811](https://github.com/edwardkim/rhwp/pull/2811) |
| 작성자 / base | [@planet6897](https://github.com/planet6897) / `devel` |
| reviewer | [@jangster77](https://github.com/jangster77) |
| 관련 이슈 | [#2668](https://github.com/edwardkim/rhwp/issues/2668), [#2559](https://github.com/edwardkim/rhwp/issues/2559) |
| 범위 | per-page 각주-밴드 배분 가설을 한글 COM PDF 기하 측정으로 반증하고, 재현용 진단·분석 하니스를 추가 |
| 처리 경로 | collaborator 체리픽 누적 통합 검토. 기여 커밋 `ad25eb97`만 적용하고 원 PR의 `Merge branch 'devel'` 커밋은 제외 |
| 통합 기준 | `upstream/devel` `4775e8c2` 위 체리픽, #2807·#2810과의 충돌 0건 |

## 검토 결론

측정 보고서는 16개 문서의 유효 각주 741쪽을 대상으로 body area 바닥과 한글 PDF 각주 블록의 기하를 대조한다. 쪽 번호 오인과 구분선 오검출을 분리한 뒤, 한글이 빈 꼬리말 밴드를 각주에 쓰지 않는다는 결과와 `footer_band_reclaim`이 다른 페이지 적재 오차를 상쇄하는 근사라는 가설을 기록한다.

프로덕트 기본 동작은 바꾸지 않는다. `RHWP_FB_OFF`는 A/B 계측 시에만 밴드 회수를 끄고, `RHWP_DIAG_FBAND`는 진단 값을 stderr로 낸다. 두 환경 변수가 설정되지 않은 통상 실행에서는 기존 경로가 유지된다. `tools/task2668/`은 Windows 한글 COM 측정용 조사 도구이며, 제품 빌드나 배포물의 의존성은 추가하지 않는다.

이 PR은 [#2668](https://github.com/edwardkim/rhwp/issues/2668)의 해결이 아니라 기존 설계 전제의 반증과 목표 재정의 근거를 남기는 변경이다. 따라서 해당 이슈는 open 상태로 유지한다.

## 검증

- `git diff --check`, `cargo fmt --all -- --check`: 성공
- `python3 -m py_compile tools/task2668/*.py`: 성공
- 체리픽 누적 검증의 `cargo` focused test, IR field-sweep baseline, `wasm-pack build` 성공
- 최신 원 PR head GitHub Actions: CI, CodeQL, Render Diff 전체 성공

## 권고

통합 PR의 최신 CI와 작업지시자 승인을 조건으로 수용한다. merge 뒤 [#2668](https://github.com/edwardkim/rhwp/issues/2668)은 open으로 유지하고, 다음 구현은 보고서가 제시한 footnote safety margin 등 다른 원인 후보를 별도 검증한 뒤 결정한다.
