# 9개 이관 이슈 — 소스 32의 최적화 교차 검증

생산 소스는 `2f46647df16da00365b22a1364cdc0b28b8344bc`다. 이후 문서·테스트 커밋은
생산 소스를 바꾸지 않는다. Native와 최적화 fresh WASM의 빌드 명령·소스 파일 해시·실행물
해시는 `native-producer.json`, `wasm-producer.json`에 고정했다. WASM은 `--no-opt`가
없는 locked wrapper의 release 빌드다. Native는 opt-level 3의 release-test 빌드다.

- Native SHA-256: `3836895000bbe4544802ff4bebb52a0a5ef7dfa3c4a049b07f06d4bd341336d7`.
- WASM SHA-256: `00206b93702fa10fc92203713a15d5992fee487cbcf9b9e996c605726489e4a8`.
- JS SHA-256: `23130c8002fe1aa4e6545910d92ef5716ccb92097747662d2b64ef129ff27b08`.

`qualified-documents.json`은 실제 완료한 원본별 실행만 담는다. 문서별 `native/`·`wasm/`에는
전쪽 TSV 원본과 `silhouette_manifest.json`을 그대로 보존했다. 원래 출력 경로를 유지한 manifest의
`tsv_sha256`과 복사한 TSV를 대조했다. 진단·실행 당시 repository head와 실제 생산 소스 SHA를
구분한다. 이 목록은 전체 9개 이슈의 완료나 최종 전체 Rust·보호 검증 통과를 뜻하지 않는다.

측정은 2026-11-22 KST, 96 DPI, Print profile, PDFium, 색상 경계 보정 전 2px 내용 실루엣이다.
판정에는 TSV의 공식 두 자리 표시값을 쓴다. 98% 미만 쪽을 평균으로 상쇄하지 않는다.
원본·독립 Hancom PDF Print의 출처는 상위 `input-provenance.json`을 따른다.

| 원본 | 전체 쪽수 | Native 최저 | fresh WASM 최저 | 회귀에서 보호하는 주요 범위 |
| --- | ---: | ---: | ---: | --- |
| 복학원서 HWP | 1 | 99.00% | 98.99% | 원서·서명·접수증의 소유·내용 순서, 중복 효과 없는 워터마크 |
| distribution_doc | 3 | 99.78% | 99.80% | host 단위 문구의 다음 줄에 놓이는 표, 들여쓰기, 제목·목록 소속 |
| 143E433F503322BD33 HWP | 1 | 99.94% | 99.94% | 다단 본문·OLE·각주와 문자별 폰트 추적의 원본 소유 |
| 86712 구판 HWPX | 64 | 98.69% | 98.69% | 저장 원점·빈 줄 소유, 재조판 행의 실제 컷과 자식 첫 최소 상자 |

## 직접 판독과 회귀 복원

`issue7756/issue7756-p001-detail.png`는 독립 Print·Native·WASM의 접수증 영역이다.
같은 배율로 점선·제목·본문·날짜·날인·하단 안내의 순서와 위치를 확인했다. Latin 글리프의
윤곽과 serif/sans 모양에는 차이가 남으므로 100% 일치로 표현하지 않는다. `issue_938`의 기존
SVG PNG·톤 검사는 유지하고, 이관됐던 layer 검사를 복원했다. 이미 bake된 PNG에는
`realPic`, brightness 0, contrast 0이 맞다. 예전 grayScale/-50/70 요구는 효과를 이중 적용하므로
원 해결 코드 `3aeaa5ba94`와 후속 `e5d3049199`의 계약에 맞춰 바로잡았다. 같은 PNG 바이트를
SVG와 layer가 사용하는지도 비교한다. 과거 #677의 전체 SVG 해시 역할은 원서·서명·수신인·
점선·접수증 표의 내용·포함·앞뒤 순서로 보호한다. 기존 PUA 원문·표시 검사(#948 등)는 유지한다.

`issue7757/issue7757-p003-detail.png`는 예산 표와 단위 문구·후속 안내다. 세 출력 모두 같은
내용과 줄·표 순서를 확인했다. 공식 기준은 기존 Microsoft Print PDF가 아니라 상위 provenance의
한컴 2020 Print다. 3쪽은 두 경로 모두 100.00%이며, 독립 PDF의 왼쪽 괘선은 약90.83 CSSpx다.
복원한 표 왼쪽 검사는 ±2px, 기존 유효한 단위 문구 ±1px 검사는 유지한다. host 문구가 표보다
앞에 있고 표가 본문 안에 있는 관계 및 정상 같은 줄 표·제목/목록 대조도 유지한다.

`issue7759/issue7759-p001-detail.png`에서 제목, 양쪽 단의 줄·차트·삽입 그림과 뒤 본문을
대조했다. 폰트 추적의 절대 layoutHash 재고정 대신 원본 문자 주소와 공개 런의 내용·순서·
문자별 단일 소유 계약을 복원한다. 해당 정식 검사의 실행 결과는 후속 기록에 추가한다.

86712 HWPX는 단계 2의 원본 바이트·독립 Print 실험과 같은 입력이다. 전쪽 Native 진단에서
소스 30→32는 26쪽만 97.36→100.00%이며 나머지63쪽은 같다. 최적화 Native·WASM에서도
전쪽64쪽이 98% 이상이다. `regulatory-hwpx-font-environment.json`은 같은 최적화 Native의
별도26쪽 출력에서 수집한 실제 선택 프로그램 metadata이며 글꼴 바이트나 설치 경로는 없다.
설치 글꼴 없는 회귀에서는 이를 명시적으로 공급해 검증된 내장 메트릭의 줄 소유를 검사한다.
시각 결과에는 실제 원본 TTF → 해당 TTC face → OTF 순서를 적용했으며 선택 순서를 바꾸지 않았다.

## 집중 실행의 한계

최적화 Native 라이브러리로 `rustc --test` harness를 만들고 최소 글꼴 bwrap에서 관련 정식
여섯 파일을 실행했다. 복원 전14 PASS/0 FAIL, 복원 뒤18 PASS/0 FAIL(4.50초)다.
추가된 네 함수는 복학원서 layer·접수증 관계 두 건과 배포용 표 배치 두 건이다.
이 실행은 선행 확인이며 최종 Cargo 자동 수집 suite·전체 nextest·Native Skia 3종·브라우저·
편람·보호 TSV를 대체하지 않는다. 전체 검증 결과는 실제 종료 뒤 별도로 기록한다.

## 구판 HWPX의 정식 회귀와 수정 전 실패

`issue_7765_hwpx_saved_flow_ownership`은 다음 세 역할을 검사한다.

1. 저장 빈 prefix·끝 줄·후속 제목과 표의 실제 쪽·앞뒤 소유.
2. 26쪽 중첩 표 첫 컷의 물리 빈 띠와 다음27쪽 문장의 단일 소유.
3. 모든64쪽에서 관련9개 표와 그 중첩 표의 셀·문단별 원문 순서·누락·중복.

실제 선택 프로그램 metadata를 명시적으로 공급하므로 설치 글꼴 없는 최소 환경에서도 실행한다.
단순히 부모 문단의 전체 글 뒤에 모든 자식 표를 붙이는 초안 비교기는 떠 있는 자식의 독립 흐름을
잘못 가정해 실패했다. 이를 원본 셀·문단 경로별로 비교하도록 보정했고, 내용을 정렬하거나 중복을
제거해 통과시키지 않았다. 최종 비교는 문단 안의 모든 글자 순서와 각 원본 경로를 유지한다.

고정한 수정 전 소스30 라이브러리는 2 PASS/1 FAIL(13.79초), 수정 후 소스32 라이브러리는
3 PASS/0 FAIL(1.30초)다. 전자의 실패는 의도한26쪽 첫 최소 상자이며31.3px와 독립 Print의
44.91064px 차이를 검출한다. 수정 뒤46.3px는 두 경로100.00% 및 확대 판독에 근거한 ±2px
조건을 충족한다. 테스트는 자식 컷의 다음 문장을 앞쪽에 복제하거나 앞 문장을 뒤쪽에 다시 그려도
실패한다. 실제 판독한 세 출력은 `issue7765-hwpx/p026-028-fragment-detail.png`다.

기록은 `regression-checks/hwpx-before-after.json`과 원본 로그에 있다. 테스트 첫머리 시각만
후속 갱신했으며 assertion은 두 실행에서 같다. 이 선행 직접 harness는 최종 Cargo 수집·전체 회귀를
대체하지 않는다. Native opt-level0 진단과 최적화 Native의64쪽 PNG는64/64쪽 바이트가 같아
직전 진단의 직접 판독도 같은 출력으로 재사용했다.

## 암호 문서와 폰트 추적의 회귀 복원

실제 암호 HWPX24쪽을 평문 대응본과 별도로 두 경로에서 렌더링했다. 각 경로의24쪽 PNG는
암호/평문 대응본 사이에서 모두 바이트가 같다. 실제 암호 원본은 Native 최저98.29%, fresh WASM
최저98.22%이며 독립 Print24쪽과 일치한다. `issue7754/actual-encrypted-validation.json`에
실제 입력 SHA·생산물·stdin 전달 명령을 기록하고 `issue7754/{native,wasm}/`에 원본 TSV와
PNG-pair manifest를 보존했다. pair manifest의 DPI가 null인 것은 재채점 경로의 표시이며 실제
렌더는96DPI·compat2024·Print다. fixture 비밀번호를 명령행·검증 증거에 넣지 않는다.

`hwpx_password_fixture`·`armor_cli_contract`·`mcp_password_contract`의 잘못된23쪽 전제는
독립24쪽으로 복원했다. 암호·평문 대응본의 모든24쪽 공개 문자 런도 비교해 내용·순서·소유를
보호한다. 원래의 인증 실패·암호 비노출·응답·다른 포맷 계약은 유지한다.

월간 수출입19쪽은 양쪽 최저99.18%, 온새미로 HWP/HWPX 각각46쪽은 양쪽 최저99.98%다.
143E1쪽99.94%와 함께 기존 #4961의4개 원본에서 `renderPinsDeferredToIssue7445`를 제거했다.
이미 유효했던607문자·41문자 및 counts/status·프로필은 유지한다. 전체 layoutHash를 새 값으로
재고정하는 대신 공개 런의 전체 문자·순서·단일 기록·실제 원본 문자 주소를 검사한다.

이관됐던 exact-face까지 같은 소유 검사를 처음 실행하면서 새 비교기의 잘못된 전제를 확인했다.
머리말·각주의 usize::MAX 주변 가상 문단은 실제 본문 주소가 아니다. Native와 WASM의
예약값 크기가 다름을 확인하고 `unavailable`·null 오프셋을 검사한다. 생성 번호1./2.는 독립
Print와 원본 번호 스타일 문단3/11에서 확인한다. 공개 문자 배치가 생성 번호의 문단 주소를
생략하는 것과 trace가 문맥 문단을 알려 주는 것을 혼동하지 않는다. 65534 셀 주소의 표 caption은
원본 caption의 실제 내용과 공개 런을 대조하고, 지원되지 않는 raw 주소를 만들어 내지 않는다.
주소가 없는 자동 꼬리말도 모든 문자를 검사하며 `unavailable` 계약을 유지한다. 원본 tcps 이후의
확정 불가능한 주소를 complete로 바꾸지 않는 기존 검사도 유지한다.

최적화 Native 라이브러리의 최소 글꼴 집중 실행은 API·CLI·MCP·폰트 추적24 PASS/0 FAIL,
6.32초다. 같은 fresh WASM의 Node contract/E2E16 PASS/0 FAIL,12.02초다. 원본 로그는
`regression-checks/password-fonttrace-native.txt`, `regression-checks/font-trace-node.txt`다.
이 선행 확인과 정식 Cargo 전체 검증은 구분한다. 글꼴 파일을 테스트 자원으로 추가하지 않았다.

## dash 회귀 다섯 건 복원

86712 HWPX의 기존 `issue_5804_dash_leader_glyphs` 세 검사와
`issue_5830_dash_leader_last_line` 두 검사를 복원했다. 글리프 수·em 전진 대역·
치환 선 미출력은 독립 Print와 일치하는 기존 계약을 유지한다. 전역 오른쪽 px 핀은
실제 소유 셀 안에 dash 런이 포함되는 관계로 바꿨다. 공유 helper는 실제 선택 프로그램
metadata를 공급하며 설치 글꼴을 요구하지 않는다.

독립 PDF 33쪽의 네 런은 0.49857·0.51280·0.49874·0.56977em, 34쪽의 세 런은
0.57260·0.57260·0.56960em이다. 원래 em 허용 대역을 변경하지 않았다.
`issue7765-hwpx/dash-p033-review.png`와 `dash-p034-review.png`에서 Print·Native·
fresh WASM의 같은 영역을 확대 판독했다. 33쪽 raw는 Native99.99/WASM99.98%,
34쪽은 양쪽99.97%다. 글리프와 끝점·표 행·뒤 문단을 함께 확인했다.

최적화 Native 라이브러리 최소 글꼴 집중 실행은 경고 없이 5 PASS/0 FAIL, 5.93초다.
`regression-checks/dash-native-minfont.{json,txt}`에 실제 명령과 결과를 보존한다.
최종 Cargo 자동 수집 및 전체 회귀 검증은 별도로 완료한다.

## 원본별 산식·본문·공통 corpus 복원

기존 `tests/issue_2279_layout_oracles.rs`의 일곱 역할을 같은 경로로 복원했다.
비용 끝 줄·수식 줄·편익·정성 설명·본문 끝 줄·줄 pitch·대안 표·조각 포함을 보호한다.
독립 Print의2035 산식은 HWP가26쪽3줄/27쪽1줄, HWPX가26쪽4줄/27쪽0줄이다.
과거 두 형식 공통2+2줄 전제를 고정하지 않는다. HWP 비용 끝 줄은26쪽, HWPX는25쪽이다.
`issue7765-{hwp,hwpx}/formula-p026-027-review.png`와 `body-p010-review.png`를
각각 세 출력으로 확대 판독했다. 수식 기호·줄 끝·셀 포함·다음 설명·본문 줄 간격·대안 표를
함께 대조했다. 본문 pitch는 고정 사각형 대신 원본20번 문단의 실제 일곱 줄에서 측정하고,
독립 Print의14pt 구간29.76~29.92px에 대한 기존 유효한 전진 대역을 유지한다.

기존 #7243의68.245px와193.868px는 현재 원본별 Print와 다르다. 그 검사의 중첩 꼬리·
빈 줄·뒤 표 보호 역할은 HWP의 `issue_7765_current_source_frames`와 HWPX의
`issue_7765_hwpx_saved_flow_ownership` 및 #2279 포함 관계에 연결한다. 이미 관계형인
내용 소유 검사는 유지하고, 독립 Print와 맞는28·29쪽 ±2px 검사는 유지했다.
별도 이름으로 잘못된 과거 좌표를 복제하지 않는다.

`issue_1891`의86712 두 원본64쪽 왕복을 복원했다. 모든64쪽의 공개 런을 원본 소유 주소별로
모아 글자 순서·내용·누락·중복도 검사한다. HWP5-origin 재저장은 저장 줄 계약으로 재배치할
수 있으므로 단순 쪽별 전체 문자열 동일성은 계약이 아니다. 실제로 HWPX의10쪽 마지막
`o 규제대안의 비교`가 다음 쪽으로 이동하는 최초 비교기 오류를 확인했고, 글자를 버리거나
정렬하지 않고 소유 주소별의 순서 있는 전체 문자열을 비교한다. 원본 자체의 쪽 소유는 위
#2279/#7765 회귀가 별도로 보호한다. 다른 원본의 기존 왕복 계약은 유지한다.

`issue_7080`의 기존 matrix에86712 두 형식을 복원했다. 각 독립 PDF 전체에서 U+318D46개의
전진을 재측정하면0.999997~1.000001em이다. 기존 전각 검사의 허용 대역은 변경하지 않았다.
네 정식 파일의 최소 글꼴 집중 실행은22 PASS/0 FAIL,8.55초다. 실제 로그는
`regression-checks/regulatory-restoration-native.{json,txt}`에 있다.

공통 이관 목록의 대상11행과86712의 별도 body/off-canvas/text-overlap 제외를 제거했다.
10개 평문 원본의 oracle/render-page 쪽수와 byte-verified Print 참조를 복원했다. 암호 원본은
명시적 비밀번호를 받는 API·CLI·MCP 검사로 보호하며 일반 스캐너는 기존 파싱 계약을 유지한다.
#7746이나 다른 문서의 이관 이슈를 자동 종결하지 않는다.

최소 글꼴의 일반 기본 설정과 별도 Print 설정에서10개 원본을 모두 스캔했다. 용지 밖·본문 바닥·
셀 넘침은0건이며 글자 겹침은 시장구조조사106쪽의 한 쌍만 있다. 같은 셀의 `세분화`/`재편입`은
독립 Print에도 촘촘한 두 줄로 존재한다. 원장에는 이1건만 반영하고 이전 대상들의 양수 상한은
실제0건으로 조였다. `issue7755/p106-compact-lines-review.png`를 세 출력으로 직접 확인했다.
이 쪽 raw는 Native/fresh WASM 각각99.94%다. 원본 로그·일반 설정·진단 원문을
`regression-checks/restored-corpus-*`에 보존한다. 최종 Cargo 전수 검증은 계속 진행한다.

## 소스32의 9개 이슈 전체 측정 종료

10개 평문 원본 합계583쪽을 Native와 최적화 fresh WASM에서 모두 측정했다.
전쪽 최저는 Native98.08%, fresh WASM98.11%이며98% 미만·누락·쪽수 불일치는 없다.
공식100.00% 표시는 각각384쪽과378쪽이다. 실제 암호 원본24쪽을 별도로 포함하면 각607쪽이며
암호/평문 PNG는 같은 경로에서24/24쪽 바이트가 같다. 이 숫자는 고정 생산 소스32의 결과다.

시장구조조사는 사용자가 제공한 수동 한컴2024 Print315쪽과 compat2022로 대조했다.
두 경로 모두 최저98.40%, 표시100.00%는 Native211쪽·WASM212쪽이다. Native의 원본
manifest와 fresh WASM의10개 원본 chunk manifest·TSV를 `issue7755/`에 보존했다.
WASM 통합 TSV는1~315쪽이 정확히 한 번씩 있고, 원본 chunk마다 원문·PDF 전체315쪽
대조를 확인한다. `chunked-manifest.json`은 원본 chunk 영수증의 집계이며 단일 실행의
manifest로 위장하지 않는다. `qualified-documents.json`과 두 `*-full-final-validation.json`에
실제 종료·생산물·명령·입력 SHA를 기록했다.

Native공식3종·fmt·workspace all-targets Clippy는 완료했다. 전체 nextest는 컴파일 중
최신 devel 유입으로 취소했으며0건 실행이다. 이를 전체 회귀 통과로 집계하지 않는다.
이전 보호표의 Native23문서 전2023쪽은32쪽 하락으로 실패했다. 원문은
`gates/protected-native-prior-baseline/`에 보존하고, 최신 devel의 새 생산물과 상향 보호표로
다시 검증한다. 새 기준에서 이 결과들을 재사용해 완료로 선언하지 않는다.
