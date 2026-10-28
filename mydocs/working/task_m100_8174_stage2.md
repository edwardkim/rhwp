---
kind: snapshot
status: active
canonical: mydocs/plans/task_m100_8174_impl.md
last_verified: 2026-10-23
---

# #8174 2단계 — 결정 비의존 공통 선행 검사

Issue: [#8174](https://github.com/edwardkim/rhwp/issues/8174)

## 결과와 승인 범위

**C01–C04의 공통 선행 절편 완료. 정책 의존 manifest·독립 Cargo 소비·실제 게시 준비는 아직 완료하지 않았다.**

- 승인: 2026-10-23 사용자 「MD01 은 결정 사안입니다. 메인테이너의 결정과 무관하게 해야 하는 처리부터 착수합니다」.
- 검증 source: `ef376d16516286b94b73f06329cf164b94def3b0`. 제품 source는 기존 `def947c21`과 같다.
- [결정 대장](../plans/task_m100_8174_maintainer_decisions.md)의 MD01–MD09와 정책 의존 구현 범위 v1은 미결정이다.
- [수행 계획](../plans/crates_io_preparation_20261023.md), [구현 계획](../plans/task_m100_8174_impl.md).

## 수행한 처리

| 범위 | 구현과 실제 확인 |
| --- | --- |
| C01 · source 재현성 | [정식 도구](../../scripts/crates_io_source_preflight.py)가 Git commit의 blob으로 후보 archive를 생성한다. 미커밋 source·untracked 파일은 포함하지 않는다. 8개 후보를 두 번 만들어 파일 목록·크기·digest가 같은 것을 확인했다. |
| C02 · 필수 자료·고지 | [내장 파일 등록](../../packaging/crates-io/runtime-files.json)의 14개 literal 참조·상대 경로, font registry의 글꼴 5개와 라이선스/파생 provenance를 확인했다. 사유·검증 corpus·환경 파일·symlink가 후보에 들어가는 실패 경계를 검사한다. |
| C03 · manifest 차단점 | 실제 manifest의 path 의존성·workspace 상속·root patch를 읽어 후보 graph와 대조한다. 버전 없는 path 의존성 3개, `publish=false` 6개, 소비자에게 전파되지 않는 root patch 1개를 보고했다. 값은 수정하지 않았다. |
| C04 · 검증·보고 | [로컬 계약 검사](../../scripts/tests/test_crates_io_source_preflight.py) 18개, 실제 source 실행·전 파일 archive readback·반복 생성 비교를 통과했다. 성공도 `publish_ready=false`이며 Cargo 검증과 구분한다. |

[사용법과 결과 해석](../../packaging/crates-io/README.md)에 명령·출력·제한을 기록했다.
입력 policy는 여전히 제안이며, 도구 실행이 8개 패키지 구조·게시 이름·버전을 채택한 결정은 아니다.

## 실제 누락의 보정

추가한 font 검사로 기존 후보에서 `assets/fonts/LatinModernMath-VariableOutline-subset.otf`의
파생 provenance가 빠진 것을 재현했다. 기존 `font-notices.json`과 `THIRD_PARTY_LICENSES.md`가
요구하는 [원본·파생 manifest](../../assets/fonts/MANIFEST-RHWP-LMM-VariableOutline.txt)를
[source 선별 목록](../../packaging/crates-io/source-policy.json)에 추가했다.

- 보정 전: 실제 검사 exit 2, `font license/provenance omitted`.
- 보정 후: 같은 source의 검사 exit 0. 글꼴 5개와 각각의 라이선스/provenance 포함을 확인했다.
- 글꼴 bytes·font registry·라이선스·제품 코드는 변경하지 않았다. 재배포 권리의 새 법적 판단이나
  PDF 전체 메모리 감사 완료를 뜻하지 않는다.

본체 후보는 **1,793파일·원본 39,822,445바이트·gzip tar 8,329,770바이트**다.
1단계의 1,792파일·8,329,278바이트와 구분한다. 파생 manifest 1,198바이트를 추가했고,
1단계 임시 도구가 전부 0644로 썼던 archive 모드도 Git의 100755 항목 7개에 맞게 보존했다.
실제 작업 트리의 파일 권한을 변경한 것은 아니다. 다른 7개 후보의 archive digest는 1단계와 같다.

본체 source archive SHA-256:
`4a109c7b88971c21262e0600e1c071edade40c9ae43eda3a80d70e2f58c2e7fe`.

## 검증과 증적

ignored `output/pr-review/issue8174/source-preflight-stage2/`에 원장·실행 결과를 보존했다.

| 실행 | 결과 | 증적 |
| --- | --- | --- |
| `python3 -m unittest scripts/tests/test_crates_io_source_preflight.py -v` | 18 PASS / 0 FAIL. 경로 이탈·비공개 자료·runtime/라이선스 누락·graph 불일치·오래된 성공 보고서·재현성 경계 포함 | `unit-tests.log`, `unit-tests.command.json` |
| `python3 scripts/crates_io_source_preflight.py --output-dir output/pr-review/issue8174/source-preflight-stage2` | exit 0. 8개 후보·14개 내장 참조·5개 글꼴 검사, 전 파일 bytes/mode readback | `source-preflight.json`, `*.files.json`, `*.source.tar.gz`, `source-preflight.command.json` |
| 같은 source·policy·도구로 임시 출력 디렉터리에 반복 생성 | 8개 후보의 원장·압축 크기·digest 동일. 비교 뒤 임시 사본 제거 | `reproducibility.json` |
| 실제 font 검사의 보정 전 실패 | exit 2. 최종 도구와 Git에 기록된 원래 policy로도 파생 provenance 누락 재현 | `font-audit-before-fix.json`, `font-audit-final-tool.json` |
| `node --test scripts/frontend-font-assets.test.mjs` | 최종 8 PASS / 0 FAIL | `font-contracts-after-refresh.log` |

최종 도구 파일 SHA-256:
`186885d14e8aad1298152dcf41cb8b971252e0c8298e3a9980b454990bcce2d7`.

보조 font 검사는 처음에 기존 Chrome·Firefox·VS Code `dist`의 오래된 폰트/고지 때문에
5 PASS / 3 FAIL이었다. 현재 build.mjs·webpack.config.js의 font 복사 규칙에 따라 ignored
폰트·고지 디렉터리만 갱신하고 8개 전체를 다시 통과했다. 갱신 경로·원래 목록은
`font-distribution-refresh.json`에 남겼다. frontend 전체 빌드·WASM 재빌드·배포 완료로 표시하지 않는다.

문서 메타데이터 전역 검사는 623개 문서 중 기존 5개 파일의 오류 20건을 반환했다. 해당 5개는
HEAD와 바이트가 같음을 확인했으며, 새 결정 문서의 직접 검사는 오류 0건이다. 기준을 바꾸거나
관계없는 기존 문서를 수정하지 않았다. 변경 문서 6개의 상대 링크·언어·`git diff --check`, Python
구문을 확인했다. `document-check.json`에 신규 메타데이터 오류 0건과 기존 오류 파일의 바이트 일치를 기록했다.

기존 자동 실행 경로를 확인했다. 이 로컬 Python 도구의 검사는 명시한 명령으로 실행하며,
CI workflow를 추가하지 않았다. 실제 게시 경로가 선택되면 필요한 최소 연결을 그 범위에서 검토한다.

## 다음 처리의 경계

현재 제품 manifest·Cargo.lock·기본 feature·보안 제한·배포 workflow·인증은 그대로다.
Rust 제품 source를 바꾸지 않아 전체 Rust 회귀·fresh WASM·시각 검증을 반복하지 않았다.
이 결과를 정규화한 `.crate` 또는 저장소 밖 registry 소비 검증으로 대체하지 않는다.

정책에 의존하는 다음 구현은 관련 MD 결정과 지원 계약을 기록해 진행한다. 이번 지시는
공통 기반 착수이며 MD01 선택, 원격 push·PR·실제 crate 게시의 승인으로 확장하지 않았다.
