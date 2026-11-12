# #8946 최신 devel 리베이스와 제출 전 재검증

2026-11-12 KST. #8946의 Canvas2D 명시적 호스트 원본 프로그램 선택·환경 갱신·수명 처리 범위를 유지한다.
수용 조건과 구현·회귀의 연결은 [단계 5](task_m100_8946_stage5.md)를 따른다. 이 기록은 최신 base 검증을 갱신한다.

현재 **PR 제출 보류**다. 코드·일반 로컬 검증은 완료했지만 collaborator의 정본 보호 검증 전체는 아직 완료하지 않았다.
원격 push·PR 생성·CI 시작을 완료한 것으로 기록하지 않는다. 미완료 항목은 아래에 구분한다.

## 기준과 생산자

- 이전 base `4c876654d86d071d342fc78706c51cfff6ddbde4`에서 최신 `80b1c354c29b7a02fb2b3c14a274da7f5b653ada`로 18개 커밋을 리베이스했다.
- 코드 head: `b4855616375c4a8fd1cdd6083d85cd77d655ad26`. 이 단계에서 제품 소스·회귀 기대값은 추가 변경하지 않았다.
- 새 devel의 #8958, #8961, #8984/#8896을 확인했다. 이슈 본문의 수정 시각은 `2026-11-11T10:17:17Z`로 유지됐다.
- base와 후보 Native/fresh WASM을 따로 빌드하고 각 바이너리를 고정했다. WASM은 최적화된 locked wrapper 결과이며 Chrome `157.0.8103.51`을 양쪽에 사용했다.
- [생산자 원장](../pr/assets/issue8946/stage6-20261112/producer.json)에 소스 Git object·바이너리·초기 자원 SHA를 연결했다. T7에서 빌드·검증했고 공유 Cargo target을 사용했다.
- 본인 열린 PR은 조회 시 0개다. collaborator write 권한을 확인했으므로 제출 시 원본 저장소의 작업 브랜치 경로를 따른다. merge는 이번 승인 범위에 포함하지 않는다.

## 최종 로컬 결과

| 검사 | 실제 결과 |
| --- | --- |
| 최소 글꼴 전체 Rust 회귀 | 13,320 PASS, 0 FAIL, suite skip 45 |
| Native Skia library | 3,852 + 15 + 165 + 2 PASS, ignore 12 |
| Native Skia placeholder / direct PDF | 2 PASS / 36 PASS |
| fmt·Native/WASM/workspace all-target Clippy·workspace build | 모두 PASS, Clippy `-D warnings` |
| suite manifest·unit-tier 정책 | 새 base SHA를 지정해 PASS |
| TypeScript·Studio·frontend 계약 | PASS / 2,789 PASS, 기존 skip 2 / 3 PASS |
| E2E manifest·fresh WASM #7403 | 190개 일치 / Canvas2D·CanvasKit PASS, 조판 계약 97개 모두 true |
| 필수 편람 HWPX Native gate | 383쪽 passed, 보호값 하락 0쪽 |

[실행 결과 원장](../pr/assets/issue8946/stage6-20261112/local-verification-results.json)에 로그 SHA와 실제 결과를 보존했다.
최소 환경에서 개인 공급 프로그램 71개는 모두 읽기 차단됐다. OS·공개 파일 394개의 읽기 확인은 전체 허용 목록의 개수와 다르며,
라이선스 확인 fixture의 정확한 파일 허용은 별도로 유지했다. 실제 시각 측정의 명시적 글꼴 공급과 회귀 격리를 혼동하지 않는다.

## 같은 입력의 전쪽 독립 Print 대조

| 입력·각 763쪽 | Native raw 최저 | fresh WASM raw 최저 | 실제 Host Canvas2D raw 최저 |
| --- | ---: | ---: | ---: |
| HWPX | 96.34886% | 99.56218% | 99.35730% |
| HWP3 | 98.77175% | 99.54840% | 99.35495% |

그림 세 영역을 제외하지 않았다. 모든 경로에서 독립 PDF와 763쪽이 일치하고 90% 미달은 0쪽이다.
동일 그림 공급·운반본 동등성·한컴 2024 Print의 출처는 [단계 4](task_m100_8946_stage4.md)를 재사용한다.
90,804,612바이트 HWPX 기준 PDF는 50 MiB 이상 PDF 규칙에 따라 로컬 정본으로 보관하며 원본 SHA와 전쪽 증적을 제출한다.
HWP3의 독립 한컴 2020 Print는 기존 tracked PDF를 재사용했다.

최종 네 Native/WASM 출력 묶음은 단계 5의 각 763쪽 PNG·render tree와 바이트 단위로 동일하다.
실제 Host 두 입력의 전쪽 PNG·tree도 동일하다. 따라서 이전 표지·본문 제목 23곳·350/608/686쪽 문장부호·729쪽과
HWP3 완료 페이지의 확대 판독을 출력 동일성이 입증된 범위에서 재사용한다. 이번에도 표지와 350쪽의 review·overlay,
실제 편집 화면을 직접 확인했다. raw 100%를 RGB 완전 동일로 해석하지 않는다.

[devel 전쪽 비교](../pr/assets/issue8946/stage6-20261112/devel-full-comparison.tsv)는 동일 입력/PDF SHA·실제 프로그램·compat·96 DPI에서
Native와 WASM을 각각 비교한 3,052행이다. 원래 정밀 raw 값을 Decimal로 계산했다.

| 입력·경로 | devel 최저 | 후보 최저 | devel 90% 미달 | 후보 90% 미달 | 최대 하락폭 |
| --- | ---: | ---: | ---: | ---: | ---: |
| HWPX Native | 85.45580% | 96.34886% | 18쪽 | 0쪽 | 0%p |
| HWPX WASM | 85.44688% | 99.56218% | 18쪽 | 0쪽 | 0%p |
| HWP3 Native | 98.77175% | 98.77175% | 0쪽 | 0쪽 | 0%p |
| HWP3 WASM | 99.54840% | 99.54840% | 0쪽 | 0쪽 | 0%p |

이 비교는 별도의 정본 보호 하한 통과를 대신하지 않는다.

## 기본 동작·편집·리베이스 기능 보존

원본 HWPX·그림 포함 HWPX·HWP3 × 공급자 없음/옵션 생략/명시적 false의 9조건에서 base/후보 각 763쪽 PNG·tree와 첫 화면이 같다.
HWP3의 omitted/false 최초 전체 tree 수집은 본문 글꼴의 지연 준비 중 일부 쪽이 달랐다. PNG는 처음부터 같았다.
첫 화면 기록을 유지한 뒤 전쪽 글꼴 요청·pending 0·메트릭 및 document.fonts 준비를 기다려 각각 독립 두 번 수집했다.
네 base/head 재실행의 763쪽 두 수집이 모두 같고, base/후보도 같았다. 초기 차이는 `default-initial-readiness.json`에 남겼다.
제품 보정이나 최초 수집 성공으로 표시하지 않는다.

실제 HWPX/HWP3에 `검증`을 삽입하고 삭제했으며 저장 후 재열기도 실행했다. 문단 소유·내용 순서·누락/중복 없음·본문 포함·
줄 겹침 없음·빈 문단 점유·후속 본문은 두 입력의 8개 관계 검사 모두 통과했다.
프로그램 원본 SHA와 faceIndex 및 indexed program ID가 첫 준비·측정·paint에 연결되는 초기 trace도 다시 확인했다.

#8958의 원 승인 범위는 44조건·88개 기능 주장 페이지와 문서 전체 쪽수다. 새 base/후보 Native/WASM에서 모두 재실행했고,
최저 raw 90.79%, PNG·render tree는 모두 동일하다. 후보 176개 페이지 이미지는 기존 승인 증적과 RGB가 같다.
이를 원 PR 범위 밖의 모든 페이지 개선 주장으로 확대하지 않는다. #8984/#8896·#8945의 표 종료·이어받기 8입력·14쪽도
양 backend 100.00%, base/후보 PNG·tree가 동일하다.

재측정 중 발견한 두 환경 문제는 제품 코드를 바꾸지 않고 원래 공급 조건을 복원했다.

- WASM에서 Native의 명시 환경과 OS 프로그램이 빠지자 pr8788 p3가 77.73%였다. 원 승인 Mac의 실제 프로그램 SHA 96개를 맞춘 뒤 99.20%로 복원됐다. Native는 99.22%다.
- pr8930 p1의 제목은 raw 100.00%여도 잘못된 고딕 모양이었다. 원본 charPr 9/font 5의 ‘나눔고딕 Bold’는 ‘한컴바탕’을 대체 글꼴로 저장했고, 독립 PDF도 Haansoft Batang을 사용했다. 공급 웹폰트의 cmap 91개에는 제목 한글이 없었다. 기존 승인 환경의 7개 언어 슬롯별 HCRBatang glyph fallback을 복원해 base/head 두 backend의 2쪽 전체를 재실행했다. 제목의 명조 형태와 간격을 직접 확인했다. WASM의 기존 굵기 잔차는 base 및 승인 증적과 같으며 이번 신규 후퇴로 분류하지 않는다.

## 남은 제출 조건: 정본 보호 전체

[정본 보호 TSV](../pr/assets/visual-protected-baseline/final.tsv)의 SHA는
`ee41455cf7f05585487d92961ccb9fd8160d5d1e15c3567019ea89e2b7bf2aeb`다. 18입력 × Native/WASM, 총 3,542쪽의 공식 두 자리 값 하락 금지 조건을 별도로 적용한다.
현재 진행 상태와 하락·누락 목록은 [보호 비교 원장](../pr/assets/issue8946/stage6-20261112/protected-comparison.json)과
[제출 준비 상태](../pr/assets/issue8946/stage6-20261112/submission-readiness.json)를 따른다. 전쪽 완료 전에는 통과로 판정하지 않는다.

기존 Mac 검증의 `HBATANG.TTF`(`bef63718…`)·`HMKMM.TTF`(`d033df71…`)를 로컬 글꼴·다운로드·T7 자료에서 찾지 못해 보관 경로를 요청했다.
최신 #8896의 Linux 공급 목록에는 Windows Gulim/Batang·Courier/Times 등 추가 프로그램도 있다. 이름만 같은 Mac 파일로 동일 프로그램이라고 간주하지 않는다.
기존 정본·메인터너 보정·독립 Print의 조건을 추적하여 필요한 실제 프로그램을 복원하고 전쪽 재측정해야 한다.

공개 환경 대조에서 Helvetica WASM의 초기 96.46%는 Native가 선택한 OS collection을 공급하지 않은 결과였다.
같은 Helvetica.ttc와 Native 환경을 명시한 재실행은 99.25%로 보호값 98.79% 이상이다.
편람 추가 WASM 비교는 HWP 30·168·194·258쪽, HWPX 258쪽의 공식 값이 100.00%에서 99.99%로 내려갔다.
Times New Roman을 같은 파일로 추가 공급해도 값은 그대로여서 해당 가설은 제외했다. 다섯 쪽의 후보와 최신 base PNG는 모두 바이트 단위로 같다.
확대 판독과 PDF의 실제 font 객체를 대조해 반각 괄호 `｢｣`와 가운뎃점 `･`의 Haansoft Batang 공급 차이로 범위를 좁혔다.
각 쪽의 미대응은 2~4픽셀이지만 정본 보호 기준의 예외로 처리하지 않는다. 원본 파일 확보 후 실제 재검증이 필요하다.

사용자가 추가로 지정한 `$T7/hft-fonts-otf`도 모든 387개 파일의 SHA와 family·PostScript 이름을 조사했다.
전부 OTF이고 Haansoft Batang 및 위 두 TTF는 없다. ‘휴먼명조’의 `HMKSM.otf`는 SHA `99760ec2…`의 다른 프로그램이다.
별도 TTF/TTC 폴더의 보관 경로를 요청했다. 현재 보호 측정은 1,544/3,542쪽이며 나머지 20개 데이터셋과 위 5쪽 잔차가 남았다.
제품 기대값·정본 보호 TSV·허용폭을 낮추지 않았다.

이 조건을 완료한 뒤 증적 상태를 확정하고, 최신 열린 PR·base 및 병합 가능 여부를 재확인해 승인된 push·Open PR·CI 확인으로 진행한다.
