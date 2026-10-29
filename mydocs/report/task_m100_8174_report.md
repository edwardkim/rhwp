# #8174 — 0.9.0 패키징·기본 기능·자원 정책 보고

2026-10-29. T01–T08을 마쳤고, T09는 최신 devel `f291d772a105d46a4d4d7f40ed13e82391ebdc1a`의 실제 코드까지 통합한 최종 검증 중이다. 현재 생산 Rust는 `fa254efdfe9e33c4dcaf54c0ac3e534819111a9e`, PDF 배치 회귀와 생성 binding 기록은 `a4111b02fb`다. 이전 `237ded61b4` 후보의 통과를 현재 후보의 통과로 대신하지 않는다. PR 생성·push·merge·crates.io 업로드는 아직 실행하지 않았다.

## 구현 범위

14개 crate의 본체 0.9.0·Rust 1.93.1·공식 parser/IR/DocumentCore/service/저장 API 계약과 게시·부분 실패 복구 절차를 정리했다. JBIG2·JPEG 2000·ICC·손상 xref 복구·SVG 글리프·링크·제한된 명령 단추·보호 문서 복구를 기본 제공하며 명시적인 제외 빌드도 검증한다. 모든 표현·URI·스크립트·DRM을 무제한 지원한다는 뜻은 아니다.

CLI·공통 API·WASM Worker·Studio에 같은 `standard`/`constrained` 정책, 더 낮은 호스트 한도, sticky 실패·취소·기존 문서 보존·새 정상 요청 복구를 연결했다. 표준 operator 절대상한은 승인한 8,000,000이며 선택 쪽당 250,000과 더 낮은 호스트 cap을 유지한다. Linux `import-pdf` 작업자와 Rust 격리 예제를 다른 CLI 명령의 현재 프로세스 실행과 구분한다.

[지원 경계](../manual/security_feature_limits.md)·[자원 정책](../manual/pdf_resource_policies.md)·[게시 가이드](../manual/crates_io_guide.md)를 따른다. 요청의 논리 카운터는 전체 process heap/RSS 상한이 아니며 `parserAllocationBoundsProven=false`와 호스트 책임을 유지한다. 실제 게시 일정은 제품 0.9.0 릴리즈와 별도로 결정한다.

## 폰트 지정 후 비교

본문 글꼴과 수식 글꼴을 따로 확인하고 기준 devel·후보의 두 실행 경로에 같은 프로그램을 공급한다. 설치 원본 `HYHWPEQ.TTF` 155,612 bytes와 `HancomEQN.ttf` 244,816 bytes는 바이트가 같은 로컬 사본을 사용했다. glyph ID·advance·outline·family를 바꾸지 않았고 폰트 바이너리를 재배포하지 않는다. 전체 font path는 실제 존재와 파일 크기를 확인했으며 원래 요청 한도를 늘리지 않았다.

수식 글꼴이 빠진 상태에서 수식 1쪽과 두 미주 문서의 낮은 점수를 관측했다. 올바른 글꼴 공급 후 같은 조건의 Native base/후보는 수식 99.89%, 11월 미주 전21쪽 최저 91.04%, 9월 미주 전23쪽 최저 93.39%다. 이 45개 PNG는 양쪽 바이트가 같고 직접 판독에서 수식 글자를 확인했다. 새 devel 자체의 Native/fresh WASM 67개 항목·956쪽도 각각 전쪽 90% 이상이다. 현재 후보의 나머지 WASM 전쪽 검증은 진행 중이므로 이 Native 수치를 후보 WASM 통과로 표시하지 않는다.

사용 방법은 [CLI 가이드](../manual/cli_commands.md)와 [Visual Sweep 가이드](../manual/verification/visual_sweep_guide.md)에 반영했다. 글꼴 공급으로 해소되는 차이와 배치 결함을 구분한다. 실제 글꼴 예외나 허용치 완화를 사용하지 않았다.

## 통합 중 확인한 PDF 배치 회귀

새 devel의 기여 PR 20개와 #8350을 실제 통합하고, 새 PDF 편집 모드의 옵션을 CLI·격리 작업자·WASM·공통 Worker·Studio 환경 설정까지 전달했다. 이어받기 reset 원장·부동 도형/셀 색인 등의 새 할당에도 같은 Storage를 연결했다. 원본 작업 120개 커밋과 이전 검증 후보는 별도 브랜치에 보존했다.

수식 글꼴을 공급한 뒤에도 macOS PDF 3쪽의 78.52%는 남았다. 앞 두 줄의 위치 고정 owner와 그 아래 첫 flow의 경계를 가로질러 문단을 합치면서 본문 점유가 사라지는 실제 회귀였다. 같은 단의 선행 본문 owner를 첫 flow와 함께 보존하도록 공통 producer에서 고쳤다. 별도로 2010 13쪽·2020 12쪽은 마지막 글줄 내용과 추론한 문단 꼬리 간격을 구분하고, 원본의 다음 원점 앞에 실제로 남은 공간만 같은 저장 LineSeg·누적 원점에 전달했다. 이미 수용한 줄 상자·본문 owner·엄격한 50 HU 원점 검증을 유지했다.

현재 생산 Native와 최적화 fresh WASM의 세 editable PDF는 전쪽 최저 98.48% 이상이고 원본 PDF와 쪽수가 같다. 보정한 macOS 3쪽은 Native 99.79% / WASM 99.80%, 2020 12쪽 99.37% / 99.35%, 2010 13쪽 99.53% / 99.53%다. 동일 영역을 원본·새 devel·후보로 확대 판독해 선행 줄·후속 본문·보기 상자의 위치와 내용 순서를 확인했다.

이 선행 검증 뒤 기존 자동 수집 모듈에 독립 PDF의 paint baseline ±2px와 줄 순서를 보호하는 검사 1건을 추가했다. 같은 검사 원문에서 보정 전 `ceda3a7953`는 3쪽 본문 baseline 220.53px / 원본 269.12px로 FAIL, 현재 생산 코드에서는 PASS다. 기존 본문 소유·정상 단/쪽 연결·7쪽 위치·편집·양쪽 저장/재열기를 포함한 모듈 5건 모두 PASS다. 기존 기대값·쪽수·허용치를 바꾸지 않았다.

## 현재 최종 검증 상태

| 검증 | 확인한 결과 |
| --- | --- |
| Native 시각 | 작업/이전 통합 보존 99개 항목·726쪽 + 새 devel 보존 67개·956쪽, 총 166개·1,682쪽 전쪽 PASS. raw 최저 91.04%. 동일 원본의 다른 경로·일부 중복 대조군을 포함하며 고유 문서 수로 표시하지 않는다. |
| 후보 fresh WASM 시각 | 실제 wasm-opt 제품을 사용한다. 세 PDF의 fidelity/editable 전쪽 선행 검증 PASS. 나머지 전쪽·base 대조·최종 직접 판독 진행 중. |
| Native Skia | 본체 library 3,857 + 내부 182 PASS·기존 12 ignore·승인한 page-border 1건 제외. glyph replay·누락 그림·직접 PDF의 필수 3개 gate PASS. |
| 집중·정책 | PDF 요청 수명 43·기존 보정/대조군·새 editable 모듈 5건 PASS. fmt·Native/WASM Clippy·suite/unit 정책을 통과했다. 전체 nextest·workspace 전체 대상 lint/build는 진행 중. |
| Studio·브라우저 | Node 2,665 PASS·기존 2 skip, TypeScript/build, 실제 Worker 17개의 271개 단언과 Studio의 거절/취소/이전 문서 보존/복구·15쪽 표시·편집·양쪽 저장 PASS. PDF 편집 우선 환경 설정과 새 AI provider/Canvas2D 경계도 실제 browser에서 확인했다. |
| editor·운영 계약 | editor Node/binding/types/pack 4개 gate PASS. 변경이 없는 workflow Python 262·게시 Python 36·CI Node 334 PASS 결과는 해당 경로의 source 동일성을 대조해 유지했다. |
| 입력·제품 | 공개 입력 279개의 실제 바이트가 현재 커밋과 같다. Native 89,430,256 bytes·SHA-256 `8f7a4256f395e9a1b75f5dd286c1d46d80ed164822e2bf68404bff741339ccbd`. WASM 27,120,619 bytes·`6bbd2a8bbefb62180f984b6a3ddc01443af7d3ef0b446dd3ab4df01bd4f8b34e`. 생산 Rust는 이후 바뀌지 않았다. |
| 남은 필수 항목 | 전체 Rust 회귀·일반 release·14개 실제 archive와 독립 Rust/CLI/WASM 소비·최종 성능 대조·현재 공개 시각 asset/검증 원장 확정. |

Native는 기능 검증용 release-test/root opt1·cgu64, WASM은 표준 mixed rlib/cdylib·release·실제 wasm-opt 제품이다. 현재 생산 코드의 CFF/Type3·표/캡션·중첩 셀도 직접 판독했다. 글자 획/보간 같은 기존 표현 차이를 전체 피델리티 결함의 해결로 주장하지 않는다. 비공개 15쪽 PDF는 수치만 제출하며 식별 원본·이미지·텍스트·폰트 프로그램·토큰은 공개하지 않는다.

## 다음 절차

남은 필수 로컬 게이트를 완료한 뒤 현재 source의 전쪽 TSV·대표 review/overlay·검증 원장을 확정한다. 승인 범위에 따라 Open PR을 만들고 정확한 code head GitHub CI 성공 뒤 archive self-review·오늘할일을 문서 trailing으로 반영한다. 그 head의 CI와 최신 base 병합 가능성을 확인해 일반 merge·후속 동기화/issue/comment/정리까지 진행한다. registry 게시·owner 변경·태그는 실행하지 않는다. 상세 이력은 [T09 작업 기록](../working/task_m100_8174_t09_completion.md)을 따른다.
