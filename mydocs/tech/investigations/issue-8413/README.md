---
kind: investigation
status: active
canonical: mydocs/manual/codex/docs_and_git_workflow.md
last_verified: 2026-11-01
---

# PDF 가져오기 샘플 네 건의 원인 조사

이슈: [#8413](https://github.com/edwardkim/rhwp/issues/8413),
[#8414](https://github.com/edwardkim/rhwp/issues/8414),
[#8415](https://github.com/edwardkim/rhwp/issues/8415),
[#8416](https://github.com/edwardkim/rhwp/issues/8416). 담당 edwardkim.
[공통 수행 계획](../../../plans/task_m100_8413.md).
아래는 수정 전 재현 근거이며, 2026-11-01의 곡선·회전·마스크 수정과 원본 전쪽 검증은
[공통 구현 기록](../../../working/task_m100_8413_implementation.md)에 있다.

## 기준과 재현 범위

2026-10-31 `upstream/devel`을 fetch한 결과 기준은
`bc9dfdd057258de59afebeb5121c4bee7398e07f`다. 네 파일 모두 현재 제품 WASM의
`HwpDocument.importPdf(bytes, { resourceProfile: 'standard' })`에서 사용자 오류를 재현했다.
실행은 Node이며 Studio 대화창·Worker 실행을 새로 검증한 결과와 구분한다.

제품 WASM producer는 `b13334f327193b7a6ed76c9f501f6fdee71c5e11`이며 현재 기준까지
`src/`, `crates/`, `vendor/`, `Cargo.toml`, `Cargo.lock` 차이가 없다.
`pkg/rhwp_bg.wasm`과 `rhwp-studio/public/rhwp_bg.wasm`의 SHA-256은 모두
`b9a7ebc5875dee4cd2d068a2e3bf3283b8beede8fd8f26c24eca0cb0d1f78d78`다.
원인 확인을 위해 기존 제품을 사용했으며 수정 후 fresh 빌드 검증을 대체하지 않는다.

독립 분석은 `pdfinfo`와 Python `pikepdf`로 page/Form content stream, image XObject,
clip 경로, 암호화 사전과 권한을 읽었다. 구조 집계는 실제 요청 예산 카운터나 peak RSS와 다르다.
입력·재현 로그·분석 스크립트는 `output/pdf-import-errors-20261031/`에 있으며,
정리 후에도 아래 원인·원본 식별·코드 근거가 남는다. 원본은 수정하거나 업로드하지 않았다.

## 입력 식별

모두 `/mnt/e/tmp/` 아래의 사용자 지정 원본이다.

| 파일 | 바이트 | 쪽수 | SHA-256 |
| --- | ---: | ---: | --- |
| `img_clipping_01.pdf` | 16,697,228 | 32 | `75324afbe62535a6b4f2bc980269310ca14d90eb385f828fc6857a44bea542b4` |
| `img_bug_01.pdf` | 12,783,971 | 37 | `3fc955a5ceec37d0c5d83c28d09fce54a5fa2654884e82b5ecabe090f20e810b` |
| `PDF_ENCRYPTION_UNSUPPORTED.pdf` | 18,705,307 | 57 | `51eecae552f21e3ddb196ea8ff59f3a84cf9ab8cc431a9cab1425e7f01babe71` |
| `f87pro.pdf` | 15,520,189 | 1 | `6454edae50bd2f93e56ecbe71bf7c3b959dbbf241abf2906753f20dd22a15a57` |

## #8413: 실제 곡선 clip 미지원

`img_clipping_01.pdf`는 32쪽이다. 첫 쪽 content object 117의 경로는 1840×382 크기의
둥근 사각형으로, 모서리에 cubic Bézier와 `W n`을 사용한 뒤 `/X1 Do`로 이미지를 그린다.
대표 경로 시작은 `0 372 m`, `0 377.522858 4.477153 382 10 382 c`다.
원형 clip도 있으며 구조 집계의 clip 52개 모두 곡선을 포함한다.

생산·소비 경로는 `ImageClipTracker::apply` → `ImageClip::visible_bounds` →
`src/parser/pdf/extract.rs`의 이미지 처리 → `PdfImageHandle::decode`다.
`vendor/rhwp-pdf-import-backend/src/extractors/image_clip.rs`는 CurveTo/V/Y를
Unsupported로 표시하므로 정상 곡선 입력을 디코딩 전에 거부한다.

기존 #7970은 직선 볼록 다각형, #8174의 연속 MoveTo 보정은 빈 부분 경로 판정에 해당한다.
이번 입력은 실제 곡선이므로 그 수정의 누락이나 회귀라고 단정하지 않는다.
전체 이미지 50회의 선언 픽셀 합계는 224,528,384다. 곡선을 지원한 뒤에도 문서 64,000,000
픽셀 한도와 관련된 후속 오류가 예상되므로 #8414와 함께 전체 완료를 확인한다.

## #8414: 큰 이미지와 누적 배치 픽셀

`img_bug_01.pdf` 첫 쪽 object 7은 DCT 이미지 7680×4320 = 33,177,600픽셀이다.
단일 이미지 기본 한도 16,000,000을 넘으므로 `extract.rs`의 선언 픽셀 검사에서 실패한다.
코덱 opt-in이나 JPEG 2000 미지원이 원인이 아니다.

2쪽부터 재사용되는 object 14도 같은 해상도의 Flate 이미지다. 전체 124회 배치,
52개 고유 이미지의 선언 픽셀 합계는 배치 기준 1,388,854,656,
고유 객체 기준 162,184,133이다. 고유 객체만 세어도 문서 기본 64,000,000을 넘는다.
캐시만 추가하거나 첫 한도만 늘리면 해결된다는 근거가 없다.

`PdfImageHandle::decode`의 공유 source cache와 배치별 copy/clip 처리, frontend IR의 PNG
인코딩·저장과 첫 Core 생성까지 소비 비용을 추적해야 한다. 단일 stream/codec 예약 32 MiB,
누적 decoded 예산, Worker 종료도 함께 검증한다. 사용자 JSON은 현재 낮은 cap만 허용한다.

## #8415: 인증 전 암호화 일괄 거부

`PDF_ENCRYPTION_UNSUPPORTED.pdf`는 Standard V5/R6, AES-256, `/P=-3392`다.
독립 도구는 빈 사용자 비밀번호로 열 수 있지만 추출·편집·인쇄 권한은 제한돼 있다.
Backend `document.rs`의 budget 있는 constructor는 trailer의 `/Encrypt`가 있으면 인증 전에
`Error::EncryptedPdf`를 반환한다. Frontend `src/parser/pdf/mod.rs`에도 오류 매핑과
추가 `is_encrypted()` 거부가 있다. 기존 일반 backend의 AES 인증 지원과 제품 지원은 다르다.

**2026-10-31 메인테이너 결정:** “편집, 가져오기 허용 정책으로 진행합니다.”
유효한 사용자 인증(빈 비밀번호 포함)에 성공하면 복사·편집 권한 플래그로 추가 차단하지 않는다.
소유자 비밀번호를 별도로 의무화하지 않는다. 비밀번호가 필요한 입력의 인증 자체를 생략하거나,
요청 예산을 없애는 결정은 아니다. 비밀번호를 로그·진단·설정에 보존하지 않는다.

57쪽에 곡선 clip 8개와 배치 기준 66,310,310 선언 이미지 픽셀이 있어 인증만 성공해도
다른 제한에 닿을 수 있다. 권한 플래그·진단만 추가하고 이 샘플을 거부한 채 완료하지 않는다.

## #8416: 쪽수로 좁힌 명령 수 예산

`f87pro.pdf`는 676.546×2319.86pt 크기의 1쪽이며 content stream에 623,266개 명령이 있다.
대표적으로 cubic c 303,629개, l 117,309개, m 41,270개다. 이는 pikepdf의 독립 집계이며
backend가 반복 파싱을 포함해 실제 누적한 값과 구분한다.

`PdfImportPolicy::for_selected_pages`는 standard 8,000,000을 선택 쪽수×250,000과
min으로 좁힌다. 따라서 한 쪽은 250,000이며 실제 입력 명령보다 작다. 다음 옵션으로도
동일 오류를 재현했다.

```json
{"resourceProfile":"standard","resourceLimits":{"backend":{"operators":8000000}}}
```

타임아웃이나 JSON 숫자 조정만으로 해결되지 않는다. 다만 쪽수 축소를 없애면 성공한다는
검증도 아직 하지 않았다. Operator Vec 단일 버퍼, decoded/path/geometry 예산은 별도로
남는다. 이 파일의 object 2118은 5309×3379 = 17,939,111픽셀이고 곡선 clip도 6개 있다.
이미지·clip 후속 제한을 함께 검증해야 한다.

## 선행 해결 이력과 현재 대조

| 원 이슈·PR | 실제 반영 SHA | 당시 해결·독립 근거와 이번 차이 |
| --- | --- | --- |
| #7913 | `4662c069e557e928ed7968d061121fde90b1f027` | bounded 제품 가져오기와 암호화 일괄 거부 도입. 이번에는 인증 경로를 bounded 상태로 연결해야 함 |
| #7958 → #7970 | `515bd72dcde0eecdeece333666b675354b7fa91d` | 요청 예산·캐시 보정. 과거 12쪽 Native 90.51927%/WASM 90.17588%; 이번 1쪽 623,266명령은 별도 입력 |
| #7970 clip | 위와 같음 | 볼록 직선 최대 32 vertices. 곡선·복합 clip은 명시적 비지원 |
| #8174 → #8360 | `f125c735c7d2d44eaa44b78c70465bc89af45fb1` | 연속 m 보정, operator 버퍼 사전 예약 검사·800만 절대 상한/쪽당25만. 이번 실제 곡선·고밀도 한 쪽과 구분 |
| #8403 → #8410 | `1cfa5726cdd8c500df612ec5708103b4a1c4d147` | 타일 이미지·문자 획 수정. 일반 곡선 clip과 암호화 지원으로 확대 해석하지 않음 |

기존 이슈·PR 본문/댓글과 현재 정책·clip·constructor를 대조했다. 네 증상을 검색한 결과
동일 샘플의 기존 열린 이슈를 발견하지 못해 #8413–#8416을 새로 등록하고 담당을 지정했다.
본인 계정의 열린 PR도 조회했으며 없었다. 조사 결과는 수정 완료나 시각 검증 통과가 아니다.
