# Task #7911 공동 구현을 위한 세부 보완 백로그

- Issue: [#7911](https://github.com/edwardkim/rhwp/issues/7911), WIP Draft: [#7913](https://github.com/edwardkim/rhwp/pull/7913)
- 작성·갱신일: 2026-10-16 (Asia/Seoul)
- 관리: 메인테이너 작업 브랜치 `task_m100_7911`; 후속 구현은 기여자와 분담
- 정본 계획: [수행계획](task_m100_7911.md), [구현계획](task_m100_7911_impl.md)
- 이관 근거: [2단계 기록 3.22](../working/task_m100_7911_stage2.md#322-폰트-전역-캐시-격리와-truetype-cmap-예약-경계-2026-10-16)
- 현재 위치: **F1–F4의 현재 제품 연결·저장 보정과 최신 devel 통합의 로컬 검증 완료. [4단계](../working/task_m100_7911_stage4.md)에 전체 회귀, 실제 Worker·편집 저장, 원본 PDF 전쪽 비교와 산출물 비용을 기록했다. 저장본 한컴 재조판 1·11쪽 차이는 승인된 B10-1 후속 구현이며, 원격 #7913의 폰트 고지 보정 head `d6bde7307` CI 성공·Ready 전환을 확인했고 승인된 self-merge 절차를 진행한다.**

## 1. 진행 규칙

메인테이너 지시에 따라 **제품 전체 경로의 기반 구현 → 현재 범위 검증·devel 통합 →
기여자와 세부 일감 구현**을 진행한다. F1–F4의 범위·기능 완료 조건은
[구현계획 6절](task_m100_7911_impl.md#6-구현-순서와-승인)에 한 번만 정의한다.
백로그는 기여 참여를 위한 일감 목록이며 전체 소진을 devel 통합의 선행 조건으로 삼지 않는다.

- 새 발견은 실제 소비 경로, 확인된 결함인지 미검증인지, 기능 진행을 막는지를 기록한다.
  F1–F4 실행을 막는 결함은 현재 기능 작업에서 해결하고, 나머지는 등록 후 진행을 계속한다.
- 아래 ID는 #7911 내부 일감 번호이며 새 GitHub 이슈 번호가 아니다. 같은 원인·경로의 항목은
  중복 생성하지 않고 증거를 추가한다. 원격 댓글·이슈 생성·push는 수행하지 않는다.
- 상태는 `대기 / 진행 / 검증 대기 / 완료 / 비해당`으로 관리한다. 완료에는 source 식별자·
  실행 명령·결과가 필요하며 비해당에는 실제 feature/call 경로 근거가 필요하다.
- 기존 출력과 검사는 보존한다. 작은 한도 반례와 정상 대조군으로 검증하고, 성공 건수만으로
  전수 감사·총 heap/RSS 보장·한컴 시각 일치를 주장하지 않는다.
- 통합 전/후의 처리는 아래 분류를 따른다. 우선순위가 높다는 이유만으로 후속 일감을 모두
  현재 PR의 선행 조건으로 바꾸지 않는다. 기존 기능을 깨뜨리거나 현재 제출 범위의 필수
  검증을 실패시키는 항목은 구체적인 실행 근거와 함께 통합 전 처리한다.
- 공개 일감으로 인계할 때 원인·범위·진입 코드·공개 가능한 재현·기대 결과·완료 조건·담당자를
  연결한다. 현재 아래 후속 일감은 담당 미정이다. 실제 GitHub 게시와 할당은 별도 승인 후 수행한다.

### 통합 준비와 후속 기여의 구분

| 항목 | 처리 시점 | 기여자가 맡을 경계 |
| --- | --- | --- |
| B01–B05, B07 | 후속 보안·자원 보완; 비신뢰 PDF 배포 판단에 필요 | 폰트/코덱/캐시의 개별 실제 호출 경로와 작은 한도 반례. 이미 보완한 경로는 유지 |
| B06 | 현재 변경·적용 lint의 실패는 통합 전, 무관한 upstream 정리는 후속 | 원본/patch 진단 대조로 수정 책임을 분리 |
| B08 | devel에서 빌드 가능한 graph·고지는 통합 전, 외부 registry 소비·게시 준비는 배포 전 | package별 dependency·라이선스·독립 소비 재현 |
| B09 | 주장한 지원 범위의 결함은 현재 검토, 명시된 미완성 기능의 개선은 후속 | 표 소유권·fallback·줄/문단 인식의 구체 사례별 개선 |
| B10–B12 | 현재 PR 범위의 검증·devel 정합·CI는 통합 전, 후속 범위는 각 변경에서 실행 | 시각/편집/기존 포맷 회귀 및 실제 배포 범위의 검사 |

개별 항목의 완료 조건과 **이번 기반 PR의 통합 조건**은 다르다. 예컨대 B08의 외부 package
검증이 남아도 현재 저장소 빌드가 올바르다면 그 이유만으로 devel 통합을 막지 않는다.
반대로 현재 코드가 기존 빌드를 깨뜨리면 B08 전체가 후속이라는 이유로 넘기지 않는다.

## 2. 세부 처리 대상

각 항목의 로컬 증거 경로는 `output/pr-review/issue7911/` 기준이다. ignored 증거는 Git에
추가하지 않고 기존 실행 소스에 고정한다. 백로그 작성은 아래 실행을 새로 수행했다는 뜻이 아니다.

| ID·우선순위 | 대상과 현재 근거 | 처리 범위·종료 조건 | 상태 |
| --- | --- | --- | --- |
| B01 · 후속 보안 | Type1/CFF encoding 예약. `fonts/type1_encoding.rs`, `fonts/cff_encoding.rs`; `font-cmap-probe/next-font-audit.md`의 정적 관측 | 실제 simple/Type0 경로의 INDEX·charset·encoding 확장에 요청 예산 연결. 큰 선언·정확한 한도·실패 뒤 정상 요청과 정상 글자 매핑을 Native/WASM에서 확인 | 대기·미검증 |
| B02 · 후속 보안 | glyph name·Unicode memo. `FontInfo::embedded_glyph_name`, `char_to_unicode`의 문자열/Vec/HashMap 생성·복사 | 실제 사용 경로의 생성·반환·memo 삽입 전 계상 및 sticky 오류 전달. cache hit·miss·fallback·요청 격리와 정상 텍스트 보존 확인 | 대기·미검증 |
| B03 · 후속 보안 | font/cache index·FontInfo 잔여 복사. `load_fonts_impl`, encoding/metric/name payload·`Arc::make_mut` | font set/name/fingerprint/identity index·정렬·extractor 이름 복사·남은 payload를 추적. 실제 deep copy와 공유를 구분해 제한·복구 검사. 이미 보완한 전역 cache 격리·TrueType map 공유는 재구현하지 않음 | 대기·미검증 |
| B04 · 후속 보안 | 객체/색 공간 cache index·문자 생성·postprocess 및 frontend 수집의 남은 경계. 3.21은 소유 payload 복사만 검증 | 실제 `parse_pdf_with_options` → 추출 → recognition/lowering → Document 반환에서 기존 제한을 소비하지 않는 컬렉션·문자·진단 생성 경로를 목록화. 필요한 선행 제한과 정상 출력 보존을 검사하고 도달하지 않는 backend API는 근거와 함께 비해당 분리 | 대기·미검증 |
| B05 · 보안 증거 | zune JPEG worker horizontal resize. [3.19](../working/task_m100_7911_stage2.md#319-rgbgray-jpeg의-실제-예약에-요청-예산-연결-2026-10-16)에서 실제 capacity 증가 분기 미관측 | 증가 분기를 소비하는 최소 정상 입력/한도 반례 확보, 예약 전 거절·정상 픽셀·오류 복구를 Native/WASM에서 실행. 해당 feature 경로에서 도달 불가라면 코드와 실행 근거로 판정 | 대기·실행 증거 부족 |
| B06 · 제출 품질 | vendor 단독 strict Clippy. `jpeg-alloc-probe/lint-comparison.json`; 원본 129·patch 128건, 신규 worker 함수 길이 경고 1건 포함 | 고정 toolchain·feature로 upstream과 patch를 구분하고 보정이 만든 진단 해결. 적용 lint 정책과 원본 진단 처리 근거를 명시. blanket allow로 숨기거나 root lib PASS와 합치지 않음 | 대기·실행 실패 확인 |
| B07 · 자원 계약 | 개별/누적 예산의 적용 범위·CPU·메모리 비용. 기존 제한은 총 heap/RSS 상한의 증명이 아님 | 제품 경로와 실제 활성 의존 graph에 대한 적용/미적용 경계 확정. 별도 이미지 64/128 MiB 초기 제안과 구현된 공통 예산의 차이를 명시·정합화. F3에서 실제 Worker의 deadline·취소·세션 보존·재생성을 확인했다. 여기에 자원 소진/지연 입력을 추가 검증하고 Native/WASM 비용 측정. 안전성 주장은 입증한 경계로 한정. 4단계에 실제 12쪽 변환 비용·WASM 크기와 관측 memory capacity를 기록했으나 추출/인식/lowering별 시간과 일반 상한은 미측정 | 진행·일부 비용 측정, 일반 상한 미검증 |
| B08 · 패키지·배포 | backend와 zune/jpeg-decoder 보정판, root optional dependency와 저장소 공용 PDF Worker(`rhwp-shared/pdf/`). 과거 package 증거는 최근 fork 변경 전이며 Worker의 npm export·앱 UI 채택은 아직 미연결 | 실제 배포 graph·정확한 버전·라이선스/NOTICE·고지·archive 내용 확인. 저장소 path 우회 없는 packaged Native/WASM 소비자가 동일 보정판을 사용하는지 검증. 외부 게시 권한/절차를 정리하되 `cargo publish`는 별도 승인 | Studio 직접 PDF 열기 UI는 [#7946](https://github.com/edwardkim/rhwp/issues/7946)·[수행계획](task_m100_7946.md)으로 분리. 패키지 배포·나머지 앱 채택은 대기·최신 통합본 미검증 |
| B09 · 출력 보완 | 12쪽 PDF의 semantic table 보류 7개 쪽·3쪽 모호한 소유권, 줄 단위 임시 Paragraph·fallback. 원본 HWP의 단일 그림이 PDF에는 3개 XObject로 나뉘며 현재 변환은 3개를 보존한다 | F4 제품 진단으로 실제 보류 쪽과 원본 대응 확인. 표 확정·fallback의 내용 누락/중복·소유권·배치 결함을 구분해 승인 범위의 결함 보정. Deferred를 임의 Complete로 바꾸지 않으며 완전한 논리 문단 복원으로 범위를 넓히지 않음. 후속 그림 조각 병합은 CTM·픽셀 경계·clip·paint 순서의 연속성과 별도 그림 반례를 입증한 뒤 수행 | 대기·기존 제한/미검증 |
| B10 · 시각·편집 호환 | F3의 BIN_DATA 유형과 HWP 저장 구조를 보정해 한컴 열기·13개 그림 참조를 복구했다. 11쪽 그림 10의 표시 누락은 일반 표의 물리 여백이 CHAR 단위로 저장되어 앞 행이 늘어난 별도 결함이다. 새 직접 HWP의 한컴 Print에서 그림·캡션 복원을 확인했다. 3쪽 PDF clip 누락도 보정했으며 두 지점의 사용자 시각 판정은 통과했다. [3단계 7절](../working/task_m100_7911_stage3.md#7-f4-hwp-그림-참조-손상-보정-2026-10-16) | 현재 기능 결함부터 보정. 원본 PDF와 제품 출력, 저장본의 독립 한컴 Print를 구분해 전체 쪽수·내용·표 소속·전쪽 TSV·직접 확대 판독. 쪽수나 byte 동일성을 한컴 열기/시각 증거로 대신하지 않음 | 현재 범위 검증 완료·사용자 승인 출력 보존. 통합본의 원본 PDF 12쪽 Native/WASM·실제 텍스트 편집 저장·재열기 통과. B10-1은 후속 대기 |
| B11 · 최종 회귀 | default/feature-on 전체 lint, workspace/all-targets, 전체 Rust 회귀·Native Skia, 기존 HWP/HWPX/HML | 집중/시각 결과 뒤 정본 절차로 최종 제품 head 검증. baseline 완화 없이 회귀 수정. 긴 PR-CI 성격 검증의 별도 승인 절차 유지 | 로컬 완료·전체 10,435건, PDF 집중 130건, Native Skia, default/on lint·실제 Worker 통과. 원격 CI는 B12 |
| B12 · 최종 통합 | 최종 devel `375aff294` 동기화·merge `466533599` 완료. 제품 bundle 비용·본문·대표 이미지 준비 완료; code head CI 성공·Ready 전환 완료; 문서-only 후행 CI·merge 진행 | 원격 base 확인·필요한 통합 후 영향 검증. 최종 제품 bundle 크기/시간 측정(크기 축소 목표 없음), 본문·증적·head 정합 확인. 원격 push/Ready/merge/close는 별도 승인 | 진행·d6bde7307 CI 성공·Ready 완료, 승인된 self-merge의 후행 CI 확인 필요 |

B01–B04는 아직 실제 결함을 모두 재현한 목록이 아니라 미검증 경계다. B06은 별도 lint
실패가 관측된 항목이다. B09는 명시된 지원 제한과 출력 결함 가능성을 구분해 처리한다.
원래 범위 밖 기능을 지원하도록 자동 확장하지 않으며, 현재 경로에서 발견한 누락·오류 은폐는 보정한다.

### 2.1 B10-1 — 저장 HWP의 한컴 재조판 시 1·11쪽 배치 차이

- **상태:** 후속 구현 대기. 2026-10-16 메인테이너가 “1의 해결은 백로그에 기록해서 나중에 구현”, “2번을 진행”하도록 명시했다. 현재 저장·재열기 보정 checkpoint의 선행 조건에서 제외하며, 미달 값을 통과로 바꾸지 않는다. 담당은 미정이다.
- **입력/재현:** `pdf/issue5875/nested_table_text_caption-2020.pdf`를 PDF frontend로 가져와 저장한 `output/pr-review/issue7911/hwp-save-20261016/p3/crop-fixed.hwp` 및 이 저장본의 독립 한컴 2020 Print `crop-fixed-hwp-2020.pdf`. 검증 소스는 같은 디렉터리의 `source-state.json`, 출력·해시는 `validation-result.json`에 고정했다.
- **관측:** 같은 저장 HWP의 Native/WASM 화면과 한컴 재인쇄를 대조하면 1쪽 68.40217% / 68.72149%, 11쪽 78.54899% / 78.74329%다. 별도의 원본 PDF 기준 비교는 전쪽 Native/WASM 최저 90.14609% / 90.16832%이므로 두 기준을 혼동하지 않는다. 그림 10 누락과 3쪽 clip 누락은 이미 보정·사용자 확인한 범위다.
- **진입점/조사:** `src/parser/pdf/structured_lowering.rs`의 문단 물리 여백·셀 흐름 및 `lowering.rs`의 위치 정보를 공통 IR → HWP export snapshot → 한컴 재조판과 대조한다. 원인과 기대값을 먼저 확인하며 px 보정이나 기존 측정값 고정으로 맞추지 않는다.
- **완료 조건:** 같은 입력/저장본/독립 Print의 전체 쪽수·내용·그림 소속을 보존하고 관련 전쪽 Native/fresh Docker WASM TSV 90% 이상 및 1·11쪽 확대 판독을 확인한다. 그 후 독립 계약에 따른 관계형 회귀를 추가한다. 기존 올바른 검사·원본·실패 출력은 보존한다.
- **영구 재현 입력:** `samples/issue7911/pdf-import-nested-table.{hwp,hwpx}` 및 `pdf/issue7911/pdf-import-{hwp,hwpx}-hancom2020.pdf`. 4단계 저장본은 기존 보정본과 바이트가 같고 Native/WASM 12쪽 픽셀도 같아 기존 독립 Print와 미달 수치를 재사용한다. 새 입력이나 새 Print로 바꾸지 않았다.
- **증거:** `p3/native-hancom-scores/silhouette.tsv`, `p3/wasm-hancom-scores/silhouette.tsv`, `p3/hancom-png/`, `p3/native-source-scores/`, `p3/wasm-source-scores/` (위 증적 디렉터리 기준). 이번 후속 이관은 다른 미검증 경로나 배포 준비의 포괄 면제가 아니다.

## 3. 이미 확보한 결과와 재사용 경계

- [3.22](../working/task_m100_7911_stage2.md#322-폰트-전역-캐시-격리와-truetype-cmap-예약-경계-2026-10-16)의
  Native 자원 계약 111건·frontend 4건·명시 여백 1건, TrueType unit 23건,
  fresh Docker WASM 새 계약 14건과 정상 대조 결과를 보존한다.
- PNG/Flate scratch, RGB/Gray·CMYK JPEG의 기록된 예약, 객체/색 공간 소유 payload 복사,
  전역 폰트 cache 요청 격리·TrueType cmap 경계는 이미 보완한 범위다.
  B01–B07은 그 나머지 경계이며 기존 결과를 0부터 다시 구현하는 작업이 아니다.
- `font-cmap-probe/source-final.json`과 원래 실행 로그는 변경하지 않는다. 이후 문서 정리나
  제품 연결의 검증을 과거 실행 증거에 덧씌우지 않는다. 코드가 같고 영향이 없는 계약은
  재사용하고 바뀐 소비 경로는 새로 검사한다.

## 4. 현재 진행 위치

- **현재 실행:** 최종 devel `375aff294` 통합 후 [4단계](../working/task_m100_7911_stage4.md)의 로컬 검증 완료. 실제 Worker 83항목과 편집 저장 19항목 통과, HWP/HWPX 전쪽 Native/WASM은 이전 승인 출력과 동일하다.
- **기능 checkpoint:** F1–F4의 현재 범위와 통합 로컬 검증 완료. 확인한 텍스트 편집·저장 여정을 모든 편집·호환 범위의 완료로 확대하지 않는다. 1·11쪽 한컴 재조판은 B10-1 후속 대기다.
- **백로그 보완 완료:** 미충족. 각 항목의 상태는 2절을 따르며 B10-1은 명시적으로 후속 구현 대기다.
- **devel 통합 준비:** 로컬 검증·기여 진입점·본문·대표 증거 준비 완료. 원격 반영 승인·본문 갱신 완료. CI 보정 head 성공·self-review 기록·Ready 전환을 마쳤다. 사용자 병합 승인을 받았으며 문서-only 후행 CI·병합을 진행한다.
- **배포 준비:** 별도 미충족. 기본 비활성·`parser_allocation_bounds_proven=false` 유지.
- 2026-10-16: 실행 순서 변경과 일감 이관만 기록했다. 이 문서로 실행 실패·미검증을 통과로
  바꾸거나 원격 PR 상태를 변경하지 않았다.
- 같은 날 목적 정정: devel은 함께 구현할 기반이다. 전체 백로그 완료를 통합 조건으로
  두었던 초기 해석을 제거하고, 통합 검증과 후속 기여·배포 항목을 1절에서 구분했다.

## 5. 기여자가 이어받을 진입점

- 변환 책임: [`src/parser/pdf/`](../../src/parser/pdf/)의 추출 → 인식 → lowering → 공통 IR.
  backend 제한은 [`vendor/rhwp-pdf-import-backend/RHWP_PATCHES.md`](../../vendor/rhwp-pdf-import-backend/RHWP_PATCHES.md)를 따른다.
- 기존 계약: [`issue_7911_pdf_frontend.rs`](../../tests/cases/issue_7911_pdf_frontend.rs),
  [`issue_7911_pdf_resource_limits.rs`](../../tests/cases/issue_7911_pdf_resource_limits.rs),
  [`issue_7911_explicit_text_margin.rs`](../../tests/cases/issue_7911_explicit_text_margin.rs).
  테스트 추가·실행 절차는 [`CONTRIBUTING.md`](../../CONTRIBUTING.md)를 따른다.
- 일감 인계 시 `B번호 → 수정할 진입 함수 → 공개 가능한 입력/재현 명령 → 기대 결과와
  정상 대조군 → 이번 PR의 비범위`를 연결한다. 하나의 항목을 맡은 기여자에게 PDF 기능 전체
  완성이나 모든 후속 보안 검증을 요구하지 않는다.
- 현재 ignored output의 로그만 있는 증거는 공개 가능한 최소 입력·검사·관측값을 추려
  기여 인계 자료로 준비한다. 로컬 경로만 제시한 상태를 공개 재현 준비 완료로 표시하지 않는다.
- 미구현 API와 계획된 CLI 명령은 구현·실행 확인 전까지 사용 가능한 기능으로 안내하지 않는다.
  실제 사용 예·지원 제한·후속 일감 링크를 제품 연결 결과와 함께 제공한다.
