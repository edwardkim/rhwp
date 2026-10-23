# #8183 3단계 — 실제 소비 검증과 사용자 가이드

Issue: [#8183](https://github.com/edwardkim/rhwp/issues/8183) ·
[수행 계획](../plans/task_m100_8183.md) · [2단계 결과](task_m100_8183_stage2.md)

2026-10-23 작업지시자가 「3단계 진행 승인」으로 실제 CLI·fresh WASM/Studio 검증,
시각·성능 확인과 빌드·사용자 안내 작성을 승인했다. 기준 source는 `b799b1f47`이다.
원격 게시·push·PR 생성은 이번 승인에 포함하지 않는다.

상태: **진행 중. 아래 항목은 완료 검증 결과가 아니다.**

## 검증 범위

- default/JBIG2/JPEG 2000/양쪽의 실제 Native·WASM 산출물과 공통 지원 정보 일치.
- CLI 가져오기·저장·재열기, Rust 공개 API, Studio Worker의 오류·취소·다음 요청.
- 정상 입력과 mask·palette·alpha·공유 사전, 전체 페이지 TSV와 직접 판독.
- 동일 조건의 시간·메모리 관측·산출물 크기, 기존 대조군과 필수 lint·회귀.
- 실제 실행 결과에 맞춘 선택 빌드·설치·진단 가이드.

임시 증적은 `output/pr-review/issue8183/stage3/`에 둔다. 공용 target과 8개 Cargo 잡을
사용하고 Cargo 빌드는 순차 실행한다. 시작 시 가용 디스크는 17 GiB였다.

## 진행 중 관측

제품 지원 정보와 초기 가이드는 `a4039329a`에 커밋했다. 공통 `pdf_support`의 세 boolean을
CLI `capabilities`·가져오기 metadata·WASM 사전 조회 API가 소비한다. 기존 필드·오류 코드는 유지한다.
가이드는 실제 경로에 맞춰 보충 중이며 최종 검증 완료를 뜻하지 않는다.

- 최신 fetch의 devel은 계속 `d9fd85ed5`다. 재배이스는 하지 않았다.
- Native 네 구성의 실제 CLI 19건씩, 총 76건이 예상대로 동작했다. 각 선택 코덱·정상 JPEG,
  공유 사전·palette·Indexed·alpha·mask·손상 입력과 저장·재열기를 확인했다.
  원본 페이지를 96dpi 및 200dpi로 독립 래스터화한 입력을 포함한다. 200dpi는 1,655×2,339픽셀이다.
- Native 비교 빌드는 동일한 `release-test` 프로필이다. 표준 release 기본 빌드와 혼동하지 않는다.
- Native 원본 PDF 직접 비교 17쪽 중 JPX alpha 두 쪽이 최초 TSV에서 각각 51.40187%,
  75.70093%였다. 정식 렌더링 회귀 추가를 보류하고 원본·독립 decoder를 대조했다.
- PDFium 153의 원본 JPX 처리는 `SMaskInData=1`에서 alpha가 두 번 적용된 값이고,
  `2`는 preblend 복원을 하지 않은 값이었다. Ghostscript 10의 원본 PDF 출력은 사양상
  sample 합성과 Native에 일치한다. 원본·실패 TSV는 그대로 보존한다.
  원본 PDF만 Ghostscript pdfwrite에서 PDF 1.4의 Flate·외부 SMask로 독립 변환한 기준을
  별도로 만들어 같은 PDFium·DPI·실루엣 알고리즘으로 대조 중이다. rhwp 출력으로 기준을 만들지 않는다.
  상세 명령·hash·외부 근거는 `stage3/independent/reference-policy.json`에 있다.
- 실제 한컴 2024 Print로 가져온 JPEG 대조군 HWPX를 출력했다. Native 최저 raw는
  99.71675%다. 원본 PDF와의 직접 비교는 100%다. fresh WASM·직접 판독은 아직 남아 있다.
- 현재 Studio의 문서 열기는 `wasm-bridge.ts`의 동기 `new HwpDocument(data)`다.
  공통 `PdfImportSession` Worker의 취소·deadline 검증과 Studio UI 보장을 구분한다.

이 관측은 중간 기록이다. fresh WASM 네 조합·실제 브라우저·전체 TSV·직접 판독·기준 devel 비교,
성능 관측·정식 회귀·필수 lint와 전체 회귀를 완료한 뒤 최종 상태를 갱신한다.

## 지원 정보의 합산 feature 보정

라이브러리 소비 앱은 backend feature를 다른 의존 경로로 켤 수 있다. 본체 alias feature만
조회하면 실제 가져오기 지원보다 적게 표시할 수 있어 backend의 실제 compile-time flags를
공통 조회가 소비하도록 보정했다. decoder·요청 한도·픽셀 처리 로직은 바꾸지 않는다.
중간 산출물은 `a4039329a`의 근거로 보존하고, 최종 source에서 빌드·소비 검증을 이어간다.
진행 중이던 이전 source의 로컬 선택 빌드만 종료했다. CI를 취소하거나 재실행하지 않았다.

## 최종 제품 source와 실제 소비

지원 정보 보정 source는 `cfc5be8fe`다. 이 source에서 Native 네 구성과 최적화 포함 WASM 네
구성을 다시 만들었다. Native는 동일 `release-test`, WASM은 표준 release·codegen 16·wasm-opt
완료 산출물이다. compiler output은 `stage3/final/`, `stage3/wasm-final/`에 보존한다.

- Native 19건×4, fresh WASM 19건×4의 수용·거부·저장·재열기를 확인했다. 아래 스캔 자료 보정
  이후에도 같은 산출물로 전체 소비 검사를 다시 수행했다. 이 재실행에는 재컴파일이 필요하지 않았다.
- 실제 Worker의 취소·deadline은 정책 조회를 넘어서 공개 WASM 가져오기 API 진입을 MessagePort로
  관측한 뒤 검사한다. 자료 길이별로 관측을 구분해 앞선 실패 요청의 알림을 현재 요청으로 세지 않는다.
  정상 JPX 그림 세 번의 누적 작업 초과, 기존 활성 문서 보존·다음 정상 요청도 실제 WASM/Worker에서
  검사했다. Worker 본체는 그대로 쓰고 관측 wrapper는 시험용이다.
- 실제 Studio 파일 열기·CanvasKit 표시, 오류 뒤 기존 문서의 digest·SVG·파일명 보존, 다음 정상 열기를
  확인했다. 실제 app 초기화 응답의 WASM SHA-256은
  `76cb5f0789ddb1c9891fb1d74252eebbaec642d93f50066b153913ac36e615d7`이다.
  pkg/public과 HTTP의 JS·WASM 쌍도 일치한다. 절대 out-dir은 wrapper의 자동 동기화 표기가 아니어서
  처음 public이 남았으며, 명시적으로 같은 쌍을 동기화하고 실제 소비를 확인했다.
- 다른 의존 경로로 backend `jbig2`만 켜는 별도 Rust 앱에서 수정 전 `a4039329a`는
  `accepted=true, declared=false`로 assertion이 실패했다. 같은 앱·graph의 `cfc5be8fe`는
  `true/true`로 통과했다. `union-before.log`, `union-after.log`에 실제 실행을 보존했다.

## 자료 보정과 회귀 편입

최초 전쪽 점수 확인 뒤 직접 판독에서 JBIG2 스캔의 본문 누락을 찾았다. TIFF를 여러 strip으로
저장하고 첫 strip만 MMR로 옮긴 생성 오류였다. 최초 자료·ledger는 `stage3/inputs-initial*`에
보존했다. TIFF를 단일 strip으로 만들고 MuPDF가 추출한 **전체** bitmap을 압축 전 원본과
대조해 두 해상도 모두 픽셀 바이트가 일치함을 확인했다. Poppler 추출에는 경계 차이가 있어
해당 차이도 진단에 보존하고, 정상 입력의 완전성 확인은 원본 bitmap·MuPDF로 수행했다.
제품 decoder는 바꾸지 않았다. 새 자료의 통과를 이전 잘못된 자료의 완전성으로 보고하지 않는다.

PDF Indexed가 JP2 palette를 덮어쓰는 자료도 최초 PDFium 비교에서 검정으로 보였다.
Ghostscript는 사양의 blue/cyan lookup과 rhwp의 source sample에 일치했다. alpha 두 자료와
같이 원본만 독립 변환한 세 기준을 별도로 사용하며 원본·실패 증거를 유지한다.
실루엣 점수만으로 색상을 판정하지 않는다.

보정한 17쪽의 Native/fresh WASM 전쪽 최저 raw는 모두 100%다. 원본·독립 기준과 대표 9종을
직접 읽고 전체 본문, 색상·alpha, 표 외곽·행과 열, 그림 경계·mask의 보존을 확인했다.
이 선행 조건 뒤 `pdf/issue8183/`·원본 bitmap·정식 Rust/브라우저 검사를 편입했다.
page 좌표나 SVG hash를 고정하지 않고 독립 source sample·전체 원본 bitmap과 리소스·쪽 소유·
저장 후 보존을 검사한다. 재생성기는 격리 경로에서 입력 PDF 20개를 바이트 단위로 재현했다.
