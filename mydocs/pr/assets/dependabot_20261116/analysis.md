# Dependabot 16개 PR 통합 검토 — 원인과 검증 범위

2026-11-16, `review/integration-20261116-dependabot`에서 최신 고정 devel
`6c1d134d42213ce0fb5f66b5ad2bf343b7b64fe0` 위로 원 작성자와 원 SHA를 보존해
16개 PR을 누적 체리픽했습니다. [원 PR·SHA·파일 목록](source-prs.json)을 따릅니다.
원 PR 각각에 `jangster77` reviewer를 지정한 뒤 API에서 확인했습니다.
현재 계정의 열린 self PR은 없었고 저장소 작업 트리는 시작 시 clean이었습니다.

## 원 PR 주장과 공통 소비

- #9110·#9115·#9117: Studio·Chrome·Firefox의 Vite8.3.8과 lockfile 갱신입니다.
  package 설치·TypeScript·기존 단위 검사·제품 및 확장 build·실제 browser 경로를 확인합니다.
- #9111·#9113: Puppeteer25.17.0의 기존 자동화 API와 Chrome/확장 검증 경로를 확인합니다.
- #9112: Anthropic SDK1.8.0의 기존 transport·메시지 변환·스트림 테스트를 확인합니다.
  실제 유료 API 호출이나 비밀 키를 테스트 기록에 넣지 않습니다.
- #9114·#9118·#9120·#9122·#9124: ureq·flate2·zip·crc32fast·aes의 원본 입출력·
  네트워크 mock·압축/해제·암호화 fixture와 전체 회귀를 확인합니다.
- #9116·#9121: read-fonts0.47·skera0.10의 글꼴 읽기·부분집합 생성과 Native/fresh WASM
  출력 소비를 확인합니다. #9125의 Skia0.157은 PNG/PDF의 API·그리기 변경 가능성이 있으므로
  Native Skia 범위, 원본 출력 및 정본 보호 문서·편람을 확인합니다.
- #9119·#9123: full SHA로 고정한 rust-cache·install-action만 갱신합니다.
  trigger·권한·cache key/save-if·nextest tool 선택을 바꾸지 않았습니다.
  O3 action 의존성 변경으로 YAML/actionlint·관련 workflow 정책 검사·실제 CI를 확인합니다.

## 텍스트 충돌과 lockfile 정합화

[충돌 기록](conflict-resolutions.json)의 Cargo.toml 인접 줄 두 건은 원 PR들의 목표를 모두
유지했습니다: read-fonts0.47·skera0.10·skia-safe0.157.0입니다. 기능을 선택적으로 버리지 않았습니다.
첫 `--locked` Native 빌드는 lockfile 갱신 필요로 컴파일 전에 exit101을 반환했습니다.
명시적인 의존성 갱신 작업 범위에서 `cargo update --offline -p skera --precise 0.10.0`을 실행했습니다.
도달하지 않는 read-fonts0.46 항목을 제거했습니다. 다만 이 재해결이 bindgen의 syn 선택도
3.0.3에서2.0.119로 바꾸어 prettyplease0.3.0의 syn3 File과 불일치하는 실제 컴파일 실패를
만들었습니다. bindgen의 원래 syn3 간선을 복원해 두 라이브러리의 AST 타입을 맞춥니다.
다른 패키지 버전은 추가로 올리지 않았습니다. 이 정합화 중 오류도 같은 검토 범위에서 수정합니다. 이 실패를 제품 회귀나 빌드 성공으로 기록하지 않습니다.
이후 검증은 다시 `--locked`를 사용합니다.

## 실행 순서와 남은 상태

현재는 16개 체리픽과 lockfile 정합화 단계입니다. Native 빌드부터 수행하고 API 비호환이 있으면
원 업데이트와 공용 호출 경로를 대조해 필요한 메인터너 호환 보정만 추가합니다.
전체 Rust 회귀는 최소 글꼴 Mac sandbox, Visual Sweep은 원본 글꼴을 공급한 일반 Mac에서
수행합니다. 순차 Cargo·공용 target/pr-review·jobs8·threads8·총 capture8 이하를 유지합니다.

검증이 끝나면 개별 원 PR review와 주장별 증거를 작성합니다. 통합 PR의 CI action 변경 때문에
옵션2를 적용하여 Full CI·정상 병합 뒤 별도 기록 PR에서 통합 PR review·오늘할일을 처리합니다.
통합 병합과 devel 반영을 확인하기 전에는 원 PR을 close하지 않습니다.
원 PR별 개별 판정·원 SHA·실제 검증을 댓글로 남긴 뒤 supersede close하고 원 bot branch는 보존합니다.
사용자가 요청한 비공개 글꼴 ZIP과 다른 작업의 자료를 보존합니다.

## 1차 실제 검증 완료

- 새 Skia0.157 Native CLI `--locked --profile release-test --features native-skia` 빌드를
 5분8초에 완료했습니다. 제품 API 호환 보정은 현재까지 필요하지 않았습니다.
- Studio·Chrome·Firefox npm ci, Studio TypeScript와 전체 단위2,798PASS·0FAIL·2SKIP을
 완료했습니다. [결과](npm-validation.json)와 [실제 해석 버전](resolved-versions.json)을 보존합니다.
- actionlint 전체 호출은 기존 advisory 두 곳의 SC2016 info를 보고했습니다. 같은 base와 후보의
 두 스크립트가 action SHA 외에는 byte-identical이며, 단일 인용 echo가 Markdown backtick을
 그대로 출력하는 의도임을 확인했습니다. 제품 결함으로 수정하거나 허용치를 바꾸지 않았습니다.
 Shellcheck를 제외한 actionlint 구문 검사와 Node CI 계약·Python workflow250건은 통과했습니다.
 [명령·결과](workflow-validation.json), [기존 진단 분석](actionlint-existing-shellcheck.json)을 구분합니다.
- 새 sandbox는 CoreText1890face 중 OS528face만 노출하며 비공개183개 프로그램 읽기를 모두
 거부했습니다. 기존 라이선스 확인 vendor bundled 글꼴은 명시적으로 허용했습니다.
 최초 프로필이 이 두 공개 DejaVu 파일까지 막아 발생한 컴파일 실패는 환경 구성 오류로 보존하고,
 [수정된 격리 조건](minimal-font-qualification.json)에서 필수 게이트를 다시 실행 중입니다.
