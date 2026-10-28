---
kind: investigation
status: active
canonical: mydocs/manual/verification/visual_verification_governance.md
last_verified: 2026-07-28
---

# Task #3486 Stage 1 — HWP3 암호 문서 렌더링 기준선

Issue: [#3486](https://github.com/edwardkim/rhwp/issues/3486)

## 문제 분리

- #3483은 HWP3 암호 해제와 문서 열기 경로를 검증했다. 그 증적 PNG는 문서가 열린 상태일 뿐,
  한컴 출력과의 조판 정합을 보증하지 않는다.
- 현재 관찰한 본문 흐름과 삽입 그림 위치의 큰 차이는 암호 처리 문제가 아니다. 복호화 뒤의
  HWP3 parser → 공통 IR → layout/renderer 경로를 별도 대상으로 조사·개선한다.

## 기준선과 판정 계획

1. 제공된 한컴 오라클 `pdf/HWP3-password-123456.pdf`와 원본
   `samples/HWP3-password-123456.hwp`의 SHA-256·페이지 수를 기록한다.
2. 오라클 PDF와 rhwp의 SVG/PNG 출력을 visual sweep으로 페이지별 대조한다. 페이지 수, 본문
   줄 흐름, 그림 geometry를 분리해 후보를 좁힌다.
3. 유의미한 차이가 폰트 메트릭 차이인지 HWP3 parse/IR/layout 구조 결함인지 원인 경로를
   확정한 후 수정과 회귀 검증을 추가한다.
4. 수정 전후의 3-way/overlay 자료와 자동 지표는 PR asset으로 남기되, 최종 시각 판정은
   한컴 기준을 확인하는 작업지시자에게 요청한다.

## 입력 기준

- HWP3 fixture SHA-256:
  `db743d084efc9e08e839a5b4d978b16b8676434011776e090e4cda43e57304be`
- 한컴 오라클 PDF: `pdf/HWP3-password-123456.pdf` (24페이지, A4)
- 한컴 오라클 PDF SHA-256:
  `3ced5ad95ad30331e2756b5b34509c1ac91dfe3c72013c8e14f2556ca6bd5776`
- 기존 열기 증적:
  `mydocs/pr/assets/pr_3483_hwp3_password_open_review.png`
- 기존 증적은 기준 PDF 대조 자료가 아니므로, #3486의 renderer 수용 근거로 재사용하지 않는다.
- 구조 대조 fixture: `samples/HWP5-nopassword-123456.hwpx`
  (SHA-256 `20ed90f48c6501cad99f6aa1f82d81d2a2132eb04f2d1d32805ac251749e4d0e`)

## 1차 구조 대조 결과

- HWP3 첫 제목의 조합형 코드 `0xD3C5`는 HWPX에서 `ᄒᆞᆫ`으로 보존된다. 기존 HWP3
  Johab 파서는 중성 인덱스 30(아래아)을 미지원으로 처리해 그 한 글자를 버렸고,
  rhwp 제목이 `글 97 안내문`으로 시작했다.
- HWP3 파서가 이 경우를 초성·아래아·종성 자모열로 보존하도록 수정하고, HWPX fixture와
  실제 암호 HWP3 fixture를 함께 읽는 회귀 계약을 추가했다. HWP3의 첫 제목은
  `ᄒᆞᆫ글 97 안내문`으로 복원된다.
- HWPX에는 컬러 BMP를 가리키는 쪽 배경 레코드도 있으나 `alpha=0`이며, 제공된 한컴 오라클
  PDF에는 그 그림이 보이지 않는다. HWP3에 이를 강제로 가시 배경으로 연결하면 본문 위에
  덮여 오라클과 멀어지므로, 이 이미지는 현 수정의 표시 대상으로 삼지 않는다.
- 이 HWPX는 `samples/` 루트의 비교 전용 fixture다. 현재 IR field-sweep corpus는 HWP는
  `samples/`, HWPX는 `samples/hwpx/`만 수집하므로 baseline TSV의 대상은 아니다. 파일을
  일반 HWPX corpus로 승격·이동하는 변경에서는 그 시점의 full sweep 결과를 TSV에 등록한다.
  현 위치에서는 두 fixture를 함께 여는 `hwp3_password_fixture` 회귀 테스트가 보호 장치다.

## 시각 대조 기록

- 범위: 1쪽, HWP3 원본 ↔ 제공된 한컴 PDF, 96 DPI raster overlay
- 자료: `output/task3486-hwp3-oracle/sweep-p001-araea/`
- 결과: 페이지 수는 HWP3/rhwp/오라클 모두 24쪽이며, 제목의 누락 글자는 복원됐다.
  pixel match 85.561%, ink match 14.131%이다.
- Poppler `pdftotext -bbox-layout`은 이 레거시 PDF의 PUA 텍스트 때문에 종료한다. raster
  PDF 페이지는 정상 생성되므로 visual sweep은 이미지 overlay를 계속 수행하고, PDF 텍스트
  marker 분석만 생략한다.
- 남은 본문 줄 흐름·폰트 메트릭 차이와 16쪽 overflow 경고는 아래아 문자 복원과 별개로
  후속 대조 대상이다.
