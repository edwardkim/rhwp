# PR #9351 Dependabot 의존성 갱신과 Skia 빌드 보정

## 최종 판정

2026년 11월 23일, Dependabot 원 PR 9건을 `jangster77`이 통합 검토했습니다. GitHub에서 검증한 코드 head는 `2ecdf5d5b1bf22cdab7e4501f7ed7d83934f0d1e`이며, `devel`에 병합된 커밋은 `be6f60aa6a93889b1bbc10e7e4a113dcc27a7f07`입니다. 자동 종료 대상으로 연결된 이슈는 없습니다. [통합 PR #9351](https://github.com/edwardkim/rhwp/pull/9351) · [병합 커밋](https://github.com/edwardkim/rhwp/commit/be6f60aa6a93889b1bbc10e7e4a113dcc27a7f07)

## 통합 범위

`upstream/devel` `7d6628982c81e702d58d8d0e6f52059f68a0b9ae`에 원 PR을 번호순으로 `git cherry-pick -x` 적용했습니다. 충돌은 없었습니다.

| 원 PR | 갱신 | 원 head | 적용 커밋 |
| --- | --- | --- | --- |
| [#9340](https://github.com/edwardkim/rhwp/pull/9340) | Chrome Puppeteer 25.17.0 → 25.18.1 | `00c9a3fde82fa0ca042479f8029df454e2c42ffd` | `4ab62adc12db7fb0a0014d15b093e50b8e7c5d9c` |
| [#9341](https://github.com/edwardkim/rhwp/pull/9341) | VS Code webpack 5.112.2 → 5.113.0 | `c46a3791edadc3479c2fddc739a4f877e4988c88` | `0b3cec699cb5159a38329ed4b0b96b6d32c82055` |
| [#9342](https://github.com/edwardkim/rhwp/pull/9342) | Studio Anthropic SDK 1.8.0 → 1.11.0 | `a06b0258c9eff0a09d7f3fa5887781f2cdda80fa` | `485bf81a05f042cb7b8ad952952acbed873d0a02` |
| [#9343](https://github.com/edwardkim/rhwp/pull/9343) | zeroize 1.9.1 → 1.9.2 | `0dce8949b7599858cd5e477b86498b29714e0c54` | `e06c26be98490544823d89db611711f240591bfe` |
| [#9344](https://github.com/edwardkim/rhwp/pull/9344) | Studio puppeteer-core 25.17.0 → 25.18.1 | `ac12dc214913f37109c0fb928fa974f54a77d09d` | `a473cffc070d4725475c4f82e1c5b4ac62b6b001` |
| [#9345](https://github.com/edwardkim/rhwp/pull/9345) | taiki-e/install-action v2.90.3 → v2.90.6 | `99e318a74ae38edb18714ee45b775f1a2692cd19` | `eb0b78ce4b054432d0c181a532c1a5c884ac3f7c` |
| [#9346](https://github.com/edwardkim/rhwp/pull/9346) | zip 9.5.1 → 9.8.0 | `cf8d97af3050dc87a87a919f8a7e5d64973986eb` | `a9b63316bf0e3fe01dcc5bb42158c6f7d1d4a510` |
| [#9347](https://github.com/edwardkim/rhwp/pull/9347) | clap 4.6.7 → 4.6.9 | `6f3f7ee647b3a5a26f75372f1f60f2fe78bfc469` | `8ae1169a8c2aebdfb4d79bcd81c3452aa986020d` |
| [#9348](https://github.com/edwardkim/rhwp/pull/9348) | regex 1.13.1 → 1.13.2 | `c9cb85fcf28bc9c0ac7116b6ad635c9e0545ab96` | `c553d634ee62ac11509cea512c5b789a85566993` |

#9347의 원 Native Skia와 Canvas CI는 `bindgen 0.73.2`의 `syn 2.0.119`와 `prettyplease 0.3.0`의 `syn 3.0.10` 사이의 AST 타입 불일치로 실패했습니다. 통합 보정에서 `Cargo.lock` 한 줄을 수정해 두 패키지가 `syn 3.0.10`을 공유하게 했습니다. #9340의 원 CI는 GitHub 액션 다운로드 정보 요청에서 타임아웃됐고, 통합 head의 Frontend package gates와 로컬 Chrome 확장 자동 열기가 통과했습니다. [#9347 실패 로그](https://github.com/edwardkim/rhwp/actions/runs/45102508830/job/137460027535) · [#9340 실패 로그](https://github.com/edwardkim/rhwp/actions/runs/45102333270/job/137455835948) · [Frontend package gates](https://github.com/edwardkim/rhwp/actions/runs/45125825462/job/137529941498)

## 로컬 검증

- macOS 글꼴 격리에서 전체 Rust 회귀 13,979건 통과, 실패 0건, 건너뜀 45건을 확인했습니다. fmt, Native·WASM32·workspace all-target Clippy, 워크스페이스 빌드, suite manifest와 unit tier가 통과했습니다. Skia 보정 전후 기본 Native와 WASM 의존성 그래프가 일치해 앞선 전체 회귀와 빌드 결과를 사용했습니다. 보정 적용 후 `native-skia`의 `skia` lib 검사는 57건 통과했습니다.
- Native CLI에서 `header-footer-paper-frame.hwpx`를 왕복 저장한 뒤 IR 차이는 0건이었고 ZIP 무결성 검사도 통과했습니다. 이 통합은 의존성·워크플로 갱신 범위라 보호 문서 검증과 로컬 Visual Sweep은 생략했습니다.
- macOS locked WASM 빌드(`--target web --out-dir pkg --no-opt`)와 실제 Studio 초기화 WASM 해시가 일치했습니다. Studio 단위 테스트는 2,832건 통과, 2건 건너뜀이었습니다. Studio production 빌드, Chrome 확장 smoke와 다운로드 자동 열기, VS Code production 번들이 통과했습니다.
- Studio AI 패널에 모의 응답을 주는 E2E에서는 같은 WASM을 사용한 SDK 1.8.0과 1.11.0이 각각 93개 assertion을 통과했습니다. 문단 위 간격 TC15는 두 버전 모두 실제 26.7px, 기대 13.3px로 동일하게 실패했습니다.
- Nextest 워크플로 두 파일의 `actionlint`가 통과했습니다. Bash 5.3의 워크플로 계약 21건은 기준·후보 모두 통과했습니다. macOS 기본 Bash 3.2에서는 기준·후보 모두 같은 검사 한 건이 실패했습니다. 액션 릴리스 v2.90.6의 공식 태그 SHA를 워크플로 고정값과 대조했습니다. [taiki-e v2.90.6](https://github.com/taiki-e/install-action/releases/tag/v2.90.6)

## GitHub 검증과 병합 후 처리

코드 head의 전체 CI는 성공 34건, 건너뜀 5건이었습니다. CI preflight는 `ci-execution-surface-change`를 근거로 fast-pass를 사용하지 않았습니다. Build & Test, Native Skia, Canvas visual diff, CodeQL, Adapter, Proptest와 CI Impact Policy가 통과했습니다. [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/45125825462/job/137541928363) · [Native Skia](https://github.com/edwardkim/rhwp/actions/runs/45125825462/job/137529941439) · [Canvas visual diff](https://github.com/edwardkim/rhwp/actions/runs/45125825401/job/137529965458) · [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/45129664393)

병합 뒤 Nextest 시간 데이터 44개 대상이 갱신됐고 메타데이터 게시가 성공했습니다. 워크플로 변경을 포함한 코드 PR의 검토 기록은 옵션 2로 분리했습니다. [시간 데이터 갱신](https://github.com/edwardkim/rhwp/actions/runs/45130954486)
