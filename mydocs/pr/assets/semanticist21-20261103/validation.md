# 누적 검토의 최종 로컬 검증

GitHub 제출 뒤 추가한 [진단 출력 보안 보정](ci-security-followup.md)은 아래 조판 측정과 구분한다. 아래 결과는 측정·검증 당시 head의 기록이며, 후속 보정 검증은 별도 절에 기록한다.

검증 head는 `c6ad3eea59d1b7701efdc4fb582e2d9a44b4f775`이며 제품 source는 `39b4f81f6aab4cc08c6145ed6911db2a8fe5125e`다. 기준 devel은 `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4`다. 이후 문서·증적 정리는 제품 구현과 테스트 실행문을 바꾸지 않는다.

## 판정

10건 모두 **메인터너 보정 후 수용 가능**이다. 이 판정은 보정한 누적 후보에 한정되며 충돌이 남아 있는 원격 원 head를 그대로 승인한 것이 아니다. 원격 push·공개 댓글·Open PR·merge는 아직 수행하지 않았다.

31개 입력·71쪽을 Native Skia와 fresh optimized WASM Canvas 각각 전쪽 비교했다. 모든 페이지가 raw 90% 이상이고 독립 PDF와 쪽수가 같다. 아래 최저값과 별도로 변경 경계·작은 글자·간격을 확대 판독했다.

| 입력 | 쪽수 | Native 최저 raw | WASM 최저 raw | base 대비 최악 쪽 변화 Native / WASM |
| --- | ---: | ---: | ---: | ---: |
| form | 1 | 99.60% | 99.58% | -0.40 / -0.42%p |
| form-slot-hwp | 1 | 100.00% | 100.00% | +0.00 / +0.00%p |
| form-slot-hwpx | 1 | 100.00% | 100.00% | +0.00 / +0.00%p |
| notice-reopened | 24 | 97.74% | 97.46% | +0.00 / +0.00%p |
| header-hwp | 1 | 100.00% | 100.00% | +1.65 / +1.76%p |
| header-hwpx | 1 | 100.00% | 100.00% | +1.65 / +1.76%p |
| rotated-hwp | 1 | 100.00% | 100.00% | +0.00 / +0.00%p |
| rotated-hwpx | 1 | 100.00% | 100.00% | +0.00 / +0.00%p |
| grow-before | 1 | 100.00% | 100.00% | +0.00 / +0.00%p |
| grow-after | 1 | 100.00% | 100.00% | +0.00 / +0.00%p |
| nested-wide | 1 | 100.00% | 100.00% | +0.00 / +0.00%p |
| split-row | 18 | 92.88% | 92.88% | +0.00 / +0.00%p |
| ownership-fresh-reference | 1 | 100.00% | 100.00% | +0.00 / +0.00%p |
| ownership-loaded-before | 1 | 100.00% | 100.00% | +0.00 / +0.00%p |
| ownership-empty-host-after | 1 | 100.00% | 100.00% | +0.00 / +0.00%p |
| ownership-after-table-after | 1 | 100.00% | 100.00% | +0.00 / +0.00%p |
| polygon-90 | 1 | 100.00% | 100.00% | +0.00 / +0.00%p |
| polygon-270 | 1 | 100.00% | 100.00% | +0.00 / +0.00%p |
| polygon-hflip | 1 | 100.00% | 100.00% | +0.00 / +0.00%p |
| polygon-vflip | 1 | 100.00% | 100.00% | +0.00 / +0.00%p |
| polygon-resize | 1 | 100.00% | 100.00% | +0.00 / +0.00%p |
| spacing | 1 | 100.00% | 100.00% | +0.00 / +0.00%p |
| grow-nested | 1 | 100.00% | 100.00% | +0.00 / +0.00%p |
| grow-nested-unchanged | 1 | 100.00% | 100.00% | +0.00 / +0.00%p |
| spacing-before | 1 | 100.00% | 100.00% | +0.00 / +0.00%p |
| forms-before | 1 | 100.00% | 100.00% | +100.00 / +100.00%p |
| forms-edited | 1 | 100.00% | 100.00% | +89.08 / +89.08%p |
| no-lineseg-empty | 1 | 100.00% | 100.00% | +0.00 / +0.00%p |
| no-lineseg-text | 1 | 100.00% | 100.00% | +0.00 / +0.00%p |
| spacing-after | 1 | 100.00% | 100.00% | +0.00 / +0.00%p |
| no-lineseg-wrap | 1 | 92.90% | 96.56% | +81.31 / +84.75%p |

각 전쪽 원본은 `tsv/base/{native,wasm}/`와 `tsv/final/{native,wasm}/`에 있다. 평균으로 미달 쪽을 상쇄하지 않았다. form-01만 동일 HBATANG 공급에서 Native −0.40%p·WASM −0.42%p이며 2%p 하락 기준 이내다. base의 Forms 0/10.92%와 wrap 11.59/11.81%, notice의 기존 미달은 후보를 면제한 것이 아니라 보정 전 음성 대조다.

## 실제 편집·저장 재열기

| 실제 경로 | 전쪽 수 | 최저 raw | TSV |
| --- | ---: | ---: | --- |
| final-actual-native-grow-nested-live-scores | 1 | 100.00% | [final-actual-native-grow-nested-live-scores.tsv](tsv/actual/final-actual-native-grow-nested-live-scores.tsv) |
| final-actual-native-grow-nested-reopened-scores | 1 | 100.00% | [final-actual-native-grow-nested-reopened-scores.tsv](tsv/actual/final-actual-native-grow-nested-reopened-scores.tsv) |
| final-actual-native-notice-live-scores | 24 | 97.71% | [final-actual-native-notice-live-scores.tsv](tsv/actual/final-actual-native-notice-live-scores.tsv) |
| final-actual-native-notice-reopened-scores | 24 | 97.71% | [final-actual-native-notice-reopened-scores.tsv](tsv/actual/final-actual-native-notice-reopened-scores.tsv) |
| final-actual-wasm-grow-nested-live-scores | 1 | 100.00% | [final-actual-wasm-grow-nested-live-scores.tsv](tsv/actual/final-actual-wasm-grow-nested-live-scores.tsv) |
| final-actual-wasm-notice-live-scores | 24 | 97.46% | [final-actual-wasm-notice-live-scores.tsv](tsv/actual/final-actual-wasm-notice-live-scores.tsv) |
| final-cross-native-visual/grow-nested-native-saved | 1 | 100.00% | [final-cross-native-visual__grow-nested-native-saved.tsv](tsv/actual/final-cross-native-visual__grow-nested-native-saved.tsv) |
| final-cross-native-visual/grow-nested-wasm-saved | 1 | 100.00% | [final-cross-native-visual__grow-nested-wasm-saved.tsv](tsv/actual/final-cross-native-visual__grow-nested-wasm-saved.tsv) |
| final-cross-native-visual/notice-native-saved | 24 | 97.74% | [final-cross-native-visual__notice-native-saved.tsv](tsv/actual/final-cross-native-visual__notice-native-saved.tsv) |
| final-cross-native-visual/notice-wasm-saved | 24 | 97.74% | [final-cross-native-visual__notice-wasm-saved.tsv](tsv/actual/final-cross-native-visual__notice-wasm-saved.tsv) |
| final-cross-wasm-visual/grow-nested-native-saved | 1 | 100.00% | [final-cross-wasm-visual__grow-nested-native-saved.tsv](tsv/actual/final-cross-wasm-visual__grow-nested-native-saved.tsv) |
| final-cross-wasm-visual/grow-nested-wasm-saved | 1 | 100.00% | [final-cross-wasm-visual__grow-nested-wasm-saved.tsv](tsv/actual/final-cross-wasm-visual__grow-nested-wasm-saved.tsv) |
| final-cross-wasm-visual/notice-native-saved | 24 | 97.46% | [final-cross-wasm-visual__notice-native-saved.tsv](tsv/actual/final-cross-wasm-visual__notice-native-saved.tsv) |
| final-cross-wasm-visual/notice-wasm-saved | 24 | 97.46% | [final-cross-wasm-visual__notice-wasm-saved.tsv](tsv/actual/final-cross-wasm-visual__notice-wasm-saved.tsv) |

notice의 53개 API 호출과 Native/WASM이 실제 저장한 파일을 양쪽 backend로 교차 재열기했다. 중첩 표도 동일하게 실제 편집·저장·교차 재열기를 실행했다. 원본 정적 파일 재출력만으로 편집 검증을 대신하지 않았다. 두 backend의 실제 notice·중첩 저장본은 각각 SHA-256까지 같아, 중복 사본 없이 두 입력을 [saved](saved/)에 보존했다.

중첩 저장의 common height는 편집 시 부모·자식 각각 11748 HU, 무편집은 각각 8502 HU다. 원본 cellSz·live IR을 바꾸지 않았고 HWP/HWPX 계약 검사에서 예산 거부·취소 뒤 원자성과 새 요청 복구도 통과했다.

## 최종 자동 검증

| 검사 | 결과 |
| --- | --- |
| 전체 stock release-test nextest | 12,465 PASS / 45 skip / 0 FAIL. 실행 543.480초 |
| 관련 공개 편집·기존 기능 보존 | 전체 실행 안에서 19개 모듈 78건 모두 PASS |
| clippy-root | exit 0, 47.36초 (제품 source 동일 39b4 결과 재사용) |
| clippy-wasm | exit 0, 45.50초 (제품 source 동일 39b4 결과 재사용) |
| release-build | exit 0, 1148.44초 (제품 source 동일 39b4 결과 재사용) |
| release-lib | exit 0, 1052.83초 (제품 source 동일 39b4 결과 재사용) |
| clippy-workspace | exit 0, 18.27초 |
| full-nextest | exit 0, 863.04초 |
| native-skia-lib | exit 0, 350.61초 |
| native-placeholder | exit 0, 314.87초 |
| native-direct-pdf | exit 0, 18.61초 |
| doc-tests | exit 0, 182.48초 |
| native-build | exit 0, 301.91초 |
| wasm-build | exit 0, 864.91초 |
| workspace-build | exit 0, 173.40초 |
| native-diagnostic-compile | exit 0, 1.20초 |
| Native Skia lib 실제 개수 | root 3,858 + workspace 182 = 4,040 PASS / 12 ignored / 0 FAIL |
| 일반 release lib 실제 개수 | root 3,801 + workspace 182 = 3,983 PASS / 12 ignored / 0 FAIL |
| Studio | TypeScript exit 0; 2,716 PASS / 2 skip / 0 FAIL |
| policy | fmt·suite manifest·unit tiers·Node policy 통과. 새 테스트 파일 없음, 2,002 sources / 8,567 test attrs |

최소 글꼴 환경·실제 공급은 [fonts.md](fonts.md), 새 입력 탐지는 [security.md](security.md), 선정·원 CI·충돌과 credit은 [intake.md](intake.md), 실행 명령은 [commands.md](commands.md)에 있다. 새 source가 없어 재사용한 release/root/WASM lint와 새 테스트 head에서 다시 실행한 workspace lint·전체 nextest를 구분했다.

첫 전체 실행의 samples 전용 환경변수 오류는 탐지 기준을 고치지 않고 바로잡았다. Native 개별 검사 첫 시도도 격리 환경에서 home의 Node가 가려져 시작 전 실패했다. Node 파일 하나만 read-only bind한 뒤 실제 두 검사를 실행했다. 두 실패는 제품 결함이나 통과 결과로 세지 않았다.

최종 Native binary SHA-256: `0b04561ea64706fdb5e8d7d1038212c2e818f0346b334b02f54c72e58603a892`. fresh WASM SHA-256: `c0a4372d5b5df15a55b6b7b5c53e333c420f488f9e3f40855876c6d9d434be0b`. 기존 배포용 `pkg/rhwp_bg.wasm`은 `19e4729f73daa5a183d37b29eb09500cdd217c112383f0d643b786fce59bb3c1`로 유지했다.

## 직접 판독과 한계

대표 패널과 재출력 대응은 [직접 판독 기록](direct-review.md)에 있다. 양식 16개의 한 번씩 paint·줄 경계, 다각형의 열린 변·꼭짓점·중심, 빈 문단 앞뒤 간격, 머리말의 원본 의도인 본문 겹침, 분할 셀 마지막 두 줄·뒤 본문, notice의 12·13쪽과 표 이어받기·꼬리, 늘어난 중첩 행과 부모 clip을 대조했다. wrap에서는 그림의 같은 세로 밴드와 글자 순서를 확인하고 Print의 계단형 sampling과 backend의 부드러운 보간을 구분했다. 이 변경 영역에는 수식이 없으며 괄호·별표·숫자·작은 글자·문단 간격을 판독했다.

100% raw도 동일 픽셀을 뜻하지 않는다. 정확한 관계 검사와 페이지별 고정 좌표 조건을 구분했다. polygon 90도·form-slot의 양쪽 100% 완료 쪽에는 독립 좌표 허용 오차를 유지했고, 나머지 새 검사는 소유·순서·포함·누락·중복 관계를 보호한다. 기존 정확한 기대값은 유지했다. split-row 16쪽의 이미 해소된 hidden-line 전제는 #8187 해결 이력을 확인하고 가시 내용·꼬리·클릭 소유 검사로 바꿨다.

특정 문서 ID·우연한 수치·장치 글꼴로 맞추는 분기는 추가하지 않았다. source-unit·Storage 예산·schema 35는 유지했다. 전체 검증과 실제 최적화 browser 실행의 근거를 원 기여의 옛 녹색 CI로 대신하지 않았다.

최종 근거 링크 갱신은 [제품·테스트 실행문 동일성](source-equivalence.md)으로 구분했다. 검증 후 구현을 변경하지 않았다.

최종 근거 주석·문서 정리 뒤 fmt, generated harness prepare/check, unit-tier policy와 workspace all-target Clippy를 다시 실행해 exit 0을 확인했다(Clippy 18.65초). generated harness·manifest는 stage하지 않았다. 문서 상대 링크 132개와 UTF-8 without BOM 32문서를 확인했고, 새 HWPX 17개에 글꼴 프로그램이 없음을 확인했다.
