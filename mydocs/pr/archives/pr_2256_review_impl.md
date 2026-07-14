# PR #2256 체리픽 통합 구현 계획 및 기록

## 체리픽 순서

| 원본 PR | 원본 기능 커밋 | 통합 커밋 |
| --- | --- | --- |
| [#2232](https://github.com/edwardkim/rhwp/pull/2232) | `c7fba900` | `bde419f3e` |
| [#2242](https://github.com/edwardkim/rhwp/pull/2242) | `1194de03`, `1010ca21`, `a51b4efc`, `f8401953`, `4a1886fd` | `8ae55c243`, `bb6721bef`, `83074d8ae`, `5860e3694`, `124ba6a96` |
| [#2245](https://github.com/edwardkim/rhwp/pull/2245) | `8704a6dd`, `1146c170`, `fa603712` | `1fcb9bc86`, `f3c899f35`, `a61de9ddd` |
| [#2247](https://github.com/edwardkim/rhwp/pull/2247) | `a4e80619`, `14c84b13` | `708c3c7cd`, `33b78e688` |
| [#2251](https://github.com/edwardkim/rhwp/pull/2251) | `a78fa302`, `3cd4cd31`, `94a11698`, `706012d3` | `088643b95`, `a8f2eec29`, `1172ecbcc`, `399df58fe` |

## 충돌 처리

- [#2232](https://github.com/edwardkim/rhwp/pull/2232) 체리픽 중 `src/document_core/commands/document.rs` 한 곳이 최신 HML/XML import 변경과 충돌했다.
- XML/HML의 `include_empty` 의미는 유지하고, HWP5 문서에서만 `include_cell_empty`를 계산해 `reflow_zero_height_paragraphs`에 전달했다.
- 문서명, 표본명, 페이지 수 기반 예외는 추가하지 않았다.

## 최신 원격 head 대조

- `git range-diff upstream/devel...upstream/pr-2251 upstream/devel...399df58fe` 및 source diff로 현재 원격 stacked head와 통합 코드의 source/sample patch가 동일함을 확인했다.
- 통합 브랜치의 추가 차이는 이미 `devel`에 들어간 [PR #2255](https://github.com/edwardkim/rhwp/pull/2255) 관련 기반과 review/stage/PDF asset 기록이다.

## merge 후 순서

1. [#2256](https://github.com/edwardkim/rhwp/pull/2256)의 최신 CI와 merge SHA를 확인한다.
2. close keyword 대상 여섯 issue의 자동 close를 2~3회 확인하고, 열려 있으면 검증 요약을 포함해 수동 close한다.
3. 원본 PR [#2232](https://github.com/edwardkim/rhwp/pull/2232), [#2242](https://github.com/edwardkim/rhwp/pull/2242), [#2245](https://github.com/edwardkim/rhwp/pull/2245), [#2247](https://github.com/edwardkim/rhwp/pull/2247), [#2251](https://github.com/edwardkim/rhwp/pull/2251)에 감사·검증·통합 PR 링크를 남기고 supersede close한다.
4. `devel`을 fast-forward하고 통합 작업 브랜치의 로컬/원격 ref를 삭제한다.
5. [#2097](https://github.com/edwardkim/rhwp/issues/2097), [#2070](https://github.com/edwardkim/rhwp/issues/2070)은 open 상태를 유지한다.
