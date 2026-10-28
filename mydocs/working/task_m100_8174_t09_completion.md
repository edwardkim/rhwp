# #8174 T09 — 최신 devel 통합과 PR 제출

2026-10-29. **이전 기준 `db5cd5bcc`의 로컬 필수 검증은 완료했다. 제출 직전 devel `f291d772a1`의 #8348·#8350 통합을 확인하여 J02–J05의 최신 기준 보존 검증을 진행한다. PR 생성·push·merge는 아직 실행하지 않았다.**
사용자가 승인한 [완료 범위](../plans/task_m100_8174_t09_completion.md)의 J01–J06을 한 단위로 진행한다.

## 후보와 통합

- 기존 작업 120개 커밋과 T08 기록은 `chore/issue8174-resume-20261024`에 보존했다.
  T08 완료는 `4ef642c9edf8313ce07bc6e3334f9cd77ca8b350`, T09 계획 추가는 `371cbbb47b`이다.
- 같은 파일 트리의 누적 후보를 따로 만든 뒤 공통 기준 `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`에서
  최신 devel `db5cd5bcc60a69e42f419f18a7f57a33b6fb7fdc`로 실제 리베이스했다.
  첫 통합 후보는 `df408614f685c57548a23b0741915104c626ec57`이다.
- 들어온 first-parent 통합 PR은 14개다. 생성 JS·선언은 후보의 표준 최적화 fresh WASM 빌드 결과로 갱신했다.
  `aca291877`의 생산 Rust와 후속 문서/생성 binding 커밋을 구분하며 최종 source 대응을 고정한다.
- 원문 제어 주소·TAC 물리 그리드·줄 종료/분배·저장 프레임·자동 번호·책갈피의
  devel 계약과 이번 요청 Storage를 공통 생산자에서 결합했다. 새 소유 필드는 전수 구조 분해로
  복제 목록에 포함하고, 줄/스타일/문단 사본과 예약·실패 전파도 함께 연결했다.
- 기존 테스트 삭제·이름 변경 29개 경로는 새 devel의 정식 검사/이관 구성을 유지했다.
  #8174 전용 검사는 제거하지 않았다. #8188의 복원된 #8160 검사와 편람 제외를 구분했다.
- 미수정 qcms 라이선스·C 바인딩의 원본 공백은 보존했다. 해당 세 경로의 attributes만 명시해
  새 누적 diff 검사에서도 원본 바이트와 기존 upstream 무결성 계약을 유지한다.

## 현재 실행 결과와 보정

- Native `cargo check --locked --lib --features native-skia`와 `cargo clippy --locked --features native-skia --lib -- -D warnings` 통과.
- suite 정책: 1,866 sources, static attrs 8,038, integration targets 48/48.
  unit 정책: 4,099 tests, 296 modules, ready 0.
- 첫 요청 수명 검사 전체 실행은 **34 PASS / 9 FAIL**이다. 9건 모두 한도 거부 자체는 성공했고,
  이전의 특정 단계 문구 `PDF pagination:` 단언에서 실패했다. 한 건씩 실패 로그를 대조했다.
  원 검사는 `4a2fe9e2dc`와 후속 단계에서 추가했고 T08에서는 43개 모두 통과했다.
- 새 devel의 초기 상태와 이번에 연결한 예약으로 고정 입력 예산이 더 이른 단계에서 거부될 수 있다.
  기존 역사적 예산값은 유지하고 공개 계약 `PDF_RESOURCE_LIMIT:`를 확인한다. 실패 고정·재사용 거부·
  한도 이하 사용량·새 요청의 같은 쪽수/저장 바이트 복구 단언은 유지한다. 보정 후 **43개 모두 PASS**다. 역사적 예산값과 복구·수명 단언은 유지했다.
- 코드 대조에서 새 devel의 TAC 실측 높이를 선언 높이로 다시 덮어쓰지 않는 분기도
  fallible 공통 생산자에도 보존했다. 실측 높이와 host 구성 사본의 예약을 보완하고 위 검사를 통과했다.

증거와 명령 원문은 기본 작업공간의 ignored `output/pr-review/issue8174/t09-completion-20261028/`에 있다.
최종 통과 결과·source/제품 SHA·시각 자료와 PR 링크는 검증을 완료한 뒤 이 문서에 확정한다.

## Studio 설치 전 의존성 확인

`dependency-audit` 절차로 lockfile의 정확한 production 의존성 13개를 확인했다.
알려진 취약점·install script·deprecated·typosquat 신호는 없었다. devDependencies는 이 검사에서 제외했다.
아래 네 항목은 도구의 갱신 빈도 분류가 `stale`이며, 정확한 버전의 registry metadata에서도
취약점·deprecated·설치 시 lifecycle script는 확인되지 않았다. 이 분류만으로 의존성을 바꾸지 않았다.

| 패키지 | 확인 결과 | 처리 |
| --- | --- | --- |
| [fast-sha256 1.3.0](https://npmscan.com/package/fast-sha256) | stale, 알려진 취약점 없음 | 현재 고정 버전 유지 |
| [json-schema-to-ts 3.1.1](https://npmscan.com/package/json-schema-to-ts) | stale, 알려진 취약점 없음 | 현재 고정 버전 유지 |
| [pngjs 7.0.0](https://npmscan.com/package/pngjs) | stale, 알려진 취약점 없음 | 현재 고정 버전 유지 |
| [ts-algebra 2.0.0](https://npmscan.com/package/ts-algebra) | stale, 알려진 취약점 없음 | 현재 고정 버전 유지 |

후보 Studio에 `npm ci --ignore-scripts --no-audit --no-fund`로 385개 패키지를 설치했다.
기존 사용자 서버와 설치된 도구는 종료하거나 변경하지 않았다. 실제 빌드·테스트 결과는 별도로 확인한다.

## 승인 확대와 초기 통합 검증

아래는 최종 `cc140b5812` 보정 전의 실행 이력이다. 최종 제품과 완료 범위는 뒤의 전쪽 증적을 기준으로 한다.

2026-10-29 작업지시자가 로컬 검증 완료 → Open PR → 정확한 head의 CI 성공 → self-review →
merge까지 승인했다. registry 게시·태그는 별도다. 검증 중인 항목을 완료로 바꾸지 않는다.

- 표준 mixed rlib/cdylib·release·실제 wasm-opt를 사용한 fresh WASM은 26,695,988 bytes,
  SHA-256 `2176c819b9e2b74f704d860aa450c2a5369a465fd5b6a593bdbd1bcf3c4adfb8`다.
  후보 pkg·Studio public·dist의 실제 JS/WASM을 대조했다. 기본 작업공간의 사용자 서버는 갱신하지 않았다.
- Native 제품은 release-test/root opt-level 1·codegen units 64의 기능 검증 제품이다.
  요청 수명 43·자원 134·frontend admission 2·제어 slot 23·독립 Print 위치 29개 검사를 통과했다.
- Native Skia 전체 library·glyph replay·누락 그림·직접 PDF가 통과했다. 전체 nextest와
  후속 workspace 검사는 실행 중이며 통과를 가정하지 않는다.
- Studio Node 2,630 PASS·2 skip, TypeScript·배포 build PASS. 실제 Worker 17개의 오류/복구,
  Studio 입력 거절·취소·정상 복구와 양쪽 저장/재열기를 확인했다.
- 운영 Python 계약 196개·게시/source 계약·CI Node·actionlint를 통과했다. 최초 계약 실행에서
  0.9.0과 사용자 안내 네 곳의 0.8.7 불일치를 발견했다. 원 계약 `2d281e5e1b`와 새 base의
  동일 검사를 확인하고 안내를 보정한 뒤 재실행했다. 검사 조건은 바꾸지 않았다.
- Native 격리 host를 일반 사용자와 `unshare -Ur`에서 실제 실행해 HWPX 저장·재검증을 통과했다.
- 기존 작업의 Native/fresh WASM 각각 38개·104쪽을 새 제품으로 전쪽 산출했다.
  Native 최저 98.34671%, fresh WASM 최저 98.79200%이고 JPEG 17개는 양쪽 100%다.
  CFF·Type3·중첩 셀·분배/개행 경계·JPEG alpha를 현재 compare/overlay에서 직접 판독했다.
  글꼴 획·Type3 및 JPEG 보간 차이는 남으며 RGB 동일성을 주장하지 않는다.
  T08과 PNG 비교는 208개 중 48개 동일이다. 새 devel 반영 후 나머지의 재출력을 과거 캡처로
  대체하지 않았다. 새 base/후보 보존 대조와 최종 대표 판독은 계속 진행 중이다.

## 들어온 PR과 실제 소비 범위

원 PR 본문·review·diff와 공통 경로를 확인했다. 아래 실행 중인 항목의 최종 판정은
base와 후보의 동일 입력 결과를 확보한 뒤 확정한다.

| 들어온 PR | 실제 변경과 이번 후보의 소비 경로 | 현재 검증 범위 |
| --- | --- | --- |
| #8116 | 저장 줄 없는 float host·셀 예약·재조판 공통 경로 10개 | 공개 #8080 원본 네 개·Print 전쪽, 관련 정식 회귀 |
| #8188 | 원본 HFT/WMF·저장 글줄·구형 수식·탭, #8160 검사 복원과 CI 5 archive | 목표 sample16 세 개 전체와 정상 대조군, 원본 Print·소유/내용·Native/WASM |
| #8228 | 10개 contributor 통합의 글꼴/표/원본 프레임과 Studio AI | 실제 공통 호출의 재현/대조 입력, 정식 회귀·AI SDK 실제 browser |
| #8231·#8240·#8274·#8280·#8288 | review·고지 문서 | 제품 코드 비해당; 실제 diff·링크/보존 경로 확인 |
| #8273 | duration CLI의 integration-e 보고 소비 | 기존 실제 Node CLI 계약·Python workflow 검사 |
| #8278 | duration trigger와 문서 변경 시 실행 조건 | workflow 계약·actionlint |
| #8284 | Rust/npm/Actions 21개 갱신과 SDK 1.x 인증 검사 | 잠금 유지·Native/WASM·Studio 및 실제 AI panel/settings browser (외부 API 대역) |
| #8291 | 종료 시 output 전체·공유 debug 정리 규칙 | 제품 코드 비해당; 최종 종료 gate에서 해당 canonical 적용 |
| #8319 | 45개 PR의 편집·slot·글꼴·paint·저장 경로 | 현재 정정 원장의 유효 재현 입력 39개, 실제 API/저장·정식 회귀 |
| #8328 | #8319 원본 대응 정정과 EMF/저장 줄·drawText 소유 보정 | 정정 입력/Print·예외 및 원본/대체 분류 보존, 공통 source/font/paint 실제 출력 |

#8328의 현재 입력 대응을 우선한다. 재현·공식 대체·파일 없는 단위 계약을 원 작성자의 실물
원본으로 표현하지 않는다. 배경 원본 미첨부 다섯 단위 계약과 #8033의 별도 진단은 현재 base의
명시된 계약을 따라 구분한다. 새로 확보한 공식 문서 전체의 추가 피델리티 개선을 이 작업의
새 기능 범위로 확장하지 않는다. 기존 source/Print와 현재 필수 보존 검증은 유지한다.

## 보존 대조에서 검출한 통합 회귀 — 보정 및 재검증 중

새 base Native와 첫 후보를 같은 원본·Print·글꼴·2022 조건에서 직접 대조했다.
#8080의 중첩 셀은 base 98.86895% → 후보 68.48688%였고, 제목 문서는 base 4쪽 → 후보 5쪽이었다.
정식 #8080 검사 원본의 현재 바이너리 실행도 12 PASS·3 FAIL이었다. 글꼴만의 차이로
분류하지 않으며, 기존 기대값이나 입력을 바꾸지 않는다. 초기 전체 Rust 실행의 일부 결과를
전체 통과로 재사용하지 않는다.

원 해결 #8116 merge `faa885e8b5`의 실제 diff와 최종 base의 공통 줄 구성 조건을 확인했다.
`try_layout_paragraph_in_frame_impl`의 저장 줄 없는 host 수용 조건을 fallible 경로로 옮기면서
글앞/글뒤 표·도형을 허용하는 두 항이 빠져 있었다. 이 때문에 실제 가용 상자에서 채워야 할
줄이 45자 합성 줄에 남았다. 공통 frame 수용 조건에 base의 두 항을 그대로 복원했다.
문서명·쪽수·특정 좌표 분기는 추가하지 않았다. 같은 줄 구성 결과를 측정·배치가 소비하는
기존 경로를 되살린 보정이며, 현재 제품 재빌드·집중/전체·Native/fresh WASM 재검증은 남는다.

## 2026-10-29 최종 제품 재검증과 시각 조건 보완

`351b0d587b`의 생산 Rust를 재컴파일한 Native와 표준 최적화 WASM을 사용했다.
공유 target에서 base를 빌드한 뒤 JPEG 디코더 소스 차이가 제품에 반영되지 않은 현상을
독립 Rust/CLI에서 검출했다. source bytes는 유지하고 후보의 변경 Rust/TOML/lock mtime을
갱신해 실제 decoder 재컴파일을 유도했다. JPEG 집중 3건과 실제 alpha PDF CLI가 통과했다.
캐시 파일·fingerprint를 삭제하거나 산출물을 검증 결과에 맞춰 바꾸지 않았다.
재컴파일 후 Native SHA는 `303ebf1788243b980a043160a4a656b53ad0553ca61be4f7d6f47efd653b0be3`,
최적화 WASM은 26,670,759 bytes·`b372d86dca1c7b9f1dbf99f978efdf08944e67d8f7e0200344e7f468e2ef46af`다.
이 제품의 집중 7종·Native 전체 library·양쪽 lint·Studio 2,630 PASS/2 skip·타입·build·실제
Worker/Studio 오류·취소·정상 복구를 통과했다. 새 devel의 실제 AI SDK panel/settings와
CI duration Node 도구도 base/후보 양쪽에서 통과했다. 아래 보정 후 최종 head의 결과와 구분한다.

sample16의 초기 미달은 실행 조건부터 확인했다. 기존 검증은 2024 원본/Print에는 compat 2024,
2020 Print에는 compat 2022와 독립적으로 확인한 Gulim 프로그램을 사용했다. 기존 raw100
원장에 고정된 독립 HFT 프로그램·명시적 환경도 일반 fallback과 구분했다. Windows collection
face를 추출하고 사용 글자의 cmap·glyph ID·hmtx·glyf bytecode를 유지한 공급을 만들었다.
불필요한 대형 face가 Native의 64 MiB 공급 예산을 먼저 소진하지 않게 필요한 실제 face를
앞에 공급하며 제품의 보안 상한은 바꾸지 않았다.

기존 독립 HFT job은 만료 상태였다. 같은 전체 한글 11,172자 표본을 새로 Print 했다.
2024 시도는 `HANCOM_OBSERVER_HANDOFF_FAILED`로 실패했고, 2020의 별도 job
`ad16c528-047b-4dbe-925b-c0eccfbd0cfa`는 succeeded/download success/Print 0이었다.
대상 PDF에서 글꼴을 만들지 않았다. 새 원본 윤곽은 대상 2024의 193자·51자와 2020의
193자를 읽기 전용으로 대조해 전진폭·윤곽이 모두 일치했다. 글꼴·독립 표본 PDF는 비공개다.
같은 원본/환경의 sample16 세 문서 전64쪽을 base와 후보 Native/fresh WASM에서 다시 산출해
모두 99.47% 이상으로 확인했다. 단순 2022/기본 fallback 진단을 원 검증의 회귀로 판정하지 않았다.

## 규제분석서 뒤쪽 저장 컷 — 현재 보정 검증 중

`86712_regulatory_analysis.hwp`와 공급 글꼴 한컴 2024 Print를 같은 조건으로 대조했다.
고정 base `db5cd5bcc`와 첫 통합 후보 모두 21쪽이 약39.8%였다. 20쪽 표 끝에서 한 줄이
21쪽으로 밀리며 표·후속 본문도 한 줄 내려갔다. 원본과 독립 Print의 20/21쪽에서 확인했고,
글꼴 공급 보완·명시 TTF/원본 HFT·compat 진단으로 배치 차이가 없어지지 않았다.

원본 셀 문단69의 저장 LineSeg는 `69120 → 70560 → 0` HU로 이어지며, Print의 앞 두 줄은
20쪽·마지막 `다`는 21쪽이다. 실제 cap 진단은 `start=84`, `j=133`, 기존940.8px+
다음19.2px, 예산958.67px로 1.33px 초과를 보고했다. 다음 줄로 가는3.2px는 저장된 쪽
재시작 뒤의 전진이다. 원 해결 #7704(`429dee416e`)의 공통 helper는 행의 첫 재시작만
조회했고, #8319 통합의 물리 컷 경로에서도 뒤 조각의 현재 시작 컷을 전달하지 않았다.

보정은 `native_saved_reset_cut_trailing_trim → stored_plain_restart_cut → 수용 컷/물리 높이 →
scan/emit → table_partial`의 기존 공통 결과를 유지하며 해당 셀의 실제 시작 컷 뒤 재시작을
조회하도록 한다. 형제 셀의 원본 검증, 편집/재조판·중첩 viewport 제외, 실제 물리 fit 검사는
유지한다. 파일명·쪽수·좌표 예외나 허용치 변경은 없다. source 증거와 새 제품의 경계·전쪽
Native/fresh WASM을 먼저 확인한 뒤 정식 회귀를 보완한다.

이 보정 때문에 이전 전체 nextest를 5,079개 실행 후 중단했다. 실패 검출이 아닌 로컬 중단이며
11,969개 전체 통과로 기록하지 않는다. 보정 후보의 최종 전체 회귀·포장·PR/CI는 아직 남았다.

현재 시작 컷만 전달한 첫 후보는 PNG20/21쪽이 바뀌지 않았다. 해당 후보를 통과로 기록하지 않는다.
추가 실행 진단은 현재/다음 유닛이 원본 슬롯(`reflow=false`), 셀 frame·문자 partition·세로
위치도 미편집임을 확인했다. 그런데 `line_restart=false`, 행 컷은 `None`이었다. 원본 문단69에는
하이퍼링크 `Field` 표식이 보존되어 있었다. 단일 단 메타데이터만 인정한 공통 판정이 이를
실제 배치 개체처럼 제외했다. 원본 제어 주소를 보존하는 #8319 경로와의 실제 소비 차이다.

필드/링크는 기존 composer의 `control_is_width_neutral_marker`로 별도 폭·상자를 소유하지 않음을
확인하고 공통 셀 메타데이터 판정에 연결했다. 결과 글자는 문단 텍스트가 소유하며 원본 field와
주소를 삭제하지 않는다. 자동 번호처럼 별도 생성 문자를 가진 다른 표식으로 범위를 넓히지 않는다.
정규화·측정·저장 컷·물리 배치가 같은 판정을 소비하게 했으며 여러 단·표/그림/수식/각주의 기존
제외를 유지했다. 임시 진단 출력은 제품 변경에서 제거했다. 이 보정의 Native 재빌드·경계 실행이
진행 중이며, 통과 시 최종 전체 검증을 새 source에서 실행한다.

필드 메타데이터와 현재 시작 컷을 함께 보정한 Native 경계 실행은 64쪽을 유지했다.
20쪽 raw99.89009%, 21쪽99.91311%이며 표 끝·아래의 `3`·비용편익 분석 제목·후속 표의
동일 배율 직접 판독에서도 위치가 복원됐다. 원본 field/텍스트는 보존했고 글꼴 획 차이는 남는다.
이 경계 결과만으로 전체 완료를 판정하지 않으며 새 최종 제품·전쪽·전체 검증을 이어간다.

### 최종 제품의 전쪽 선행 검증과 회귀 등록

프로덕션 source `cc140b5812ee628901f89c330530ec5bf75282ad`의 Native와 실제 wasm-opt를 거친 fresh WASM을 다시 출력했다. 규제분석서 64쪽은 독립 한컴 2024 Print와 쪽수가 일치하며 전쪽 최저는 Native 97.67149%, WASM 98.07239%다. 20·21쪽과 기존 px 보호 대상 28·29쪽은 양쪽 모두 98% 이상이다. 최종 제품의 20·21·28·29쪽을 독립 PDF와 같은 배율로 직접 판독해 표 끝·다음 본문·내용 순서와 빈 종료 줄 보존을 확인했다. Native의 일부 글자 획 두께 차이는 배치 차이와 구분한다.

이 선행 검증 뒤 기존 자동 수집 모듈 `issue_7765_current_source_frames`에 20·21쪽의 링크 필드 포함 원본 문단이 2줄/1줄로 나뉘고 원문을 한 번씩 소유하며 후속 문단보다 먼저 닫히는 관계 검사를 등록했다. 기존 25~29쪽의 3개 검사는 유지한다. 28·29쪽의 독립 ±2px 조건도 최종 양쪽 출력과 직접 판독이 허용하므로 유지했다. 새 검사의 수정 전 FAIL/후 PASS와 최종 통합 실행 결과는 아래에 별도로 기록한다.

### 새 관계 회귀의 실제 실행

신규 회귀를 자동 수집하는 파생 harness를 `--prepare`로 재생성했고 기존 48개 target 한도를 유지했다. 같은 테스트 실행 파일의 새 검사 1건을 고정 base 바이너리로 실행하면 20쪽 1줄/기대 2줄로 FAIL, 최종 Native 바이너리로 실행하면 PASS다. 기존 검사 3개를 포함한 해당 모듈 4건도 PASS다. 수집 0건인 선행 진단은 검증에서 제외하고 실제 수집 1건의 실패·성공 로그를 사용한다. 기존 회귀·허용치·입력은 바꾸지 않았다.

Native 보존 대상 61개·622쪽은 전쪽 90% 이상이고, #8160 세 문서의 모든 쪽은 기존 98% 조건도 충족한다. base와 후보의 본문·소유 대조에서 규제분석서의 의도된 20·21쪽 이동 외에는 변화가 없다. 머리말/꼬리말은 본문 연결 순서와 별도로 다룬다. 규제분석서의 전체 본문 27,112자는 순서·수량이 같다. WASM·전체 Rust·최종 package는 최종 결과에서 별도로 확정한다.

## 최종 전쪽·실제 API 보존 완료

최종 Native 제품은 88,666,272 bytes·SHA-256 `555b915741ff7d85e23989fd366c922fc187377c138fa9d08513edd9ef5fc2cd`다. 표준 mixed rlib/cdylib·release·실제 wasm-opt의 WASM은 26,667,835 bytes·SHA-256 `02d5a1fe2f40c832b85e436894411bfd7d707f8f8226f5d2e665cf4da74fbb1e`다. 두 제품의 생산 source는 `cc140b5812ee628901f89c330530ec5bf75282ad`이며 이후 테스트·문서는 생산 Rust를 바꾸지 않았다.

기존 작업 38건·104쪽과 들어온 PR 보존 61건·622쪽, 총 각 실행 경로 99건·726쪽을 실제 재출력했다. Native/fresh WASM 전1,452쪽 모두 90% 이상이며 최저는 92.00750%다. #8160 세 문서의 전쪽 98%와 규제분석서 28·29쪽의 좌표 보호 조건도 양쪽에서 유지했다. base 비교 1,244쪽 중 의도된 규제분석서 20·21쪽의 네 PNG 외 1,240개는 바이트가 같다. 두 쪽의 변경은 독립 Print의 줄 소유와 후속 본문을 회복했고 문서 전체 본문 27,112자를 순서대로 보존한다.

[정식 전쪽 TSV·대표 이미지](../pr/assets/issue_8174/t09/README.md)에 원본 TSV 320개와 현재 제품의 review/overlay 및 이어받기 대조 PNG 158개를 보존했다. [최종 검증 원장](../pr/assets/issue_8174/t09/verification_manifest.json)은 source·제품·입력/독립 Print·TSV/PNG hash와 폰트 공급의 glyph ID/advance/outline 보존을 기록한다. 비공개 15쪽 PDF는 수치만 제출하며 식별 원본·이미지·텍스트를 공개하지 않는다. Windows/한컴 폰트 프로그램도 재배포하지 않는다.

CFF의 숫자·기호·리거처/Unicode, Type3 그림 글자·표/캡션, 중첩 셀의 앞뒤 내용, 개행·분배 경계, JPEG alpha, #8080 중첩 셀/제목, #8160 18·35쪽의 WMF, 규제분석서 20·21·28·29쪽과 rowspan-header 2·3쪽의 반복 머리행/셀 소유를 같은 영역으로 직접 판독했다. 남은 글꼴 획/라디오 단추 모양·이미지 보간, 기존 목차 leader/탭 위치와 일부 고정 폭 줄바꿈 차이는 해당 base PNG에도 동일하다. 보존 검증을 이 기존 표현 차이 전체의 해결로 주장하지 않는다. 새 배치 회귀나 2%p 이상 하락은 없다.

실제 브라우저 WASM API 305개 단언·44개 기록을 base/후보에서 대조해 열기·내용/쪽/slot 소유·저장 재열기·그룹/해제 후 active owner·책갈피 결과가 같다. `pr8039`의 불완전한 렌더링 fixture만 HWPX 저장에서 `charPrIDRef: [0]` 미등록 오류를 양쪽 동일하게 보였고, 이 입력의 정상 저장 성공으로 보고하지 않는다. 원본과 검사 조건은 보존했다. 정상 업무 경로는 별도의 실제 Worker/Studio 편집·저장·재열기에서 검증했다.

최종 제품 Studio Node 2,630 PASS·기존 2 skip, TypeScript·build·실제 Worker 17개의 271개 단언·Studio 오류/취소/기존 문서 보존/복구·15쪽 표시·양쪽 편집 저장을 통과했다. 최종 workflow Python 196·게시 preflight Python 36·CI Node 334 PASS, editor/생성 binding 3 PASS다. 독립 AI SDK/설정 browser와 duration CLI는 변경되지 않은 해당 source의 base/후보 실제 실행 결과를 함께 사용한다. 전체 Rust·일반 release·최종 package 소비는 여전히 실행 중이며 이 시각 완료와 구분한다.

## 실제 포장 preflight에서 검출한 기존 정책 중복

현재 후보의 실제 preflight에서 JPEG encoder 중복을 검출했다. 고정 base `db5cd5bcc`의 목록도 14개 entry 중 encoder가 2개여서 같은 명령의 `--ref db5cd5bcc` 실행이 `duplicate package name/source`로 실패했다. 원 해결 #8191의 `6097253468` 뒤 메인테이너 보정 `f9dc41064a`에서 같은 항목이 재추가된 것을 대조했다. 원본 작업 브랜치는 qcms를 포함한 고유 14개였고 통합 후보는 중복 포함 15개였다. 정의가 완전히 같은 두 entry 중 뒤 항목만 제거해 승인한 고유 14개를 복원했다. 패키지 이름·버전·include·의존 간선·제품 Rust는 바꾸지 않았다. 기존 preflight의 중복 거부 조건을 유지하고 정책 계약·실제 source/포장·소비를 다시 실행한다.

preflight 출력은 검증 worktree 내부의 ignored output만 허용하므로 준비 스크립트도 그 경로를 사용하고 공유 증적 경로에서 참조하게 고쳤다. 앞선 경로 오류와 정책 중복 실패를 성공으로 바꾸어 기록하지 않으며, 이후 실제 포장 결과로 완료를 판정한다.

## 로컬 최종 게이트 완료

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

[통합 완료 보고](../report/task_m100_8174_report.md)에 현재 source·지원/보안 경계·실제 소비·전쪽 증적을 연결했다. 일반 release library의 12GiB 컴파일 제한 OOM은 16GiB 실행 한도에서 같은 opt3/LTO/cgu1로 재실행해 통과했다. 제품 정책·assertion·기존 사용자 서버는 바꾸지 않았다. 통과한 build·전체 회귀·archive는 유지했다.

독립 소비는 원본 테스트의 cases/fixtures/PDF 상대 구조를 유지하고 모든 입력 바이트를 대조해 공개 계약 54개를 통과했다. 실제 `.crate`는 Cargo `--no-verify`로 포장한 뒤 strict gzip·manifest/원본/digest·독립 컴파일/실행으로 검증했다. 실제 crates.io publish/dry-run·owner 확보·태그는 실행하지 않았다. 최종 원격 결과는 이 PR의 archive self-review·오늘할일·후속 comment에 exact head/merge SHA로 연결한다.
