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
