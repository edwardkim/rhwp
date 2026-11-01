# #8413–#8416 공통 구현·검증 기록

상태: 진행 중. 전체 구현·시각 검증·PR 준비 완료가 아니다.
기준과 범위: [수행 계획](../plans/task_m100_8413.md),
[원인 조사](../tech/investigations/issue-8413/README.md).
공통 계획 승인 기록은 `f1c08f7112`다. 메인테이너의 추가 요구인 PDF 열기 암호 입력을
#8415에서 먼저 연결하고, 나머지 공통 clip·이미지·명령 수 처리를 계속한다.

## #8415 생산·소비 경로와 변경 중인 계약

- Studio 암호 입력 → 공통 Worker의 options snapshot → `PdfImportRequest` →
  `PdfImportOptions.password` → bounded `PdfDocument` 인증 순서로 전달한다.
- CLI는 기존 `--password`/`--password-stdin` → supervisor의 비공개 입력 pipe →
  작업자 옵션 → 같은 parser를 사용한다. 일반 `convert`·Core의 password 생성자도
  PDF 요청 세션을 사용해 Core 초기화까지 한도를 유지한다.
- 인증된 uncompressed 객체를 로드할 때 문자열·stream을 복호화한 뒤 캐시한다.
  이미지·mask·content·ObjStm의 bounded decoder가 같은 평문을 소비한다. XRef와
  암호화 사전 자체는 복호화하지 않으며 Metadata의 EncryptMetadata 설정을 구분한다.
- PDF Standard R2–R6를 대상으로 길이·알고리즘·crypt filter 조합을 먼저 검증한다.
  현재 handler가 표현하지 못하는 혼합/Identity crypt filter는 명시적으로 미지원으로
  남기며 다른 알고리즘으로 잘못 복호화하지 않는다. 인증 성공 후 복사·편집 플래그로
  추가 차단하지 않는 메인테이너 결정을 적용한다.
- 인증 KDF의 유한한 최악 작업량과 scratch를 먼저 예약한다. 실제 현재 heap/RSS가 아니다.
  복호화 전 stream/codec 단일 cap과 누적 decoded·작업량을 확인한다.
- 옵션 Debug는 암호를 가리고, 공통 session이 보존하는 재열기 options에서는 암호를 제거한다.
  Studio는 잘못된 암호 재입력·취소·성공 시 입력을 비운다. 출력 문서나 진단에 암호를 넣지 않는다.
- 독립 근거: [PDF Association의 ISO 32000-2 §7.6 정오표](https://pdf-issues.pdfa.org/32000-2-2020/clause07.html).
  stream 복호화는 filter decode보다 먼저이며, ObjStm의 데이터와 내부 문자열을 중복 복호화하지 않는다.

## 현재 실행한 검사

최종 키 필드 보정까지 포함해 Native 23건·실제 CLI 42건·fresh WASM 29건과
실제 Studio 검사를 재실행해 통과했다. 암호 입력 기능의 로컬 검증 결과이며 공통 작업 전체의
완료 판정은 아니다.

| 검사 | 결과·한계 |
| --- | --- |
| 공통 Worker options 검사 | 27 PASS. 암호 snapshot·오입력 재시도·재열기 설정에 암호를 보존하지 않음 |
| Studio 단위·TypeScript | 2,702 PASS / 0 FAIL / 2 기존 skip, TypeScript 통과 |
| Rust Parser/IR 인증 회귀 | Parser/IR 17 + CLI 6 = 23 PASS. R2–R6 사용자/소유자·한글·추가 바이트·낮은 cap·손상 사전/미지원 방식 포함 |
| Native 실제 CLI | 인증 17건 + 확장 25건 = 42건 PASS. 추가 바이트 보정 후 암호 누락 exit 2 / 오입력 exit 1·저장 보존 재확인 |
| 최적화 fresh WASM | 29개 가져오기/실패 사례 PASS, 저장 재열기·옵션 타입/길이·낮은 예산·실패 후 복구 확인 |
| 실제 Studio | 암호 요구·오입력·Enter 확인·IME 조합 보호·Escape 취소·기존 문서 보존 통과 |
| 기존 UI 회귀 | #8398 진행 커서·취소·일반 문서, #8411 대화창·키보드·한도·재시도 모두 통과 |
| 인증 1쪽 시각 제어군 | 96dpi Native/fresh WASM raw 각각 100.00%, 본문·이미지 직접 판독 완료 |
| 실제 입력 127쪽 | 수정 전 오류 재현 완료. 곡선 clip·큰 이미지·명령 수 보정과 수정 후 전체 검증은 남음 |

암호 제어군은 pikepdf로 생성한 동일 내용의 평문·R2/R3/R4/R5/R6와 빈 암호/추출 제한본이다.
암호는 테스트 전용 합성 값이며 실제 사용자 암호는 읽거나 기록하지 않았다.
임시 입력·로그·진단 도구는 `output/pdf-import-errors-20261031/`에 두었다.

### 실제 인증 경로 진단

초기 Native 실행에서 누락·오입력 분리는 통과했지만 정상 인증 뒤 추출이 실패했다.
`load_uncompressed_object_impl`의 메모리 입력 분기가 reader 분기의 후처리를 거치지 않고
반환하던 원인을 확인했다. 두 분기가 캐시 삽입 전에 같은 `decrypt_loaded_object`를 소비하게
보정했다. 복호화 `StreamData`에는 원래 요청 예산을 유지하고, bounded ObjStm은 재복호화를
시도하지 않는다. 이 보정 뒤 backend의 R2/R4/R6 인증·1쪽 인식·본문 추출을 실제 실행해
정상임을 확인했다. 압축 해제 후 36 MiB인 암호화 stream은 `stream output bytes` 한도에서
거부됐다. 최종 CLI·WASM 결과는 아래 후속 검사로 확인했다. 이 진단만으로 완료하지 않았다.

R5의 기존 handler는 UE/OE로 감싼 실제 문서 키를 풀지 않고 SHA-256 중간 키를 반환했다.
독립 pikepdf/QPDF 생성본에서 정상 인증 뒤 내용 추출 실패를 재현했고,
[QPDF 참조 구현](https://github.com/qpdf/qpdf/blob/main/libqpdf/QPDF_encryption.cc)의
`recover_encryption_key_with_password`와 대조해 R5의 키 unwrap을 보정했다. 기존 단위 검사의
잘못된 중간 키 기대값 2건도 독립 Python cryptography로 생성한 AES-CBC UE/OE와 문서 키로
교체했다. 실제 R5 파일의 사용자/소유자 인증·내용 추출·저장 재열기는 최종 Native/WASM에서 통과했다.

긴 한글 암호 제어군에서 UTF-8 문자 경계로 127바이트를 줄이던 기존 처리도 인증 실패를
일으켰다. 규정된 첫 127바이트를 그대로 hash에 전달하도록 보정하고 43글자 한글 암호를
독립 pikepdf 입력에 추가했다. 기본 R6 한글 암호와 구분해 Native/WASM에서 재검증했고 통과했다.

### #8415 Native 제품 실행

최신 `release-test` + `native-skia`에서 인증·CLI 검사 17건과 추가 사용자/소유자·한글·
한도·기존 출력 보존·저장 내용 검사 22건이 모두 통과했다. R2–R6의 사용자/소유자 인증,
빈 암호, UTF-8 및 127바이트 경계 한글 암호를 포함한다. `import-pdf`와 `export-hwpx`,
HWP/HWPX 저장의 실제 CLI를 실행했다. 7개 저장본에서 본문이 보존되고 이미지 PNG 해시가
동일했다. 짧은 기본 한글 암호만 통과한 것을 전체 Unicode 경계 통과로 확대하지 않았다.

합성 1쪽 제어군의 독립 평문 PDF와 암호 가져오기 HWPX를 96dpi Native Skia로 비교했다.
기본 폴백은 raw 99.92%였으나 직접 판독에서 글자 굵기 차이가 보여, Helvetica → Arial과
실제 `/mnt/c/Windows/Fonts/arial.ttf`를 명시해 다시 비교했다. 이 조건의 Native raw는
100.00%이며 본문과 빨간 그림을 직접 대조했다. 원본 PDF는 표준 Helvetica 이름만 포함하고
글꼴 프로그램은 내장하지 않는다. 이 대체 환경을 WASM에도 동일하게 적용해 raw 100.00%와 직접 판독을 확인했다.
이 1쪽 제어군은 사용자 원본 127쪽의 시각 완료 근거가 아니다.

Studio 단위 검사: 2,702 PASS / 0 FAIL / 2 기존 skip, 공통 Worker 검사: 27 PASS,
TypeScript·변경 문서 7개의 내부 링크 검사 통과. Native 제품 lib/bin Clippy와 전체 fmt 검사도 통과했다.


### 실제 Studio 키보드와 증적

첫 실제 브라우저 검사에서 암호 인증 자체는 성공했지만, 입력란의 Enter가 제출되지 않았다.
기존 공통 모달은 모든 input의 Enter를 확인 동작에서 제외한다. 공통 capture 경로에 기본값
false인 입력별 확인 hook을 두고 PDF 암호 입력만 연결했다. 다른 입력·JSON·선택/펼치기
동작은 유지하며 IME 조합 중 Enter는 제출하지 않는다. 보정 후 실제 Worker 인증, 편집 문서
게시, 입력 취소 시 이전 generation 유지가 모두 통과했다. 정상·오류 대화창 PNG도 확인했다.

최적화 WASM SHA-256: `cf203025645070b658e79d048503df3030a47d5f587660cde022bbdfedff9c1f`.
`pkg/`와 Studio `public/`의 바이트가 같고, #8415/#8398/#8411 브라우저 검사에서도 실제 응답 SHA를
같은 값으로 확인했다. 인증 matrix의 기본 평문 검사는 최초에 SVG가 글자별 text 요소인 점을
고려하지 않은 진단 assertion에서 실패했다. 텍스트 노드의 순서를 연결해 본문을 비교하도록
진단만 바로잡고 27개 사례를 통과했다. 제품 출력을 바꿔 검사를 맞추지 않았다.

정식 공개 증적은 [암호 인증 제어군](../../pdf/issue8415/authentication/README.md)과
[검증 산출물](assets/issue8415/password-authentication/README.md)에 보존한다.
사용자 원본의 성공을 의미하지 않으며, 암호화 원본 57쪽에는 곡선 clip·이미지 예산이라는
독립적인 후속 장애가 있다. 공통 계획의 나머지 오류 해결과 원본 127쪽 검증 전에는 PR을
제출하지 않는다.

CLI 전역 입력 암호와 같은 종료 코드 계약으로 `import-pdf`의 암호 누락은 2, 오입력은 1로
맞췄다. 출력 암호 거부는 유지하며 기존 검사 메시지만 새 입력/출력 구분에 맞췄다.
실제 stdin 전달·오입력 시 기존 출력 보존·정상 재시도 검사를 정식 CLI suite에 추가했다.


### 원본 V5 인증 키 필드의 추가 바이트

합성 제어군 통과 뒤 원본 암호화 57쪽을 실제 CLI로 다시 실행하자 `PDF_PARSE_FAILED`가
발생했다. backend 원인은 `invalid encryption key material length`였다. 원본의 `/O`·`/U`는
127바이트이며 앞 48바이트 뒤에 79바이트가 추가돼 있다. 독립 pikepdf/QPDF는 빈 암호로
이를 연다. [QPDF Encryption::initialize](https://github.com/qpdf/qpdf/blob/main/libqpdf/QPDF_encryption.cc)는
V5의 O/U/OE/UE/Perms가 필수 길이보다 긴 경우도 수용하고 인증은 정의된 접두부만 소비한다.

이에 V5는 필수 접두부 길이를 확인하고 handler가 복사하기 전에 각 필드를 48/32/16바이트로
정규화했다. R2–R4의 정확한 길이 검사와 요청 내 원본 파싱·복사 예산은 유지한다. 짧은 필드를
0으로 늘려 인증하지 않는다. 사용자 원본의 필드나 키를 복사하지 않고, 공개 R6 제어군에
incremental 암호화 사전을 추가한 `password-padded-fields.pdf`를 만들었다. 독립 QPDF 인증,
수정 backend의 사용자·소유자 인증을 검사한다.

보정 backend에서 원본의 빈 암호 인증과 57쪽 인식을 확인했다. 기본 backend 예산으로 한
본문 추출은 `parsed object count`에서 중단됐다. 이 probe는 제품의 페이지별 정책과 다른
진단이며, 원본의 가져오기·렌더링 성공으로 해석하지 않는다. 최종 제품 경로를 다시 실행한다.

최종 Native 제품 실행에서도 원본 57쪽은 인증 뒤 `PDF_RESOURCE_LIMIT: parsed object count`에서
중단된다. 공통 예산 검토에서 함께 다뤄야 할 후속 한도이며, 암호 입력으로 모든 문서 오류가
해결됐다고 안내하지 않는다. 추가 바이트 제어군은 사용자/소유자 인증·HWPX 저장·재열기,
독립 평문 PDF와 Native 1쪽 raw 100.00%를 확인했다. 8개 저장본의 그림 PNG 해시도 동일하다.


### 열기 암호 추가 요구의 로컬 완료

최종 확인 시각: 2026-11-01T00:29:10+09:00. Native 23건(기존 Parser/IR 17 + CLI 6), 실제 CLI 42건,
최적화 WASM 29건, Studio 인증 및 기존 대화창/진행 상태 E2E 3개가 통과했다.
추가 바이트 제어군을 포함한 최종 Native/fresh WASM의 1쪽 raw는 각각 100.00%다.
같은 본문·그림을 PNG에서 직접 대조했다. 검증 중 함수 선언의 줄바꿈만 rustfmt 형식으로
정리했으며 인증·복호화 실행 로직은 바꾸지 않았다. Native Clippy는 이 최종 소스를 검사했다.

확인한 동작은 PDF Standard Security R2–R6의 암호 입력·오입력·정상 재시도·취소·저장 재열기다.
새 JS/WASM을 로컬 Studio에 동기화했다. 구현·매뉴얼·공개 합성 자료만 로컬 task 브랜치에
기록하며 원격 push·PR·merge는 수행하지 않는다. #8413–#8416 공통 작업은 계속 미완료이고,
원본 57쪽의 `parsed object count`와 다른 원본의 곡선 clip·이미지·명령 예산을 이어서 처리한다.

## 자원 한도 확대 — 2026-11-01

메인테이너가 자원 한도를 늘려 처리를 계속하도록 지시했다. 공통 `PdfImportPolicy`의
표준 프로필을 조정하고 CLI·WASM·Studio가 같은 결과를 소비하게 한다. 절약 프로필과
명시적으로 낮춘 host cap, 실패 후 복구, OS/Worker 종료 한도는 유지한다.

변경 전 신뢰하는 Rust host 정책으로 독립 진단을 실행했다. 원본 57쪽의 텍스트·경로·
이미지 handle 전수 추출은 parsed objects 81,182, entries 166,060, operators 98,694,
decoded work 408,295,629바이트였다. 그림 디코딩·IR·저장 비용은 이 수치에 포함하지 않는다.
기본 50,000은 간접 객체 수가 아니라 사전·배열의 scalar를 포함하므로 정상 문서도 넘는다.
`f87pro.pdf`는 실제 backend에서 623,266 operators와 475,961 path operations를 사용했다.
쪽당 250,000 operators / 최소 80,000 path operations의 축소는 이 복잡도를 반영하지 못한다.

표준 후보: objects 200,000, operators 요청 상한 8,000,000과 경로 요청 상한 1,000,000을
쪽수로 축소하지 않음, 단일 그림 64,000,000픽셀, 배치 누적 2,000,000,000픽셀,
stream/codec 단일 예약 256 MiB. 8K 원본의 33,177,600픽셀과 RGBA 132,710,400바이트,
37쪽 원본의 배치 합계 1,388,854,656픽셀을 수용할 유한한 등급이다. 각 최대값을 동시에
소비할 수 있다는 보장은 아니며 누적 decoded 2 GiB·작업량·프로세스 제한도 적용한다.
공유 이미지의 반복 디코딩·저장 개선 필요성은 없어지지 않는다.

높은 한도로 원본 제품 parser를 실행한 결과, 57쪽은 기존 객체 한도를 넘은 뒤 **10쪽의
회전·비균등 배율 텍스트 미지원**에서 실패했다. 큰 이미지 원본은 첫 픽셀·버퍼 제한을
넘은 뒤 lowering 실패가 나타났다. 한도 증가만으로 네 원본이 모두 성공했다고 판정하지
않는다. lowering의 상세 원인은 opt-in 진단에만 남기고 일반 오류 문자열은 유지한다.
현재 한도 변경·경계 검사·새 WASM 및 실제 제품 재검증을 진행 중이다.


후속 진단에서 큰 이미지의 lowering 실패는 별도 16,000,000픽셀 고정 검사였다.
`extract.rs`가 선언·합성 픽셀을 현재 요청 정책으로 검사한 뒤 만드는 private
`ExtractedPage`만 lowering이 소비하므로, lowering의 중복 상수를 제거하고 PNG 인코더의
동일 요청 버퍼/작업량 검사를 유지했다. `f87pro.pdf`에는 37,310개 경로가 있어 추출 후
고정 20,000개 검사도 후속 차단 원인이었다. 이미 경로 작업량·추출 저장소를 제한하고
IR materialization 전에 문서 자원 ID 상한을 검사하므로 이 별도 고정 개수 검사를 제거했다.
텍스트 span 상한·개별 경로 상한과 모든 host cap은 그대로다.

합성 제어군의 수정 전 WASM은 객체·명령·픽셀 예산에서 각각 실패했다. 초기 객체 제어군의
빈 Flate stream은 한도 확대 후 별도 decoder 오류를 드러냈다. 원본을 보존하고 내용이
없는 `q Q` stream을 가진 별도 `resource-objects-nonempty.pdf`로 객체 계수만 검증한다.
이 진단 입력 변경을 원본 오류 해결로 기록하지 않는다. 원본 네 파일은 변경하지 않았다.


### 확대 정책의 Native 재검증

- 객체·명령·이미지 예산, 인증, 요청 수명과 실패 복구를 포함한 focused Rust 163건 PASS.
  새 8K 검사에서 raw Flate 출력은 codec scratch 16 MiB보다 커도 scratch를 그만큼 쓰지
  않으므로 처음 만든 거부 기대가 틀렸다. 실제 codec scratch보다 작은 1 KiB cap으로
  수정해 거부를 확인했다. 제품 한도를 낮추거나 구현을 바꿔 테스트를 맞추지 않았다.
- 실제 CLI: 60,028 객체 값 / 640,000 명령 / 8K 두 쪽 제어군 모두 가져오기와 HWPX
  재열기 성공. 감독에서 관측한 최대 RSS는 각각 31,821,824 / 68,780,032 / 52,432,896바이트다.
  주기적 관측치이며 정확한 최고점이나 일반 문서의 메모리 보장이 아니다.
- 8K 두 쪽의 독립 원본 PDF 대비 96dpi Native raw는 각각 100.00%다. 회색 그림과
  페이지 여백을 직접 대조했다. fresh WASM도 두 쪽 raw 100.00%이며 실제 Studio에서 열기를 확인했다.
- 공통 Worker 27건과 Studio TypeScript 통과.

원본 전체를 새 Native CLI로 실행한 후속 실패는 다음과 같다. 모두 요청 전체 실패이며
앞부분만 가져온 결과를 성공으로 저장하지 않았다.

| 입력 | 현재 첫 실패 | 이번 조정으로 통과한 경계 |
| --- | --- | --- |
| img_clipping_01.pdf | 곡선 clip 미지원 | 한도 변경으로 곡선 지원을 대신하지 않음 |
| img_bug_01.pdf | 9쪽 `document decoded bytes` | 33,177,600픽셀·큰 버퍼·lowering 중복 한도 |
| PDF_ENCRYPTION_UNSUPPORTED.pdf | 10쪽 회전·비균등 배율 텍스트 미지원 | 인증·parsed objects 50,000 제한 |
| f87pro.pdf | `limited image mask DecodeParms` | 623,266 명령·475,961 path operations·37,310개 경로 |

큰 이미지 원본은 decoded work 2,091,548,519바이트를 쓴 뒤 99,532,800바이트 추가 예약이
2,147,483,648바이트 상한을 넘었다. 동시에 codec work도 901,890,658이었다.
개별 한도만 추가로 높이는 것으로 전체 성공을 보장할 수 없으며, 승인된 공통 계획의 공유
이미지 재사용과 저장 중복 제거가 남는다. 2 GiB는 live heap이 아닌 누적 작업량이다.
기능 미지원과 실제 남은 자원 실패를 구분하며 #8413–#8416 완료·PR 준비 판정은 보류한다.

### 확대 정책의 fresh WASM·Studio 확인

최적화 WASM SHA-256: `048d8d735cd47c7df3d89a1ab3606085bbee19ada6a2e76b205aa13f176a033f`.
직접 WASM 14건(세 제어군 가져오기·재열기, 낮은 한도 거부·정상 복구, 암호 재시도) 통과.
실제 Studio Worker에서 세 제어군 열기, 암호 입력·재시도·취소 E2E,
기존 대화창의 테마·작은 화면·키보드·timeout·재시도 E2E가 통과했다.
Native Clippy와 rustfmt도 통과했다. [8K 전쪽 TSV와 PNG](assets/issue8413/resource-policy/README.md)를 보존한다.
원본 네 파일은 WASM에서도 위 표와 같은 범주의 오류로 중단됐으며 각각 다음 정상 문서를
다시 열 수 있었다. 원본 37쪽의 WASM 누적 메모리 capacity는 365,297,664바이트였으며
2 GiB 실패는 누적 예약 카운터의 실패다. live heap 2 GiB 사용으로 해석하지 않는다.

이 기록은 자원 정책 절편의 로컬 검증이며 전체 회귀·최종 PR 게이트 완료가 아니다.
원본의 반복 이미지 처리와 기능 미지원은 승인된 공통 계획에서 계속 처리한다.

### 반복 이미지의 실제 작업량 보정

원본 37쪽에서 8K XObject 하나가 36쪽, 3999×442 XObject 하나가 35쪽의 Resources에
재사용된다. 객체 번호와 문서 수명으로 같은 source를 식별할 수 있으며 문서 이름·쪽 번호에
따라 분기할 이유가 없다. 같은 PDF 요청의 bounded LRU에 self-contained image source를
보관하고, 불변 픽셀을 Arc로 공유한다. 마스크·ICC 변환 결과까지 공유하되 named ColorSpace,
Indexed base와 현재색 stencil은 scope 의존성을 피하기 위해 캐시 대상에서 제외한다.
배치별 opacity·CTM·clip은 캐시 이후 항상 적용하고, 픽셀 변경 시 budgeted copy-on-write를
사용한다. PNG 인코딩 결과도 같은 불변 source에서 공유하되 반환하는 압축 바이트 복사는
실제 요청 예산을 소비한다. 픽셀/색상 변환은 PNG 캐시를 무효화한다.

선언 픽셀·배치 누적 계수는 캐시와 별개로 매번 검사한다. 누적 decoded 2 GiB·codec work
1,000,000,000·OS/Worker 제한을 올려 반복 처리 비용을 숨기지 않는다. 아래는 구현 당시의 중간 진단과 최종 자원 절편 검증 결과를 구분한 기록이다.

첫 회귀 실행에서 기존 #7974의 서로 다른 현재색 stencil 재사용 검사가 실패했다.
BitsPerComponent를 생략한 ImageMask도 있으므로 비트 수를 stencil 판정으로 쓴 가정을
제거하고 실제 ImageMask 선언으로 공유 대상에서 제외했다. 빨강·초록의 독립 원본 기대값은
바꾸지 않았다. 직접 ColorSpace 배열은 decoder가 페이지의 색상 이름 사전을 조회하지
않으므로 불필요한 사전 복사를 없애며, 이름/간접 참조의 scope 해석은 유지한다.

#8190의 원 통합 커밋 `e7a0dec55c`와 #8183 stage 3 기록을 확인했다. 기존
`valid_repeated_jpx_paints_exhaust_work_and_next_request_recovers`는 같은 1655×2339 JPX를
세 번 디코딩하여 1,000,000,000 작업 단위를 초과한다는 전제였다. 공유 후에는 이 입력이
성공하므로, 소스의 실제 픽셀·저장 결과와 Native/fresh WASM을 먼저 확인한 뒤 반복 배치 보존과
명시적으로 낮은 codec cap 거부·복구를 검사하는 계약으로 갱신했다. 변경 전 양쪽 raw 100.00%와 직접 판독을 완료했다.
37쪽 원본은 ColorSpace 배열을 간접 객체로 참조한다. 실제 decoder와 동일하게 참조를 먼저
해석한 다음 배열/이름을 구분하도록 보정했다. 해석 실패를 가져오기 성공으로 숨기지 않는다.

공유 보정 후 Native에서 원본 37쪽의 가져오기·HWPX 저장·재열기가 성공했다.
감독 관측 wall 21,493ms, peak RSS 419,172,352바이트, 저장 46,292,946바이트이며
최종 누적 decoded 1,478,483,157바이트·codec work 491,881,360으로 기존 상한 이내다.
이번 관측은 별도 성능 벤치마크의 전후 비율이 아니다.

96dpi Native 전쪽 raw 최저는 기본 글꼴 공급에서 99.69%였으나 직접 판독에서 Gibson 글꼴의
대체 굵기 차이를 확인했다. 원본 CFF subset의 윤곽·advance를 로컬 검증용 OpenType으로
모으고 Windows 글꼴과 함께 공급한 결과 제목·본문 굵기가 복원됐으며 전쪽 최저 99.70%다.
추출 글꼴은 ignored output에만 두며 배포·fixture에 포함하지 않는다. 임베딩된 TTF subset은
name/cmap이 없어 그대로 host 글꼴로 사용할 수 없었고 Windows Arial/Consolas를 공급했다.
37쪽 모두를 비교했으며 9쪽과 최저 32쪽의 이미지·제목·본문·강조 문구를 직접 대조했다.
새로운 원본 문서 회귀를 등록하거나 전체 공통 작업의 완료로 승격하지 않는다.

별도의 #8190 JPX 반복 배치 제어군은 Native 1쪽 raw 100.00%다. source의 글자가 bitmap에
포함되어 host 글꼴에 의존하지 않는다. fresh WASM도 raw 100.00%이며 직접 판독 후 기존 작업량 검사를 갱신했다.

직접 판독의 제한: 글꼴 공급 후에도 32쪽 `enter KeX` 등의 단어 간격과 빨간 강조 문구의
미세 배치가 원본과 완전히 같지는 않다. 이를 raw 99.70%로 면제하거나 전쪽 시각 완료로
판정하지 않는다. 이번 자원 실패 해소와 공통 작업의 문자 배치·최종 시각 gate를 구분한다.

캐시 적용 전 정책-only 최적화 WASM `048d8d735cd47c7df3d89a1ab3606085bbee19ada6a2e76b205aa13f176a033f`로
9쪽만 가져와 같은 Native/글꼴 조건으로 출력한 PNG와, 공유 후 37쪽 중 9쪽 PNG의 SHA-256은
모두 `35f3ffeec1d3d9d04c4f055241fde0b1a5136f54eef9847b7dd7849849edffb3`였다.
이 페이지의 출력 보존 근거이며 다른 페이지·전체 글꼴 fidelity의 증명으로 확대하지 않는다.

픽셀 한도 확대에 따라 한쪽으로 매우 긴 이미지의 HWP 1/7200 inch 크기와 signed crop
좌표 변환도 checked arithmetic으로 검사한다. 정상 픽셀 수 이내여도 저장 형식의 좌표
범위를 넘으면 PNG 인코딩·BinData 추가 전에 거부한다. 좌표를 wrap/clamp하여 저장하지 않는다.


### 자원 절편 최종 검증 — 2026-11-01

정책 확대 커밋: `62720ae4052693605e6e3e70e53fbc451c04f6ff`.
최종 공유 구현의 제품 diff SHA-256:
`13488c6d73fc0526a13c530eada0ce50e2d78e44fc21485b2f4dc8c79218473a`.
최적화 WASM SHA-256:
`1e6b3794e268a1ea09f47af4e2848c6eef404523e21d5674f39ad0c99a0fb35a`.

- 관련 Native 검사 169건 통과(마스크·패턴·affine clip·이미지 배치·정책·operator·새 좌표 범위 검사).
- 갱신한 JPX 반복 배치 검사를 포함한 코덱 모듈 6건 통과. 중복 5건을 제외하면 최종 계약 170건을 확인했다.
  새 검사의 JSON 키를 camelCase `codecWorkUnits`로 바로잡았으며 제품 예외를 추가하지 않았다.
- 원본 37쪽 최종 Native 가져오기·HWPX 저장·재열기 성공. wall 24,012ms, peak RSS 418,750,464바이트.
  누적 decoded 1,478,483,157바이트·codec work 491,881,360으로 유지한 상한 안에 들어왔다.
  직전 성공 실행과 HWPX SHA-256이 동일하다.
- fresh WASM 원본 37쪽 가져오기·저장·재열기 성공. 세 작업 합계 28,107ms,
  linear memory capacity 554,827,776바이트(프로세스 RSS가 아님). 직접 WASM 제어군 14건도 통과했다.
- 실제 Studio에서 기본 30초는 `PDF_IMPORT_TIMEOUT`; 고급 설정을 60초로 늘린 실행은 27,299ms에 37쪽 열기 성공.
  시간은 부하에 따라 달라지며 기본 30초에서 항상 성공한다고 주장하지 않는다.
- 동일 최적화 WASM의 암호 입력·오입력·Enter 재시도·Escape 취소, 대화창 테마·작은 화면·키보드,
  실제 1초 timeout·재시도 E2E 통과. 네트워크로 받은 WASM 해시도 일치했다.
- 원본 FontDescriptor의 ascent/descent를 적용한 최종 글꼴 환경에서 Native 37쪽 최저 99.70%,
  fresh WASM 37쪽 최저 99.75%. JPX 1쪽은 양쪽 100.00%다.
  [전체 TSV·대표 PNG·직접 판독과 실행 근거](assets/issue8413/image-sharing/README.md)를 보존했다.

`img_bug_01.pdf`의 자원 부족은 이번 한도 확대와 반복 디코딩 공유로 해소했다.
다른 원본의 곡선 clip, 회전·비균등 텍스트, image mask DecodeParms와 위 문자 간격 차이는 남아 있다.
따라서 이번 자원 절편의 성공을 #8413–#8416 전체 완료나 PR 제출 가능 판정으로 확대하지 않는다.

최종 Native Clippy(`--lib --bin rhwp --features native-skia -D warnings`), rustfmt,
변경 문서 링크 검사와 `git diff --check`도 통과했다. 원격 push·PR 생성은 하지 않았다.

## 쪽별 진행 표시 추가 — 2026-11-01

사용자 지시에 따라 무한 좌우 이동 진행바를 실제 페이지 완료 비율로 바꾼다.
원인은 기존 `PdfImportRequest.open`이 동기 함수이며 Worker가 initializing/importing/ready
단계만 전달하던 구조다. 생산·소비 연결은 다음과 같다.

- `parser/pdf/mod.rs`: 선택 쪽의 extract/lowering 직전과 메타데이터 수록 직후에
  원본 현재 쪽(1-based)·원본 전체 쪽·선택 쪽 완료 수·선택 전체 수를 알린다.
- `PdfImportSession.open_with_progress` → `PdfImportRequest.openWithProgress` → Worker의
  request ID가 붙은 progress message → 공통 Channel 검증 → Studio 대화창.
- 알림은 선택 쪽당 최대 2건이며 픽셀/연산마다 호출하지 않는다. 기본 open API는 같은 구현의
  no-op observer를 사용한다. callback 오류는 요청 실패로 처리한다.
- Channel은 유한 정수·범위·전체 수 일관성·역행 여부를 검사하며 취소/기한 만료/오래된
  request ID 알림으로 후보 문서를 게시하지 않는다. 기존 phase-only observer도 유지한다.
- 페이지 수 확인 전에는 정지된 0%와 준비 상태를 표시한다. 확인 후에는 현재 쪽/전체 쪽과
  `floor(완료 쪽 / 선택 전체 쪽 × 100)`을 표시한다. 일부 선택 시 원본 번호와 선택 완료 수를
  구분한다. 페이지 변환 100%와 이후 조판·편집 문서 직렬화/확인 단계를 구분하며 남은 시간의
  추정 비율로 설명하지 않는다. 취소·실패 후 재시도는 값을 초기화한다.

변경은 진행 관찰 경로이며 조판·paint·저장 규칙을 바꾸지 않는다. 출력 보존은 실제 37쪽
입력의 새 WASM 저장 바이트와 직전 성공 HWPX를 대조하며, 새 시각 golden을 등록하지 않는다.
Native 선택 쪽 순서·중간 callback 실패·다음 요청 복구와 실제 Studio의 중간 진행률·취소를 검증한다.
초기 연결 시 잘못된 함수 위치에 삽입한 코드를 WASM 컴파일에서 검출하여 수정했으며,
해당 실패 빌드는 적용·검증 산출물로 사용하지 않는다. 최종 빌드와 실행 결과를 아래에 이어 기록한다.


### 쪽별 진행 표시 실행 결과

- Native 정책/진행 계약 10건 통과. 선택 원본 쪽 1·3의 시작/완료 순서와 완료 수 0·1·1·2,
  callback 실패 시 최종 완료 알림 없음·세션 재사용 거부·새 요청 복구를 확인했다.
  observer 유무에 따른 HWPX 바이트도 동일했다.
- Studio 전체 Node 검사 2,704건 통과, 기존 skip 2건. 공통 Worker 29건 포함.
  TypeScript, Node 구문 검사, rustfmt, Native/WASM32 Clippy, suite manifest 및 E2E manifest 검사 통과.
- 새 최적화 WASM에서 실제 37쪽 시작·완료 74건, 일부 쪽 선택 번호, callback 오류·복구 확인.
  37쪽 전체 저장 바이트는 직전 성공 HWPX와 같았고 재열기도 37쪽이었다.
- 실제 Studio E2E: 37쪽과 기본 2쪽 대조군 모두 현재/전체 쪽·완료 비율·중간 취소 후
  기존 문서 보존 통과. 암호 입력·오입력·재시도·취소, 기존 대화창 테마·작은 화면·키보드·
  1초 timeout·재시도 E2E도 같은 새 WASM에서 통과했다.
- [실제 화면과 빌드·입력·저장 결과 근거](assets/issue8413/page-progress/README.md).
  화면에서 `17 / 37쪽 처리 중 · 43% (16쪽 완료)`를 직접 확인했다.

이 절편은 요청한 진행 표시의 로컬 구현·실사용 검증이다. 공통 PDF 기능의 미해결 항목이나
최종 PR 사전 검증을 완료했다고 표시하지 않는다. 원격 push·PR 생성은 하지 않았다.
