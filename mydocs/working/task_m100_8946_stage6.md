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

### 사용자 제공 글꼴로 재검증한 결과

사용자가 `HBATANG.TTF`·`HMKMM.TTF`와 Windows의 `batang.ttc`·`gulim.ttc`를 제공하고 로컬 검증에만 사용하도록 승인했다.
원본은 변경하지 않았고 검증용 사본은 Git에서 제외된 T7 output에 두었다. 시스템 설치·글꼴 업로드·저장소 수록은 하지 않았다.
[한컴 글꼴 원장](../pr/assets/issue8946/stage6-20261112/user-font-audit.json)과
[Windows 글꼴 원장](../pr/assets/issue8946/stage6-20261112/windows-font-audit.json)에 SHA·출처·TTC face를 기록했다.

| 제공 파일 | SHA-256 앞 12자리 | 기존 기준과의 관계 |
| --- | --- | --- |
| HBATANG.TTF | bef6371830a14 | 기존 Mac 편람 검증 파일과 바이트 일치. #8896 Linux의 35f84328…와는 다른 버전 |
| HMKMM.TTF | d033df71fa41 | 기존 Mac 및 #8896 공급 파일과 바이트 일치 |
| batang.ttc | 84b0ba79a5d1 | 바탕·바탕체·궁서·궁서체. #8896 파일과 SHA 불일치 |
| gulim.ttc | 4b9ac63e8920 | 굴림·굴림체·돋움·돋움체. #8896 파일과 SHA 불일치 |

같은 가족명이나 Windows 출처를 프로그램 동일성으로 간주하지 않았다. 브라우저의 기존 sanitizer가 만든 사본도 원본 SHA와
구분한다. `HBATANG.TTF`는 기존 cmap 종단·bitmap 제거 처리를 거친 브라우저 사본을 사용하며, 원본 글리프를 목표 출력에 맞춰
편집하지 않았다.

편람의 초기 HWP 30·168·194·258쪽, HWPX 258쪽 잔차는 이전 공개 환경이 HCRBatang을 명시 대체로 고정한 데서 남았다.
실제 제공 프로그램을 기존 `preparedPrograms` 경로에서 선택하게 하자 다섯 쪽이 모두 100.00%로 복원됐다.
HWP/HWPX fresh WASM 전 383+383쪽은 각각 최저 99.96%(130쪽의 기존 WMF), 나머지 382쪽은 100.00%이며 보호값 하락은 0이다.
괄호·가운뎃점의 이전/현재/독립 Print 확대 이미지와 review를 [글꼴 판독 증거](../pr/assets/issue8946/stage6-20261112/user-font-reviews/)에 보존했다.
이미 통과한 Native 편람 결과와 함께 편람의 네 데이터셋은 모두 보호 검증을 통과한다.

### 정본 공급 조건 복원과 독립 수식 글꼴

#8958의 원 검증과 #8896/#8945의 최종 보호 증거를 대조했다. 초기 탐색에서 387개 HFT 전체 폴더는 Native의 256파일 상한을
넘으므로 최종 공급으로 채택하지 않았다. 선택한 파일을 하나씩 별도 경로로 공급한 시도도 Native의
`TypefaceCatalog::load`가 **첫 경로의 완전한 가족 스타일 집합을 우선**하는 계약과 달랐다. 영문 파일이 같은 가족의 한글 파일을
가리는 실제 제목 차이를 확인했다. #8896처럼 선택된 HFT 언어·스타일 파일을 하나의 제한된 `00-hft` 폴더에 묶고,
한컴·Windows 공급 폴더를 구분했다. 파일 수는 문서별 24~77개다. 제품의 상한·글꼴 메트릭·선택 로직을 바꾸지 않았다.
[공급 원장](../pr/assets/issue8946/stage6-20261112/protected-grouped-supply.json)에 선택 파일과 SHA를 연결했다.

법령·개요·중첩 표에는 최신 정본이 사용한 #8945/#8896의 독립 Print 글꼴 환경도 복원했다.
법령의 ‘한컴바탕 → Batang’처럼 정본에 명시된 선택을 이전 #8958의 자동 환경으로 대신하지 않았다.
새 가족별 공급에서 법령·개요·삽입/삭제 편집 표본은 Native/WASM 전쪽 보호값 하락이 모두 0이다.

사용자는 생성된 **독립 수식 표본 HWPX만** 기존 Hancom MCP에 보내는 것을 명시 승인했다.
한컴 2020 `11.0.0.9136`, method 0, one-up Print를 받아 canonical 준비 도구로 로컬 수식 글꼴을 복원했다.
77개 원 글리프 해시는 #8896의 독립 arithmetic v2 증거와 모두 같다. OTF 전체 파일 SHA는 다르므로 파일 동일성으로 표현하지 않는다.
검증 대상 문서의 PDF를 글꼴 공급원으로 사용하지 않았으며 이동·확대·윤곽 보정도 하지 않았다.
[독립 수식 출처](../pr/assets/issue8946/stage6-20261112/legacy-equation-local-provenance.json)에 입력·Print·글리프 해시를 연결했다.
사용자가 준 TTF/TTC, 복원한 OTF, 표본 PDF와 서비스 인증 로그는 저장소에 넣지 않았다.

이 수식 프로그램 공급으로 sample16 HWPX 6쪽은 Native 99.95% → 100.00%, WASM 99.93% → 99.99%로 개선됐다.
분수의 괄호·분자/분모와 43쪽 곱셈·나눗셈을 독립 Print와 직접 대조했다. 43쪽은 양쪽 100.00%였지만
그 점수를 전체 수식 문서 통과로 확대하지 않는다. Windows Gulim 공급 후 WMF 18·35쪽의 큰 차이도 해소됐으며,
공급 전 두 쪽의 base/후보 PNG가 바이트 단위로 같았던 진단은 별도로 보존했다.

### 현재 남은 항목

[전쪽 색인](../pr/assets/issue8946/stage6-20261112/protected-index.json)은 18입력 × 두 backend의 **3,542쪽 전부**를 포함한다.
전체 페이지의 raw는 90% 이상이지만, 변경하지 않은 공식 보호값보다 낮은 **35개 문서/backend/쪽 행**이 남아
`re_review_required`다. 페이지별 최댓값을 조합하지 않고 공급 계약을 복원한 데이터셋 전체를 선택했다.
이전 미달 실행은 로컬 진단 기록에 남겼다.

| 문서 | Native 보호값 하락 | fresh WASM 보호값 하락 |
| --- | ---: | ---: |
| sample16 HWPX | 3쪽 | 4쪽 |
| sample16 HWP 2024 | 3쪽 | 4쪽 |
| sample16 HWP 2022 | 7쪽 | 8쪽 |
| 중첩 표 | 1쪽 | 1쪽 |
| sample10 HWP3 | 2쪽 | 2쪽 |

중첩 표 4쪽의 독립 Print는 제목에 `H2hdrM`, 화살표에 Haansoft Batang을 사용한다.
현재 공급에는 `H2HDRM.TTF`가 없으며 실제 제목 차이를 확인해 Windows 보관본을 요청했다.
화살표는 `굴림체/GulimChe` 요청이고 현재 명시 환경의 `Gulim` 문자 대체와 별개다. 제공된 HBATANG의 같은 코드포인트와
Print 글리프의 전진은 같으나 윤곽은 완전히 같지 않았다. 따라서 동일 프로그램 복원으로 판정하지 않았고 기준이나 기대값도 바꾸지 않았다.
sample16의 남은 본문 잔차는 줄 오른쪽 한글의 미세 위치를 포함한다.
[잔차 확대 이미지](../pr/assets/issue8946/stage6-20261112/remaining-font-reviews/)와 실제 좌표를 보존했다.

Windows Fonts 전체 백업을 받으면 필요한 원 프로그램·버전을 확인하고 남은 원인별 집중 검증과 전쪽 검증을 이어간다.
제품 코드·회귀 기대값·정본 TSV·허용폭은 변경하지 않았다. 필수 보호 검증을 완료한 뒤에만 증적 상태를 확정하고,
최신 열린 PR·base 및 병합 가능 여부를 재확인해 승인된 push·Open PR·CI 확인으로 진행한다.
