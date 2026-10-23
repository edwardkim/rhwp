---
kind: reference
status: active
canonical: mydocs/manual/security_feature_limits.md
last_verified: 2026-10-23
---

# 보안 제한과 기능 사용 범위

이 문서는 보안 경계를 유지하기 위해 제한한 기능, 자원 한도에 따른 거부, 일반 미지원을
구분한다. 기준 source는 `e5086b4bab13b8267ee5b9d0104afd3c24a9a160`이며,
0.9.0의 [crates.io 준비 #8174](../plans/crates_io_preparation_20261023.md)에 사용하는 현황이다.
제품 버전은 아직 0.8.7이고, 아래 목록 자체가 0.9.0 게시나 전체 보안 감사의 완료를 뜻하지 않는다.

## 현재 기본값

- **PDF 가져오기는 기본 포함**이다. `pdf-import`를 제외한 빌드만 `PDF_IMPORT_DISABLED`를
  반환한다. 기본 포함 전환은 [PR #7953](https://github.com/edwardkim/rhwp/pull/7953),
  통합 commit `b7a5388034ee66f58a7e1d5363974e83b6374f57`에서 확인한다.
- 본체는 PDF backend를 `default-features = false`로 사용한다. 본체의 `pdf-import`를 켜는 것과
  backend의 `rendering`, `jpeg2000`, `icc-*`를 켜는 것은 서로 다른 선택이다.
  현재 기본 Native/WASM 경로에는 이 추가 backend feature가 없다.
- `parserAllocationBoundsProven`은 계속 `false`다. PDF를 열 수 있다는 사실과
  codec·font·cache의 전체 내부 할당 상한을 입증했다는 주장은 구분한다.
- backend에는 예산을 받지 않는 legacy API도 남아 있다. 아래 PDF 자원 제한은 rhwp가
  사용하는 요청 예산 경로의 계약이며, backend 공개 API 전체의 제한으로 확대하지 않는다.

근거: [본체 manifest](../../Cargo.toml), [PDF frontend](../../src/parser/pdf/mod.rs),
[backend manifest](../../vendor/rhwp-pdf-import-backend/Cargo.toml),
[보안 패치와 감사 범위](../../vendor/rhwp-pdf-import-backend/RHWP_PATCHES.md).

## 보안 경계에서 제한하는 기능

| 기능 | 적용 범위와 현재 동작 | 사용자가 할 수 있는 조치 |
| --- | --- | --- |
| PDF의 JBIG2 그림 디코딩 | 기본 빌드에서 비활성. `pdf-jbig2`로 Native/WASM decoder를 개별 선택한다. | [코덱 가이드](pdf_image_codecs.md)에서 현재 산출물 지원을 확인하고 필요한 feature로 빌드한다. 선택 후에도 요청 한도와 mask 조합 제한은 유지한다. |
| PDF의 JPEG 2000 그림 디코딩 (`JPXDecode`) | 기본 빌드에서 비활성. `pdf-jpeg2000`으로 Native/WASM decoder를 개별 선택한다. | [코덱 가이드](pdf_image_codecs.md)를 따른다. `pdf-import`만 다시 켜는 것으로는 활성화되지 않는다. |
| PDF의 ICC CMM 색 변환 | PDF backend의 `icc-qcms`·`icc-lcms2`가 꺼져 있다. CMM을 통한 정밀 색 변환을 제공하지 않으며 기존 대체 색 변환 경로를 사용한다. ICC가 있다는 이유만으로 모든 PDF를 거부하는 것은 아니다. | 인쇄 색 충실도가 필요한 입력은 색 변환 차이를 확인한다. 다른 backend feature를 켠 결과에 기존 할당 감사 범위를 적용하지 않는다. |
| 손상된 PDF의 xref 전체 재구성 | 요청 예산을 사용하는 PDF 열기 경로는 무제한 legacy 복구 대신 오류로 중단한다. | 원본 프로그램에서 정상 저장한 PDF를 사용한다. 일부 PDF를 다른 뷰어가 복구해 연다고 해서 rhwp도 수용한다고 가정하지 않는다. |
| SVG 글리프의 활성·외부 참조 payload | Native Skia와 Studio CanvasKit의 정적 SVG 글리프 replay 경로는 script·event handler·외부 참조·CSS URL 등을 허용하지 않는다. 정적 부분집합에 들지 않으면 replay 후보에서 제외한다. | 정적 경로로 구성한 글리프를 공급한다. 이 제한을 일반 SVG 파일 전체의 검사·차단 계약으로 확대하지 않는다. |
| 문서 링크의 임의 URI 실행 | 공통 링크 생성/수정과 Studio 클릭은 HTTP/HTTPS로 제한한다. Studio는 사용자 정보가 있는 URL도 열지 않는다. PDF URI annotation은 HTTP/HTTPS/mailto만 내보내므로 `javascript:`·`file:`은 제외된다. | 웹 주소를 사용한다. 기존 링크의 조회·보존과 실행 허용은 별도이며, Studio의 메일 링크 클릭과 PDF의 mailto 내보내기 지원도 다르다. |
| 양식 명령 단추 실행 | Studio의 `PushButton` 클릭은 웹 환경의 보안상 무시한다. 체크박스·라디오·입력란 등 다른 양식 조작과 구분한다. | 외부 명령 실행이 필요한 업무는 문서 버튼에 의존하지 않는 별도 절차로 수행한다. |
| 보호 문서의 평문 복구본 저장 | Studio에서 `requiresPasswordForSave`가 참이면 자동 저장 관리자가 평문 recovery draft를 만들지 않고 기존 draft도 폐기한다. | 명시적인 암호 저장 절차로 파일을 저장한다. 보호 문서 자체의 열기·저장 전체가 차단된 것은 아니다. |

코덱 opt-in은 요청 한도를 우회하는 옵션이 아니다. 새 decoder의 입력에 비례하는 저장소와
논리 작업량을 기존 요청 예산에 연결하지만 전체 parser·font·cache의 할당이나 process RSS·
wall-clock 상한을 증명하지 않는다. CMM은 여전히 기본 미활성이며 이번 코덱 선택으로 켜지지 않는다.

구현 근거:

- PDF codec: [`decode_jbig2_image`·`decode_jpx_image`](../../vendor/rhwp-pdf-import-backend/src/extractors/images.rs).
  `UnsupportedFilter`는 frontend에서 `PDF_PARSE_FAILED`와 backend 진단으로 연결된다.
  일부 다른 mask/codec의 `Unsupported`는 `PDF_IMAGE_UNSUPPORTED`로 연결되므로 오류 코드를
  하나로 단정하지 않는다.
- PDF 색 변환: [CMM과 fallback 계약](../../vendor/rhwp-pdf-import-backend/src/color/mod.rs).
- PDF 복구: [예산 있는 constructor와 xref 검사](../../vendor/rhwp-pdf-import-backend/src/document.rs).
- SVG 글리프: [공통 정적 판정](../../src/renderer/static_svg.rs),
  [Native replay](../../src/renderer/skia/glyph_replay.rs),
  [CanvasKit 후보 판정](../../src/renderer/layer_renderer.rs),
  [Studio 정적 SVG parser](../../rhwp-studio/src/view/static-svg-path-layers.ts).
- 링크: [공통 생성](../../src/model/hyperlink.rs), [출력 URI](../../src/renderer/hyperlinks.rs),
  [Studio 클릭](../../rhwp-studio/src/engine/input-handler-hyperlink.ts).
  scheme 허용은 DNS·사설 주소·접속 대상 전체의 안전성 검사와 다르다.
- 양식 단추: [`handleFormObjectClick`](../../rhwp-studio/src/engine/input-handler.ts).
- 복구본: [자동 저장 관리자](../../rhwp-studio/src/recovery/autosave-manager.ts),
  [보호 상태 연결](../../rhwp-studio/src/main.ts).

## 자원 한도로 거부하는 입력

일반 문서 기능을 끈 것이 아니라, 요청의 작업량·압축 해제량·깊이를 제한한다.
한도는 프로세스 전체 heap이나 실행 시간의 상한과 같지 않다.

| 입력/작업 | 현재 기본 제한과 관측 가능한 결과 | 근거 |
| --- | --- | --- |
| PDF 입력·쪽 선택 | 입력 64 MiB, 선택한 원본 쪽 최대 1,024개. stream/codec 단일 예약 32 MiB, 그림 하나 16,000,000픽셀·문서 합계 64,000,000픽셀 등 독립 한도도 적용한다. 자원 한도 초과는 `PDF_RESOURCE_LIMIT`이다. 작은 파일도 디코딩 작업량 때문에 거부될 수 있다. | [frontend](../../src/parser/pdf/mod.rs), [그림 추출](../../src/parser/pdf/extract.rs), [제품 요청 정책](../../src/parser/pdf/policy.rs), [backend 한도](../../vendor/rhwp-pdf-import-backend/src/import_limits.rs) |
| PDF operator·참조·글꼴·decoded 누적량 | 선택 쪽수와 명시한 host 정책에 따라 유한한 요청 예산을 적용한다. backend의 단독 기본값과 본체가 덮어쓴 값은 다르다. `resourcePolicy`·`resourceUsage`에서 해당 요청의 정책과 사용량을 확인한다. | [제품 요청 정책](../../src/parser/pdf/policy.rs), [metadata](../../src/parser/pdf/mod.rs) |
| HWP5 압축 stream | 문서 열기의 기본 정책에서 stream 출력 256 MiB·누적 출력 512 MiB를 적용한다. 파일 크기·암호화·BinData 등 다른 경계의 한도와 혼동하지 않는다. | [문서 열기 정책](../../src/parser/mod.rs) |
| HWPX ZIP/XML | XML 엔트리당 256 MiB, reader의 XML 누적 출력 512 MiB, BinData 엔트리당 512 MiB. 초과 시 읽기를 거부한다. | [HWPX reader](../../src/parser/hwpx/reader.rs) |
| HWPX 재귀 구조 | section 파싱 깊이 16, container 깊이 64, listParam 깊이 32 등 서로 다른 재귀 경계가 있다. 과도한 중첩은 오류가 된다. | [HWPX section parser](../../src/parser/hwpx/section.rs) |

PDF의 쪽 선택은 입력 파일 전체를 읽고 컨테이너·공유 자원을 검사하는 비용까지 없애지 않는다.
한도 오류를 정상적인 빈 출력이나 앞부분만 성공한 결과로 처리하지 않는다.
공개 CLI 동작과 쪽 선택 방법은 [CLI 매뉴얼](cli_commands.md#pdf-가져오기-기본-포함)을 따른다.

## 보안 차단과 구분할 사항

| 항목 | 현재 분류 |
| --- | --- |
| 암호화 PDF 가져오기 | 제한된 constructor에서 명시적으로 거부하는 지원 경계다. `PDF_ENCRYPTION_UNSUPPORTED`이며 PDF 암호 입력·복호화 여정을 제공하지 않는다. HWP/HWPX 암호 지원 여부와 별개다. |
| Fasoo·SoftCamp 등 DRM 컨테이너 | 보호 컨테이너 감지는 하지만 복호화는 미지원이다. `DRM_PROTECTED` 안내에 따라 권한 있는 원본 프로그램에서 처리한다. 일반 배포용 HWP나 비밀번호 HWP 전체가 미지원이라는 뜻은 아니다. |
| PDF의 일부 mask·복잡한 clip·paint, 원 저작 문단 복원 | 표현·복원 지원의 경계다. 관련 미지원을 모두 보안 취약점 또는 보안 때문에 꺼진 기능으로 부르지 않는다. 일부 Mask·SMask·stencil은 이미 지원하므로 전부 미지원이라고 안내하지 않는다. |
| `wasm-api`·`native-skia` 미선택 | JS facade/자동 초기화와 Native 렌더러의 빌드 선택이다. 보안 결함 때문에 금지한 기능이라는 근거가 아니다. |
| 내부 crate의 `publish = false` | 외부 게시 승인·패키징 준비의 경계다. 실행 중인 제품 기능을 비활성화하는 설정이 아니다. |
| `inspect`·`threat-scan`·`armor` | 조사·출처 표시 기능이다. 문서를 자동 무해화하거나 모든 악성 입력을 차단하지 않는다. `threat-scan`은 신호가 있어도 exit 0일 수 있으므로 JSON 판정을 소비해야 한다. |

보호 형식은 [CLI 암호·DRM 지원 표](cli_commands.md),
검사 명령은 [에이전트 보안 매뉴얼](agent_codex/60_보안.md)을 참조한다.
문서 유래 문자열을 도구 실행 지시로 승격하지 않는 처리는 소비 에이전트의 책임도 포함한다.

## 0.9.0 배포 준비에 남기는 조건

지원 입력·제한 기능·자원 정책의 선택은 [메인테이너 결정 대장 MD05–MD07](../plans/task_m100_8174_maintainer_decisions.md)에
기록한다. 이 현황 문서의 제한 목록을 기능 해제 또는 게시 승인으로 해석하지 않는다.

1. 게시용 README·feature 표에 PDF 기본 포함, 제외 빌드, 추가 codec 개별 opt-in과 CMM 미활성,
   실험적 가져오기와 전체 할당 증명 미완료를 함께 명시한다. backend의 upstream README에
   나열된 기능을 rhwp의 제공 기능으로 복사하지 않는다.
2. 저장소 밖 소비 앱에서 지원 입력과 함께 미지원 codec·암호 PDF·손상 xref·자원 초과의
   거부 및 실패 후 정상 요청 처리를 실제 실행한다. 현재 경계가 registry 패키지에서도 유지되어야 한다.
   Native/WASM과 기본/제외 feature 조합을 구분해 기록한다.
3. [#7911 B01–B05·B07](../plans/task_m100_7911_backlog.md)의 font·cache·할당·제품 경계 검증을
   별도로 추적한다. 목록에는 미검증 경계도 있으므로 모두 재현된 취약점으로 부르지 않는다.
   반대로 package 생성 성공으로 이 검증이 끝났다고 판정하지 않는다.
4. 실제 0.9.0 게시 전에 지원 입력의 신뢰 경계와 잔여 보안 게이트 처리 방침을 확정한다.
   이번 현황 정리는 feature 활성화나 비신뢰 입력 배포 준비 완료의 승인이 아니다.

## 확인 범위

기준 source의 코드·manifest와 기본 Native/WASM feature tree를 대조했다. 기존 Studio 검사인
`autosave-manager`, `hyperlink-target`, `static-svg-path-layers`는 합계 22개 통과했다.
이는 복구본·웹 주소·정적 SVG helper의 실행 근거이며 실제 브라우저 클릭, PDF codec 전수 실행,
전체 보안 감사나 저장소 밖 패키지 소비 검증의 결과로 확대하지 않는다.
명령·exit·로그는 [#8174 1단계 보완 기록](../working/task_m100_8174_stage1.md)에 연결한다.
