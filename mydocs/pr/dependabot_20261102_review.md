# Dependabot PR 일괄 검토 — 2026-11-02

## 최종 판정

**머지 보류 — 필수 검증 진행 중.** 8개 원 head는 제품 CI가 성공했고 Skia #8456은 build dependency의 syn 버전 조합을 보정해야 한다. 원격 push·review·comment·merge는 별도 승인 전 수행하지 않는다.

## 범위와 기준

- 기준 devel: `d3f7209bbb0fdb1711c5f40a70ad22f0ca0be66d`.
- 기본 경로: `maintainer_general`. 보조: `intake_and_review`, `local_validation`, `multi_pr_update_branch`, `rework_and_exceptions`; 실제 렌더링 소비 경로는 `visual_fixture_evidence`를 추가한다.
- 9개 원 PR별 reviewer `edwardkim` 지정과 API 재확인을 완료했다. 모두 Open·non-draft·devel 대상이다.
- 원 head는 보존한다. 독립·누적 merge-tree에서 충돌은 없었으며 사용자 기본 devel·다른 worktree·기존 Studio WASM은 보존한다.
- 8개 원 PR의 녹색 전체 CI는 4.3.0 조건을 충족하는 범위에서 재사용한다. 실제 소비 경로의 focused·브라우저·시각 검증과 Skia 보정의 필수 검증은 별도로 완료한다.

| PR | 변경 | 원 head | 현재 상태 |
| --- | --- | --- | --- |
| [#8449](https://github.com/edwardkim/rhwp/pull/8449) | chore(deps-dev): bump @types/chrome from 0.3.6 to 0.3.7 in /rhwp-studio | `2fa686d569a078ce9e44e05ca493e65d704784c7` | 제품 CI success |
| [#8450](https://github.com/edwardkim/rhwp/pull/8450) | chore(deps): bump zip from 9.0.5 to 9.1.3 | `94fdf6fea2a7b56865e41ef3e8c8923033211c07` | 제품 CI success |
| [#8451](https://github.com/edwardkim/rhwp/pull/8451) | chore(deps): bump @anthropic-ai/sdk from 1.1.0 to 1.3.0 in /rhwp-studio | `b4b31c7fc834c2ed502ef1983e9fc36b341ab41f` | 제품 CI success |
| [#8452](https://github.com/edwardkim/rhwp/pull/8452) | chore(deps): bump unicode-width from 0.2.2 to 0.2.3 | `5829d65eb6fd368a2557c016df4747d7b12362ca` | 제품 CI success |
| [#8453](https://github.com/edwardkim/rhwp/pull/8453) | chore(deps-dev): bump webpack from 5.112.1 to 5.112.2 in /rhwp-vscode | `dc8b52ad418594a42b52560c7b706ca0fe4f1783` | 제품 CI success |
| [#8454](https://github.com/edwardkim/rhwp/pull/8454) | chore(deps): bump libc from 0.2.189 to 0.2.191 | `4dd0e7eb196a1bfede636c4314fbe8fcc7bec8eb` | 제품 CI success |
| [#8455](https://github.com/edwardkim/rhwp/pull/8455) | chore(deps): bump roxmltree from 0.21.1 to 0.21.2 | `db38e6af4d65e5a8bdc7ba2697d3658a9839e505` | 제품 CI success |
| [#8456](https://github.com/edwardkim/rhwp/pull/8456) | chore(deps): bump skia-safe from 0.153.3 to 0.156.0 | `54d0255c594241d1593adff0367a236a9b6f3ebc` | Native/Canvas CI build 실패 |
| [#8457](https://github.com/edwardkim/rhwp/pull/8457) | chore(deps): bump taiki-e/install-action from 2.87.31 to 2.88.1 | `cd0ccf51995d9e985f8137d211db6077d7003a8b` | 제품 CI success |

## Skia 진단과 보정 범위

원 #8456의 Native와 Canvas visual diff는 `bindgen 0.73.2`에서 같은 E0277로 실패했다. 실제 Canvas 비교 단계는 시작하지 못했다. 새 bindgen은 `syn >=2,<4`를 허용해 lock에서 syn 3.0.3을 고르지만, 남아 있는 prettyplease 0.2.37은 syn 2.0.119를 사용한다. bindgen의 `prettyplease >=0.2.7,<0.4` 범위 안에서 syn 3 기반 prettyplease 0.3.0을 선택해 type 계약을 일치시키는 lockfile 보정을 검증한다. 제품 소스·CI 기대값을 바꾸는 우회는 하지 않는다.

- [bindgen 0.73.2 manifest](https://github.com/rust-lang/rust-bindgen/blob/v0.73.2/bindgen/Cargo.toml)
- [prettyplease 0.3.0 manifest](https://docs.rs/crate/prettyplease/0.3.0)
- 로컬 source branch: `review/pr-8456-skia-upgrade`, 원 head에서 연속 추가 commit.
- 누적 검증 branch: `review/integration-20261102-dependabot`, 8개 원 기여 commit을 보존한 merge. Skia 보정 뒤 이 후보에서 결합 호환성도 검증한다.

## 검증 계획

1. npm 변경 3개와 전이 delta의 설치 스크립트·취약점·integrity·호스트 변경, 각 upstream release와 직접 소비 경로를 확인한다.
2. Studio TypeScript·npm test·빌드·실제 브라우저와 VS Code compile을 실행한다.
3. Rust XML/ZIP·자원 경계·암호·저장 왕복·기존 #8309를 focused 검사한다. unicode-width의 도형/캐시 소비와 XML/SVG의 실제 출력 영향은 Native/fresh WASM으로 대조한다.
4. Skia의 lock 보정 후 필요한 lint·전체 회귀·Native·최적화 WASM과 독립 Print 시각 검증을 완료한다. 실패한 기존 검사나 baseline을 완화하지 않는다.
5. PR별 수용 판정·CI·current-base merge 근거를 분리해 기록하고 구체적인 원격 처리 승인을 요청한다.

## npm 감사의 범위

NPMScan `diff_dependencies`로 PR별 실제 변경 lock entry만 대조했다. @types/chrome(dev), Anthropic SDK(runtime), webpack(dev) 각 1개씩이며 전이 패키지 변경은 없다. unchanged graph를 생략한 축약 입력이므로 도구의 다른 root dependency 미포함 경고는 전체 lockfile 결손을 뜻하지 않는다. 변경 3개는 `still-clean`, 신규 설치 스크립트·출처 바뀜·identity mismatch는 없었다. 전체 프로젝트의 모든 기존 npm 의존성 감사를 수행한 것으로 표시하지 않는다.

실행 로그·임시 원문: 기본 작업공간의 `output/pr-review/dependabot-20261102/`. 확정 근거는 검증 완료 뒤 정식 asset과 개별 PR review로 보존한다.
