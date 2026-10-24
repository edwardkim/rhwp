# Stage107: 기존 범위의 최종 로컬 검증과 제출 증적

Issue: #7445. 제품 실행 HEAD는 `1f54c3df17747b71c8be9b47b3198add4fa4fd3b`,
base는 `d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`이다. 이후 Rust 차이는
[Stage101](task_m100_7445_rebase_repair_stage101.md)의 진단 속성 두 개뿐이다.
HWP 보정 대상을 확대하지 않았다. #7445 전체 해결이나 과거 code head의 통과를 주장하지 않는다.

## 전쪽 출력과 입력 감사

고정한 기존 50조건의 Native/fresh WASM을 각각 1,836쪽, 총 3,672쪽 비교했다.
후보 100조건은 모두 새 캡처이며 각 조건의 독립 Print 쪽수와 일치한다. raw 2px·90% 미달,
누락·중복 쪽과 같은 수치 쪽의 devel 대비 2%p 이상 하락은 각각 0건이다.

| 경로 | 조건 | 전체 쪽 | 최저 raw |
| --- | ---: | ---: | ---: |
| Native Skia | 50 | 1,836 | 93.06876% |
| fresh WASM Canvas | 50 | 1,836 | 92.99291% |

둘의 최저는 rowbreak18의16쪽이다. 같은 base 쪽은 Native87.46657%, WASM92.01257%다.
상단 이어받기·본문·하단 표/인용 상자·쪽 번호의 소속과 내용은 보존되며 얇은 선·미세 위치·획
차이가 남는다. 점수 통과를 엄격 RGB 또는 절대 좌표 동일성으로 확대하지 않는다.

base 100 TSV는 실제 같은 d9fd 바이너리·도구·글꼴·브라우저 신원을 대조한 stage83의 증거다.
후보 출력으로 재표기하지 않았다. base 쪽수가 다른 로드맵·low-slack·보도자료·court·분할 표의
미대응 쪽 목록도 보존한다. 같은 수치 쪽의 비교가 내용 대응을 입증하지 않으므로 후보 독립
쪽수와 내용 관계 회귀·확대 판독을 함께 확인했다.

실제 HWP/HWPX/PDF의 바이트가 커밋한 파일과 같은지 SHA로 확인했고, TSV 전쪽 순서와
PNG 쪽 집합을 별도로 대조했다. base의 미대응 쪽을 캡처 누락으로 오인했던 초기 감사 초안은
정정했으며 최종 감사는 후보 전체 범위와 base의 실제 대응 범위를 구분한다.

PDF 변환은 실제 PDFium152.0.7947.0/pypdfium2 5.12.1 smooth RGB v1, 96DPI다.
Mac fresh WASM은 locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며
Docker 최적화 통과가 아니다. 기존 사용자 글꼴·원본·PDF·공유 target을 보존했다.

## 확대 판독과 실제 소비

전쪽 결과의 대표38개 조건·쪽·backend review와 overlay를 직접 확인했다. sample11의 목차
5.2와 가로선/쪽 번호·연결 그림, 원본 sample16의 수식/dot/표, 로드맵 마지막 상자, 개인정보
마지막 조각과 독성물질 표/그래프를 포함한다. #4138의 두 분할 저장본은 각각197쪽 전쪽 비교에
더해1·10·31·197쪽의 줄 소속·끝 경계·내용 순서를 확인했다. 원본115쪽 검사도 유지한다.

sample11의 회색/갈색 면은 사용자 UI Print에도 존재하며 편집기 사진과 다른 인쇄 잔여다.
기존 독립 font-isolation 대조를 유지하고 임의 도형 추가·mask로 맞추지 않았다. sample16의
괄호·원 외곽선·안티앨리어싱, 다른 문서의 얇은 괘선/획 차이도 잔여로 기록했다.

| 실제 제품 진입점 | 독립 PDF 관측 | 90% 미달 |
| --- | ---: | ---: |
| CLI Native Skia PNG | 31 | 0 |
| Studio 합성 Canvas2D 페이지 | 31 | 0 |
| 지원 CanvasKit compat/default | 12 | 0 |

기존 비트맵3개와 일반 보존 대조군을 포함한74관측이며50조건의 HWP 보정을 확대하지 않았다.
초기화 때 받은 JS/WASM 응답과 root pkg·Studio/public을 대조했다. JS는 Vite의 정확한 asset URL
rewrite와 원 source-map 전체, WASM은 원 바이트 SHA로 검증했다. 초기화 뒤의 별도 fetch만으로
대신하지 않았다. 실제 backend·software surface·readiness·본문 canvas 소유·그림 준비를 확인했다.
Native23관측은 전쪽 PNG와 전체 padded RGB도 같았다. 실제 지원 관측은 서로 독립 캡처했고,
같은 크기·RGB SHA의 두 CK 모드는 대표 판독만 재사용했다. backend 성공은 각 관측에서 확인했다.

글꼴 준비 부족으로 Canvas2D로 fallback한10관측은 CK 통과가 아니다. 비트맵 fixture를 Vite
samples URL로 잘못 요청한9관측은 입력 SHA guard가 거절했다. 같은 커밋 바이트를 기존 제품
문서 열기 이벤트로 전달한 새 실행으로 정정했으며 원 실패를 보존했다. source·글꼴·gate를
바꾸지 않았다. 기존 C352/C354/C356의 실제 Native·Canvas2D·CK 출력은 모두 raw100%다.
엄격 픽셀 지표의 미세 계조/1px 차이와 raw100%는 구분한다.

## 로컬 실행 결과

아래 묶음은 중복 검사가 있으므로 건수를 합산한 고유 테스트 수로 보고하지 않는다.

| 검증 | 실제 결과 |
| --- | --- |
| 전체 release-test/native-skia nextest | 11,756 PASS / 0 FAIL / 45 skip, 실행1,112.661초 |
| Native Skia lib | root3,859 PASS / 12 ignore, 다른 libs182 PASS |
| 실제 Native placeholder / PDF | 2 PASS / 14 PASS |
| release lib | root3,802 PASS / 12 ignore, 다른 libs182 PASS |
| doc tests | 8 PASS / 3 ignore |
| Studio/editor/shared PDF 단위 검사 | 2,409 PASS / 0 FAIL / 2 skip |
| fmt·Native/WASM/workspace/Skia all-target Clippy | 모두 PASS |
| workspace·release·Studio·Chrome·Firefox build, VSCode compile, TypeScript | 모두 PASS |
| WASM binding/editor/font 계약, 실제 browser 글꼴/WMF 계약 | 11 PASS / 10 PASS |
| 최종 진단 보정 뒤 기존 두 usvg 소비자 회귀 | 21 PASS / 0 FAIL |
| 글꼴 trace/schema/coverage | 25 PASS / 0 FAIL / 선택 비공개 POC1 skip |
| 시각 Python 계약 / HFT 준비·보안 계약 | 191 PASS / 32 PASS |
| workflow Python / routing Node / actionlint | 69 PASS / 95 PASS / exit0 |

초기 nextest의26실패를 제거하거나 허용하지 않고 [Stage90](task_m100_7445_rebase_repair_stage90.md)과
[Stage93](task_m100_7445_rebase_repair_stage93.md)의 공통 규칙으로 복구했다. 초기 Clippy 두 실패는
Stage101의 타입/진단 계약으로 처리했다. 없는 Chrome/Firefox 의존성 때문에 실패한 최초 패키징은
소유한 빈 node_modules에 locked npm ci를 수행한 뒤 실제 build를 재실행했다. 다른 작업의 의존성이나
lockfile을 덮어쓰지 않았다.

Gym의 변경은 기존7개 task가 보존한 동일 입력의 새 경로를 가리키는 것뿐이다. base의 옛 경로와
현재 경로의 파일 바이트, instructions/input을 제외한 채점 속성이 같다. 관련 단위 계약은
총3,170건 중3,169 PASS/1skip, 구조 감사는21pack·949task/949reference·오류0이다. 양성·음성·경로
전수 벤치마크의 성공이나 제품 시각 정답으로 보고하지 않는다.

## 보존과 다음 원격 단계

[최종 증적](../pr/assets/issue7445/completed-20261009/final-20261024/README.md)에
base/후보200 TSV, 대표 review/overlay, 입력·생산자·실행·실제 소비 원장을 보존한다.
원 로그·전체 PNG·진단 script는 ignored output의 stage95~106에 있다. 새 `.log`나 중간 파일을
커밋하지 않았다. 종료한 소유 Chromium 프로필3개만 manifest 뒤 정리해578,953,216바이트를
확보했고 PNG·TSV·실행 로그·원본·PDF·공유 target은 보존했다.

제출 전 base 대비 전체 `git diff --check`에서 과거 보존 TSV36개의 CRLF와 OFL 텍스트의
줄 끝 공백1건을 확인했다. 저장소의 과거 TSV 사본만 LF로 정리하고 모든 셀 값·행 순서가
같음을 CSV 파서로 대조했다. 최종 전쪽200 TSV와 원 실행 output은 바꾸지 않았다. OFL의
단어·순서도 같다. `final-20261024/historical-tsv-curation.json`에 원 commit·원 바이트 SHA와
정리한 사본 SHA를 연결했으며 과거 원장 hash를 새 사본의 바이트 hash라고 재표기하지 않는다.

사용자 초안은 기존 보존 stash에 남기며 소스 커밋에 섞지 않는다. 새 지침을 수정하지 않았다.
기존2%p 지침 변경은 사용자 승인 커밋 `4a03cec05c`의 기록이다.
커밋한 후보의 source packaging preflight, 최신 upstream/open PR을 확인한 뒤 승인된 PR을 진행한다.
code head CI 성공 전에는 번호를 추측한 PR review·오늘할일을 만들지 않으며, CI 성공 뒤 같은 PR에
문서 trailing commit을 추가하고 새 head CI·merge·후속 처리를 확인한다. 이 보고서는 아직 실행하지
않은 GitHub CI·merge·후속 처리를 완료했다고 주장하지 않는다.
