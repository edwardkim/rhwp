# PR #8658 최종 로컬 검증

검증 제품 code는 `19de2f3b54`, 회귀 code는 `08cd5ab833`이다. 최종 증적·검토 문서는 제품 및 회귀 assertion을 바꾸지 않는다. 기준은 `08963f6825956984d38b0d9392f44777db7107ab`이며 모든 실제 사용 입력·PDF는 Git blob과 일치한다.

| 검사 | 실제 결과 |
| --- | --- |
| fmt·manifest prepare/check·manifest 계약·unit-tier | 전부 exit 0 |
| 최소 글꼴 집중 회귀 | 새 경계 포함 8 PASS |
| 선행 #3386 / #7160 대조 | 1 / 3 PASS, 기대값 불변 |
| 최소 글꼴 전체 nextest | 12,760 PASS, 기존 45 skipped, 실패 0 |
| Native Skia 공식 lib | 3,858 + 15 + 165 + 2 = 4,040 PASS, 기존 ignored 12 |
| Native placeholder / direct PDF | 2 / 22 PASS, 공식 3종 합계 4,064 |
| Native·WASM32·workspace all-target Clippy·workspace build | 전부 exit 0 |
| 원 사례·경계 전 10쪽 Native/fresh optimized WASM | 두 경로 각각 최저 100.00%, 쪽수 일치·직접 판독 |
| 전체 36입력 77쪽 Native/fresh WASM | 최저 91.16% / 91.83%, 90% 미달·누락 0 |
| 최신 devel 기능 보존 | Native 67 / WASM 67개 PNG 쌍이 각각 실제 base와 동일 |
| 실제 SVG browser 소비 | 10쪽, 최저 99.98% |
| 실제 Studio PageRenderer | Canvas2D·CanvasKit compat/default software, screen/print 24개 캡처, backend 불일치·fallback 0 |
| Studio Print 독립 비교 | 실제 화면 12개 모두 raw 100.00% |
| 원본 글꼴 추가 확인 | 선언 높이 경계의 실제 Studio 6개 캡처, Print 3개 raw 100.00% |
| Native 편람 | 383쪽, 최저 99.96%, 기준 하락 0쪽 |
| 새 입력 보안 | 공개 입력 8개 × 3종 = 24건 clean |

## 실행 환경과 출처

Cargo/WASM은 공용 절대 target `/home/edward/mygithub/rhwp/target/pr-review`에서 순차 실행했다. 현재 호스트는 논리 CPU 16개·RAM 약 31GiB다. Cargo jobs 4, 전체 nextest 동시성 12를 사용했다. 최소 글꼴은 bwrap에서 시스템·사용자 설치 글꼴을 차단하고 공개 글꼴 5개만 Fontconfig에 공급했다. 실제 `fc-list`와 private font 경로 부재를 각 실행에서 확인했으며 폰트가 없다는 이유로 새 회귀를 skip하지 않았다. Node 실행 파일만 읽기 전용으로 노출했다.

nextest 0.9.137의 권장 버전 경고와 JUnit의 report-skipped 미지원 경고는 그대로 기록했다. 실제 검사는 전부 실행됐고 exit 0이다. 원 실행 로그는 기본 작업공간의 ignored `output/pr-review/pr8658/final-*.log`, 실제 명령·종료 코드는 [실행 대장](local-validation-results.json)에 있다. 로그를 소스나 증적 경로에 커밋하지 않는다.

Native/Sweep에는 실제 함초롬바탕·함초롬돋움 프로그램을 명시했다. 8572 대조군에는 선행 검토에서 확인한 원본 NanumGothic·Bold와 HCRDotum 공급을 재사용했다. 글꼴 프로그램과 내장 글꼴 SVG는 검증 전용이며 커밋하지 않는다. Native 편람의 실제 추가 웹폰트·프로필도 gate 대장에 남긴다.

## 실제 Studio와 RGB 후보 판독

정본 renderer-baseline의 capture·readiness·backend identity·parity 계산을 사용했다. tests/fixtures 입력을 읽기 위해 검증 전용 loader만 저장소 경계 안의 실제 바이트 로드로 바꿨다. 문서 SHA와 실제 PageRenderer의 page 사각형 전체를 확인했고, 본문·앞뒤 레이어·DOM 그림을 포함한다. 임계값·paint·renderer source는 바꾸지 않았다.

원격 Canvas2D API만으로 Studio 화면을 대신하지 않는다. [실제 화면·runtime](studio/browser-baseline-report.json)에서 24개 캡처의 요청/실제 backend와 profile이 일치하고 fallback은 없다. CanvasKit은 software surface이며, GPU 검증으로 표시하지 않는다. 초기화 때 받은 JS/WASM은 24회 모두 현재 빌드와 연결된다. Vite가 JS의 WASM URL 한 곳을 변환하므로 [정확한 URL 변환과 초기 응답 해시](studio/studio-consumption-proof.json)로 원본 glue의 전 바이트를 확인했다. 뒤의 별도 fetch만으로 초기화를 입증하지 않았다.

RGB report-only parity 16개 중 Print 8개는 모두 통과, screen 8개 중 5개는 기존 기준 0.5%를 넘는 후보(최대 약 0.90%)다. 이를 숨기거나 허용치를 바꾸지 않았다. 확대 판독에서 같은 그림·괘선의 경계 래스터와 screen 가이드 차이를 확인했고 칸 소속·그림 간격·표 높이·후속 문단은 일치한다. [Print 독립 비교 12개](studio/studio-independent-results.json)는 모두 raw 100.00%다. 자동 RGB 보조값과 사람 판독을 구분한다.

END의 원본 선언은 함초롬바탕이다. 기본 웹 공급과 로컬 원본 프로그램의 차이를 구분하기 위해 실제 원본 SHA `ba2ac8f2b1b9f45c310637565ac46c2a39982241f9c78d4a2668678a4f88ee85`를 공급했다. [공급·등록 영수증](studio/original-font/font-proof.json)과 [별도 캡처·초기 bundle 검증](studio/original-font/identity-and-results.json)을 보존했다. Canvas2D의 차이는 END 일부의 37개 래스터 픽셀에 한정됐고 배치는 그대로다. CanvasKit은 실제 bundled Typeface 경로를 사용하므로 등록 성공만으로 원본 프로그램을 paint에 사용했다고 주장하지 않는다. [같은 배율의 END 판독](studio/original-font/end-font-zoom.png)과 실제 Print 3개 raw 100.00%를 함께 확인했다. 이번 PR은 글꼴 선택 정책을 바꾸지 않는다.

## 제외한 진단과 재검증

초기 기준 WASM의 공유 캐시 재사용, 최상위 종료 143인 이전 진단 빌드, Node 미노출로 127인 테스트 시작 전 실행은 최종 PASS로 집계하지 않았다. 실제 기준 패키지 재컴파일·최적화와 원래 결함 재현, 후보 exit 0 최적화 빌드, 최소 글꼴 8 PASS를 다시 확인했다. 실제 source/producer 해시·초기 소비·정상 대조를 별도 영수증에 연결한다.

JS identity의 처음 파생 검사기는 Vite URL 변환 전 원문을 기대해 캡처 뒤 exit 1이었다. 캡처·실제 WASM 소비는 완료됐으며, 기록된 초기 JS 응답 24개의 SHA와 동일한 모듈을 복원해 URL 변환 정확히 한 곳 외의 모든 소스 바이트를 검증한 독립 검사 exit 0으로 출처를 확정했다. 원 실패 상태를 0으로 덮지 않았다.

## 원격 경계

이 결과는 로컬 수용 검증이다. 정확한 최종 제출 head와 원격 처리안에 대한 승인 뒤 upstream 수용 PR을 생성하고, 해당 새 head GitHub CI·self-review·merge를 진행한다. 원 fork에는 아직 push하지 않았고 공개 댓글·새 PR·merge도 수행하지 않았다.
