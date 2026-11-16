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

## 글꼴 공급 판단과 검증 진행 기록

과거 nested 생산 기록의 특정 `gulim.ttc` 원본 hash는 확보하지 못했으므로 동일 파일이라고
기록하지 않습니다. 사용자가 추가 질문 없이 판단하도록 지정하여, 이미 확보한 보호 글꼴 묶음의
파일 hash·공급 순서를 고정하고 현행 정본 baseline과 같은 입력·PDF의 전쪽 출력을 대조했습니다.
[판단 근거](font-supply-decision.json)의 base nested 17쪽은 최저 99.86%이며 공식 보호 표시값보다
하락한 쪽은 없습니다. 이는 해당 공급 조건의 실제 재현 결과이며, 후보와 fresh WASM의 완료를
대신하지 않습니다. 원본 동일성 추정·점수 완화·폰트 예외 승인은 적용하지 않았습니다.

2026-11-16 13시대 KST, 최소 글꼴 sandbox의 최적화 release 빌드는 실제 통과했고 release lib
검사가 실행 중입니다. 일반 Mac Native 캡처는 RAM·swap 경합을 줄이기 위해 기존 프로세스를
보존한 채 대기시키고 Rust 게이트 종료 후 자동 재개하도록 했습니다. 중복 Cargo는 실행하지
않았습니다. 대기 시간이 포함된 캡처 wall time을 성능 측정값으로 사용하지 않습니다.
개별 원 PR 16개의 검토 초안은 임시 output에서 준비하며 최종 증거·CI 확인 전 승인으로 기록하지
않습니다. fresh WASM·전쪽 보호 비교·제품 소비 경로도 선행 게이트 통과 후 순차 실행하도록
준비했으며 아직 완료로 집계하지 않습니다.

## 공개 fixture 차단 보정과 체리픽 내용 확인

전체 nextest의 첫 컴파일 시 `ttfs/redistributable/happiness-sans/HappinessSansVF.ttf`를
격리 프로필이 차단했습니다. 이 파일은 기존 저장소가 공식 원본·라이선스와 함께 추적하는
variable font fixture이며 테스트의 `include_bytes!` 입력입니다. 해당 디렉터리만 허용하고
OS528face·비공개183개 파일 읽기 차단을 다시 확인했습니다. 제품·회귀 기대값은 변경하지
않았습니다. 이 실행은 테스트 본문 전 컴파일 실패로 기록하고, 성공한 release build/lib는
같은 제품 입력·원래 프로필 snapshot으로 보존한 채 전체 nextest부터 다시 실행합니다.

[체리픽 payload 감사](cherry-pick-provenance-audit.json)에서 원 PR 16개 중 15개는 추가·삭제
내용이 그대로입니다. 12개는 patch-id도 동일하며 나머지는 인접 갱신의 문맥 차이입니다.
#9121은 #9116이 이미 추가한 동일 read-fonts0.47 package stanza를 중복 추가하지 않은 차이만
있습니다. skera·skrifa·write-fonts의 갱신과 최종 read-fonts0.47 노드는 보존했습니다.
