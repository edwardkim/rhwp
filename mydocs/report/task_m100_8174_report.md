# #8174 — 0.9.0 패키징·기본 기능·자원 정책 완료 보고

2026-10-29. T01–T08과 이전 기준 T09 검증의 완료 보고다. 제출 직전 devel `f291d772a1`의 #8348·#8350을 확인하여 최신 기준 통합·재검증을 진행하고 있다. 아래 수치는 이전 후보 `237ded61b4`의 검증 이력이며 최신 후보의 통과 판정으로 재사용하지 않는다. 실제 crates.io 게시 일정은 제품 0.9.0과 별도로 결정하며 이번 작업은 registry 업로드·이름/owner 확보·태그를 실행하지 않았다.

## 최종 동작

14개 crate의 본체 0.9.0·Rust 1.93.1·공식 parser/IR/DocumentCore/service/저장 API 계약, 의존 순서·라이선스·게시/부분 실패 복구 절차를 확정했다. JBIG2·JPEG 2000·ICC·손상 xref 복구·SVG 글리프·링크·제한된 명령 단추·보호 문서 복구를 기본 제공하고, 명시적인 제외 빌드도 검증했다. 모든 표현·URI·스크립트·DRM을 무제한 허용한다는 뜻은 아니다.

CLI·공통 API·WASM Worker·Studio가 같은 `standard`/`constrained` 정책과 낮춘 호스트 한도, sticky 실패·취소·기존 문서 보존·다음 정상 요청 복구를 소비한다. 표준 operator 절대상한은 승인한 8,000,000이며 선택 쪽당 250,000과 더 낮은 호스트 cap을 유지한다. Linux `import-pdf` 작업자와 Rust 격리 예제를 실제 실행했다. 다른 CLI 명령의 현재 프로세스 실행과 구분한다.

[보안 기능과 남은 지원 경계](../manual/security_feature_limits.md), [자원 정책](../manual/pdf_resource_policies.md), [게시 절차](../manual/crates_io_guide.md), [T09 실행·보정 기록](../working/task_m100_8174_t09_completion.md)을 따른다. 요청의 논리 카운터는 전체 process heap/RSS 상한이 아니며 `parserAllocationBoundsProven=false`와 호스트 책임을 유지한다.

## 통합과 회귀 보호

고정 devel은 `db5cd5bcc60a69e42f419f18a7f57a33b6fb7fdc`이며 들어온 first-parent PR 14개의 실제 diff·소비 경로를 대조했다. 원본 120개 커밋은 별도 원본 브랜치에 보존했다. 생산 Rust는 `cc140b5812ee628901f89c330530ec5bf75282ad`, 새 관계 회귀는 `82739b1d6a`다. 이후 문서·증적은 제품 코드를 바꾸지 않았다.

통합 중 저장 줄 없는 frame의 글앞/글뒤 표·도형 수용 조건 누락을 고쳐 #8080의 제목 4쪽·중첩 셀과 원래 frame 계약을 회복했다. 규제분석서는 Field/Hyperlink를 기존 width-neutral marker 계약에 따라 공통 메타데이터로 보존하고 실제 시작 컷 다음의 저장 reset을 선택하도록 보정했다. 20·21쪽의 내용 소유·원문 순서/수량·표 종료/후속 본문 관계 검사는 같은 실행 파일에서 base FAIL → 후보 PASS이며 기존 3개 검사도 유지했다.

최신 base에 이미 있던 JPEG encoder 정책 중복도 원 해결 #8191·후속 `f9dc41064a`·base preflight 실패를 추적해 동일 항목 하나만 제거했다. 승인한 고유 14개와 이름·버전·include·의존 간선은 유지했다. 실제 포장 source는 `0abcf1301fcfe7922e235e316799690eea8fb9ff`이며 이후 보고서/증적은 package payload를 바꾸지 않았다.

## 현재 후보의 검증 결과

| 검증 | 실제 결과 |
| --- | --- |
| 일반 release | 기본 opt3·LTO·cgu1 build PASS, library 본체 3,800 + 내부 182 PASS·기존 12 ignore·승인한 page-border 1건 제외. HWP·JPEG alpha PDF CLI smoke PASS |
| 전체 nextest | 55 binaries·11,970 PASS·보고된 46 skip·실패 0. 실행 3,539.005초 |
| Native Skia | library 본체 3,857 + 내부 182 PASS·기존 12 ignore·승인한 page-border 1건 제외. glyph replay 10·누락 그림 2·직접 PDF 14 PASS |
| lint/build/feature | fmt·Native/WASM Clippy·workspace build/전체 대상 Clippy·suite/unit 정책·feature 제외·diff PASS |
| 내부/vendor/doc | 내부 crate 248·PDF backend 5,938(기존 2 ignore)·JPEG 7·doctest 8 PASS(기존 3 ignore) |
| Studio·브라우저 | Node 2,630 PASS·기존 2 skip, TypeScript/build·실제 Worker 17개의 271개 단언·Studio 거절/취소/기존 문서 보존/정상 복구·15쪽 표시/편집/양쪽 저장 PASS |
| 실제 package 소비 | 고유 14개·단일 gzip EOF/CRC/잔여 0·원본 2,624개 파일 대응·다운로드 digest PASS. 독립 Rust/CLI·공개 계약 54·WASM feature on/off PASS |
| 운영·frontend 계약 | workflow Python 196·게시 Python 36·CI Node 334 PASS. editor Node/binding/types/pack 4개 gate PASS |
| 현재 보정 회귀 | 동일 테스트 실행 파일의 base FAIL → 후보 PASS, 기존 포함 모듈 4 PASS. 전체 nextest에서도 새 관계 회귀 PASS |

일반 release library 첫 컴파일은 검증기에 설정한 12GiB 주소 공간 제한으로 LLVM OOM이 났다. 컴파일 전용 제한을 16GiB로 조정하고 같은 opt3·LTO·cgu1로 library 단계만 재실행해 통과했다. 제품 한도·source·테스트 assertion은 바꾸지 않았고 일반 release build 및 통과한 검증도 반복하지 않았다. 외부 계약 harness의 원본 상대 fixture 경로를 보정한 준비 실패와 실제 성공 결과를 구분한다. 사용자 서버는 유지했다.

실제 WASM API base/후보 305개 단언·44개 기록은 열기·내용/쪽/slot·저장 재열기·group/ungroup active owner·책갈피 결과가 같다. 기존 `pr8039`의 불완전한 렌더링 fixture는 HWPX 저장에서 `charPrIDRef: [0]` 미등록 오류를 양쪽 동일하게 보였으며 이 입력의 정상 저장을 주장하지 않는다. 정상 업무 입력은 Worker/Studio의 편집·양쪽 저장·재열기로 별도 검증했다.

## 전쪽·직접 판독·제품 대응

Native와 최적화 fresh WASM 각각 기존 38건·104쪽 + 보존 61건·622쪽, 총 99건·726쪽을 비교했다. 전체 1,452쪽 raw 최저 92.00750%, 90% 미만·누락·독립 PDF와 쪽수 불일치 0건이다. #8160 세 문서 전쪽과 기존 좌표 보호 페이지의 98% 조건도 유지했다. base 보존 1,244쪽 중 수정한 규제분석서 20·21쪽 네 이미지 외 1,240개 PNG는 바이트가 같고 전체 본문을 보존한다.

CFF/Type3·중첩 셀·개행/분배·alpha·#8080·WMF·규제분석서·반복 머리행/이어받기 경계를 독립 Print와 직접 판독했다. 남은 글꼴 획·일부 폼 모양·보간·기존 leader/탭/일부 줄바꿈 차이와 보존 검증의 범위를 구분한다. 새 배치 후퇴나 페이지별 2%p 이상 하락은 없다. T08의 이전 source 성능 측정을 이 후보의 성능 수치로 바꾸어 설명하지 않는다.

[원본 TSV 320개·대표 PNG 158개](../pr/assets/issue_8174/t09/README.md)와 [최종 검증 원장](../pr/assets/issue_8174/t09/verification_manifest.json)에 source·제품·입력/독립 Print·폰트 공급·각 파일 SHA-256을 연결했다. Native 제품은 기능 검증용 release-test/root opt1·cgu64이며, WASM은 표준 mixed rlib/cdylib·release·실제 wasm-opt다. 비공개 15쪽 PDF의 원본·식별 이미지·텍스트와 폰트 프로그램·토큰은 공개하지 않는다.

## 후속 절차

로컬 필수 검증을 완료한 후보로 Open PR을 생성한다. 정확한 code head GitHub CI 성공 뒤 archive self-review·오늘할일을 문서 trailing으로 반영하고 그 head의 CI를 확인해 승인된 일반 merge를 진행한다. merge 뒤 duration 결과·devel 동기화·issue/comment·승인 범위 정리는 canonical 절차대로 처리한다. 실제 registry 게시·태그는 별도다.
