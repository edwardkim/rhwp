# #8160 11단계: Native 전용 회귀의 CI 실행 누락 보정

## 관측과 범위

- 작업 브랜치 `task/8160-sample16-raw100`, 이전 제출 head `076b19d364a552b5c38da1a9639e48587f6f8b1e`, 최신 base `4e690c930105fa09fef484787ed0a2e5e182d444`다.
- 이전 head의 CI·CodeQL·Render Diff·CI Impact Policy는 모두 성공했다. 실제 GitHub archive 로그에서 #8160 기본 86건과 신규 WMF 상위 클립 검사 PASS를 확인했다.
- 최종 실행 경로 대조에서 `issue_8160_hwp2022_paragraph_relations`의 `#[cfg(feature = "native-skia")]` 이중 테두리 1건이 Native job 호출 목록에 없음을 확인했다. 로컬 3건 실행은 통과했지만 CI의 default 수집은 이 전용 검사 실행 증거가 아니다.
- 고정 base의 Native job도 해당 case를 호출하지 않았다. 새 devel이 기존 호출을 삭제했다거나 앞선 PR이 호출을 제거했다고 설명하지 않는다.

## 보정 계획

GitHub 운영 절차 5.3.1에 따라, 실제 누락이 확인된 case만 기존 Native Skia job의 `release-test`와 `release` 분기에 각각 한 줄 연결한다. 새로운 job·step·필터·권한·이벤트는 추가하지 않는다. 기존 `run-rust-test.mjs --cargo-test`의 논리 case 선택을 사용하며 파생 suite 번호를 고정하지 않는다.

기존 Python workflow 계약이 요구하는 Native case 소유 목록에도 같은 경로를 등록한다. 이 파일만 바꾸는 미래 PR에서도 실제 분류기가 `native_skia_required=true`를 반환하도록 기존 Native 경계 분류 검사의 입력 목록에 포함한다. workflow mirror 검사의 기대값은 완화하지 않는다.

추가 실행은 해당 case 3건이다. Native 전용 이중 테두리 1건과 같은 Native feature 구성에서 BCP 두 줄·이어받는 표의 2건을 함께 확인한다. 기존 default 2건을 새로운 고유 검사로 세지 않는다. 새로운 테스트 본문·기대값·문턱·제품 코드는 바꾸지 않으며 전체 #8160 고유 검사 수는 87개다.

## 검증과 후속 조건

- O3 실행 계약의 관련 검증: actionlint, 기존 Node CI/classifier/policy·suite/locking 검사와 Python CI workflow 계약, 실제 Native case의 3건 실행·건수·exit 0을 확인한다.
- Rust source·test·fixture·입력/PDF가 이전 검증과 바이트 동일하므로, 기존 전체 nextest·전쪽 Native/fresh WASM raw100·직접 판독 증거를 재사용한다. 실행하지 않은 전체 재검사를 새 실행으로 표시하지 않는다.
- 기존 workflow의 이벤트·fork 조건·권한·기존 Native 호출을 유지한다. 두 줄의 호출 추가만 rollback 단위다.
- 새 head에 해당 실제 Native 실행이 포함된 GitHub Actions와 CI Impact Policy가 모두 성공해야 self-review·오늘할일 trailing commit과 머지를 진행한다. workflow 변경을 문서-only로 처리하거나 이전 녹색 CI만으로 통합하지 않는다.

검증 결과는 아래에 실제 종료 뒤 기록한다. 임시 실행 스크립트·로그는 커밋하지 않는다.

## 실제 보정 결과

- 기존 Native job의 두 profile 분기와 Native 소유 목록·실제 분류 회귀의 입력 목록을 보정했다. 이벤트·권한·fork 조건과 나머지 Native 호출은 바꾸지 않았다.
- actionlint exit 0, 기존 Node CI/classifier/controller·suite/locking 117건 PASS, Python CI/workflow 계약 66건 PASS다. 최초 Python 실행은 소유 목록 누락 1건을 검출했고, 목록과 실제 분류 입력을 연결한 뒤 동일 66건을 다시 실행해 통과했다.
- CI와 같은 `--cargo-test` 경로에서 `issue_8160_hwp2022_paragraph_relations` 3건을 실제 실행했다. 이중 테두리 Native 전용 1건 포함 3 PASS·0 FAIL·266 filtered, 37.89초다. 필터 제외는 통과 수에 합산하지 않았다.
- 기존 최종 PNG로 본문 게시용 review/standalone overlay만 만들었다. 제품을 다시 출력한 것으로 표시하지 않으며, 384개 raw100 원본과 원본/Print 75개 commit 바이트의 증거를 연결했다.
- [CI 연결과 실제 로컬 결과](../pr/assets/issue_8160/rebase-8231/native-ci-registration.json)를 고정한다. 새 head 원격 CI는 push 뒤에 확인한다. 성공 전에는 self-review·오늘할일을 확정하지 않는다.
