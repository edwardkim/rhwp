# #8413–#8416 공통 구현·검증 기록

상태: 곡선 clip·회전 텍스트·마스크 수정과 원본 4개 127쪽의 Native/fresh WASM·Studio 검증 완료. 최소 글꼴 환경 전체 회귀·Native 3종·집중 검사·lint까지 통과. PR·push·self-review 승인 후 준비 중이며 머지는 수행하지 않았다.
기준과 범위: [수행 계획](../plans/task_m100_8413.md),
[원인 조사](../tech/investigations/issue-8413/README.md).
공통 계획 승인 기록은 `f1c08f7112`다. 메인테이너의 추가 요구인 PDF 열기 암호 입력을
#8415에서 먼저 연결했으며, 뒤에서 공통 clip·이미지·명령 수와 쪽별 진행 표시의 구현·검증을 이어 기록한다.
최신 결과는 [곡선·회전·마스크의 최종 제품 검증](#곡선회전마스크의-최종-제품-검증--2026-11-01)에 있다.

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

## 곡선 clip·회전 텍스트·마스크 보정 — 2026-11-01

사용자가 쪽별 진행 표시를 확인하고 남은 세 오류의 해결을 지시했다. 기존 공통 계획의
구현·검증 범위로 진행하며 원격 게시 승인은 포함하지 않는다.

독립 근거는 [ISO 32000-1](https://developer.adobe.com/document-services/docs/assets/35e4369068f86065372c18787171a17e/PDF_ISO_32000-1.pdf)
8.5.2–8.5.4의 경로/채움/clip, 9.4.4의 텍스트 변환과 7.4·8.9의 필터/마스크 계약이다.
원본 PDF의 객체·행렬을 pikepdf로 읽고 backend의 실제 span basis와 대조했다.

- `img_clipping_01.pdf`의 둥근 사각형·원형 clip은 정상적인 cubic 경로다.
  `ImageClipTracker.apply`가 이를 Unsupported로 바꾸면서 `visible_bounds`에서 실패했다.
  tracker의 정확한 경로 → page-space clip 교집합 → `PdfImage.apply_affine_clip`의
  alpha mask로 연결한다. bounds는 래스터 할당 영역에만 쓰며 투명도는 원래 곡선과
  nonzero/even-odd 채움으로 계산한다. q/Q와 Form CTM, 경로 종료 시 W 적용을 유지한다.
- 암호화 원본 10쪽의 90도 텍스트 basis는 `[0, 8.415, -8.5, 0]`이다. 반사나 전단이
  아니라 두 축 배율이 다른 정상 행렬이다. import 전용 span 경로에서 축의 길이를
  기준으로 높이·advance·장평을 보존하고, 공통 editable text-box의 회전과 글자 장평으로
  연결한다. 읽기 순서 추출 API 전체나 일반 조판의 배치 규칙은 수정하지 않는다.
- `f87pro.pdf`의 SMask 2122·2123은 Flate이며 DecodeParms에 Predictor가 없어 기본 1이다.
  기존 코드는 매개변수가 있다는 이유만으로 거부했다. bounded filter decoder가 매개변수를
  해석하게 하고 packed row 길이를 검증한다. SMask 2123의 5309×3379 픽셀은 16M을 넘으나
  승인한 64M 상한 안이다. 마스크도 유한 픽셀·stream·누적 메모리 한도를 적용한다.
  암호화 원본 SMask 80은 회색조 DCT이므로 요청 예산을 받는 JPEG decoder로 실제 샘플을
  읽고 선언 크기·8-bit·성분 수를 확인한다.

구현과 실제 실행 검증 중이다. 컴파일·원본 열기 성공을 시각 검증 완료로 간주하지 않으며,
전체 Native/fresh WASM 출력과 저장 재열기는 결과를 확보한 뒤 이어 기록한다.

### 첫 Native 실행에서 확인한 후속 경계

첫 빌드는 성공했고 8쪽 합성 원본 PDF의 Native 전쪽 raw 최저 98.97%를 확인했다.
원·evenodd 구멍·반대 winding 구멍·v/y 경로와 q/Q 복원·Flate 기본/PNG predictor·회색조 JPEG
마스크·비등방 90도 텍스트를 구분한 대조군이다. 90도 글자와 원형/복합 경계를 직접 판독했다.
이 결과는 아직 fresh WASM 증거나 정식 회귀 등록의 완료를 뜻하지 않는다.

실제 원본의 다음 실패도 함께 처리한다.

- 32쪽: 11쪽 image_decode에서 2 GiB 누적 예산 초과. 별도 trusted-host 진단 예산으로
  원본 전체를 끝까지 실행했을 때 32쪽 변환은 성공했고, `decoded_bytes=6,302,049,999`,
  `codec_allocated_bytes=3,609,279,018`, `codec_work_units=1,338,764,346`이었다.
  문서 전체 선언 원본 픽셀은 224,528,384개다. 작은 예산을 무시하거나 카운터를 초기화하지
  않고 표준 누적 decoded 8 GiB·작업량 20억으로 조정한다. 누적 decoded/codec allocation
  카운터는 `u64`로 계상해 WASM32에서 4 GiB 이후 wrap/포화되지 않게 한다.
  JS 전달은 안전한 정수 Number로 유지하며 재열기에 넘기는 남은 예산도 같은 값이다.
  단일 stream/codec 할당 256 MiB·원본/합성 이미지 64M·문서 2B 픽셀·호스트 시간/메모리
  제한은 유지한다. 절약 프로필의 기존 한도도 유지한다.
- 암호화 57쪽: 기존 회전 텍스트 오류 다음, 21쪽에서 일반 이미지의 affine CTM을 positional
  lowering이 거부했다. 패턴/비사각 clip에 사용하던 공통 bounded affine compositor를
  일반 회전·반사·전단 이미지에도 연결한다. 이미지 CTM → 공통 샘플링 → 축 정렬 저장
  프레임으로 연결하며 문서별 보정을 넣지 않는다.
- f87pro: DecodeParms는 통과했으나 mask 합성의 별도 16M 검사가 남아 있었다.
  읽기/합성 양쪽을 승인한 64M 기준으로 맞추며 RGBA 할당은 같은 요청 예산으로 예약한다.
- 37쪽 정상 대조군은 재열기까지 성공했다. 저장 바이트 크기는 기존과 같은 46,292,946이다.

기존 거절 회귀의 선행 근거도 확인했다. `issue_8174_pdf_consecutive_moves.rs`의
compound/curve 거절은 #8174 → 통합 #8360(`f125c735c7`, 2026-10-29 15:34:34 +09:00)의
이전 지원 범위이며, [stage8](task_m100_8174_v2_stage8.md)의 5건 검증 기록과 연결된다.
기존 원본 두 PDF도 새 Native/fresh WASM에서 직접 비교한 뒤, 거절 기대값만 새 지원 계약에
맞게 바꾸고 정상 사각 crop·alpha·요청 예산 검사는 유지한다.


### 전쪽 시각 대조에서 드러난 signed rectangle 경계

57쪽 원본의 첫 Native 전쪽 비교에서 7쪽이 raw 90% 미만(최저 79.65%)이었다.
43쪽 직접 판독에서 표처럼 보이는 배경 일부가 사라졌으나, 진단의 semanticTables=0이므로
표 승격이나 셀 분할의 결함으로 판정하지 않았다. 원본의 여러 `re`는 높이가 음수이며
compound fill로 함께 그린다. `PathExtractor::compute_bbox`가 시작점을 최솟값, 반대점을
최댓값으로만 처리하여 래스터 프레임을 잘랐다. 배경 뒤의 흰 글자는 그 때문에 보이지 않았다.

생산 `PathExtractor::compute_bbox` → `PathContent.bbox` → `path_paint::lower`의
래스터 프레임·transform → PNG 그림의 공통 위치 순서다. 원본 re 방향은 그대로 유지하고
경계만 두 끝점의 min/max로 계산한다. 문서·페이지·셀별 분기는 추가하지 않는다.
원 로직은 vendoring 커밋 `4662c069e55`에서 왔으며 #8120(`80968497c2`)의 path paint,
#8174/#8360(`f125c735c7`)의 bounded 처리, #8403/#8410(`1cfa5726cd`)의 text paint 이후에도
유지되던 계산이다. 독립 근거는 ISO 32000-1 8.5.2의 signed re와 원본 PDF 경로다.
양/음수 너비·높이의 compound fill, Form 안의 곡선 clip, 일반 affine image 대조군을
임시 PDF에 추가했다. 보정 후 Native/fresh WASM 재측정 전이며 완료 수치를 추정하지 않는다.


### Type3 검색 문자열의 실제 paint 보존

32쪽 원본의 첫 Native 비교는 1·2·12쪽이 90% 미만이었다(전쪽 최저 82.72%). 2쪽을 직접
대조하니 원본의 도형 글자 위로 대체 글꼴이 겹쳤다. `/F1`은 Type3이며 실제 CharProcs 전부가
폭·bbox를 선언하는 `d1` 한 개뿐이다. 본문 윤곽은 별도 path paint이고 검색 문자열 자체는
아무것도 그리지 않는다. 이를 이름이 Unknown이라는 이유로 함초롬바탕의 가시 글자로 바꾼
것이 원인이다. 글꼴 설치로 해결할 차이가 아니며 전체 Type3를 숨겨서도 안 된다.

실제 scoped FontInfo 생성에서 CharProcs를 요청 예산 안에서 해석한다. 모든 glyph가
유효한 d0/d1 하나만 가지는 경우에만 non-painting 근거를 표시한다. 누락·빈 사전·그리기
명령·혼합 glyph는 이 판정을 받지 않는다. lowering은 문자열과 advance를 그대로 두고
기존 TextPaint의 fill=false·alpha=0 계약을 소비한다. 별도 renderer 분기나 문서별 조건은
없으며 저장 후 재열기도 기존 text-paint 확장을 사용한다. 원본 paint와 검색 문자열을
둘 다 보존하는 계약이다. 독립 근거는 ISO 32000-1 9.6.5의 Type3 glyph procedure다.
기존 #8174/#8360의 Type3 문단/줄 계약과 #8403/#8410의 명시적 TextPaint를 재사용한다.

### 곡선·마스크 후속: PDF 이미지의 색 전달

32쪽 원본의 12쪽에서 PNG 원본과 저장 BinData의 RGB는 239/239/239인데,
2024 Print 렌더에서는 247/247/247이 되었다. 같은 HWPX를 2022 호환으로
진단 출력하면 239/239/239로 복원된다. JPEG·ICC·SMask 이후의 저장 픽셀은
정상이므로 이 차이를 디코더 오류로 판정하지 않는다.

원인은 `paint/builder.rs`가 PDF에서 변환한 PNG에도 한컴 Print bitmap 플래그를
설정하여 `image_resolver/print_tone.rs`의 RGB555 전달을 다시 적용하는 경로다.
원 구현은 #7642의 [팔레트 경계 독립 Print](task_m100_7642_stage2101.md),
#8160의 [그림 전달 검증](task_m100_8160_stage2.md), 통합 #8389
`c417310cab`에서 확인했다. 이 규칙은 일반 HWP 그림의 한컴 출력 계약으로 유지한다.
PDF의 색 공간과 마스크를 이미 처리한 그림은 별도 출처 계약으로 식별하고,
공통 paint 생성에서 해당 Print 변환을 적용하지 않아야 한다. BinData의 기존 private
PNG 선언을 확장해 저장·재열기와 Native/WASM 모두 같은 판단을 소비하게 한다.
명시적 이미지 보간과 변환 후 기본 보간도 구별해 유지한다.

진단 증적은 `p12-direct/`, `p12-no-hancom-transfer/`, `p12-tree/`다.
이 진단은 수정 완료 판정이 아니며 32쪽 전체와 정상 HWP 대조군을 다시 검증한다.

### 곡선·회전·마스크의 최종 제품 검증 — 2026-11-01

최종 제품 소스에서 Native CLI와 최적화 WASM으로 네 원본을 모두 가져오고,
HWPX·HWP 저장 후 재열기를 확인했다. 암호화 원본도 복호화한 PDF를 우회 입력으로 쓰지 않고
원본 바이트를 인증해 처리했다. 실제 Studio의 가져오기 대화창 → Worker → 편집 문서 열기에서도
32·37·57·1쪽을 확인했다. 테스트에서는 큰 입력을 위해 대화창 제한 시간을 300초로 명시했다.
기본 30초가 모든 호스트와 입력에서 충분하다는 의미는 아니다.

| 원본 | 전체 쪽수 | Native 전쪽 최저 raw | fresh WASM 전쪽 최저 raw |
| --- | ---: | ---: | ---: |
| img_clipping_01.pdf | 32 | 99.89% | 99.89% |
| img_bug_01.pdf | 37 | 99.76% | 99.83% |
| PDF_ENCRYPTION_UNSUPPORTED.pdf | 57 | 91.72% | 91.64% |
| f87pro.pdf | 1 | 99.82% | 99.82% |
| 공개 합성 대조군 | 13 | 98.88% | 98.97% |
| 일반 HWP 팔레트 대조군 | 1 | 100.00% | 100.00% |

127쪽 원본의 누락·쪽수 불일치·90% 미만은 없다. 원본 PDF의 독립 PDFium 96dpi 출력과
색상 경계 보정 전 2px 실루엣으로 측정했다. 일반 HWP 대조군은 기존 독립 한컴 2024 Print와
비교했고 Native 출력 PNG의 수정 전후 SHA-256도 같았다
(`f1db64b6cad8c4101789e738df0debc0d5bec7f8dcf99e008d4435fad89ee7c9`).
[전쪽 TSV·공개 합성 PNG·입력/소스/산출물 신원](../pr/assets/issue8413/curve-mask-validation-20261101/README.md)을 보존한다.
사용자 원본·원본 화면·추출한 글꼴은 공개 fixture나 배포 자원에 포함하지 않았다.

직접 판독은 원형·복합 구멍·Form 안의 곡선·q/Q 복원·affine 이미지, 회전 글자의 방향과 폭,
32쪽 원본의 1·2쪽 Type3 중복 제거 및 12쪽 회색/투명도, 암호화 원본 10쪽의 회전 글자와
21쪽 이미지·43쪽 signed rectangle 배경, f87pro의 연속 alpha를 대조했다.
합성 Helvetica는 정확한 Arial 프로그램을 명시했다. 처음의 넓은 Windows 글꼴 목록이
Arial Narrow를 `Arial`로 먼저 선택한 진단은 최종 증거에서 제외하고 표준 Arial 네 face만
지정해 재측정했다. 37쪽의 같은 글꼴 선택도 수정했다. 이 보정은 제품 글꼴 선택 코드 변경이
아니라 검증 환경 지정이다. 37·57쪽은 원본 프로그램을 로컬에서 복원한 글꼴 환경을 기록했다.

raw 통과를 완전한 픽셀 동일성으로 해석하지 않는다. 암호화 문서 일부 본문의 글꼴 굵기·영문
간격에는 기존 표현 차이가 남는다. f87pro 키보드의 0.135pt 세선은 PDFium 96dpi 직접 출력과
216dpi 객체별 래스터를 축소하는 현재 출력에서 농도가 다르다. 원본을 독립 PDFium 216dpi로
확대해 같은 선들의 존재와 형상을 추가 확인했다. 마스크를 버리거나 굵기를 샘플별로 맞추지
않았으며, 이 차이를 모든 PDF paint·원 저작 문단 복원이 완료됐다는 주장으로 덮지 않는다.

| 원본 | Native 가져오기 wall / peak RSS | WASM 가져오기·양쪽 형식 저장/재열기 | WASM linear memory capacity | 실제 Studio 열기 |
| --- | --- | ---: | ---: | ---: |
| img_clipping_01.pdf | 49.27초 / 688.45 MiB | 67.92초 | 769 MiB | 83.11초 |
| img_bug_01.pdf | 24.40초 / 399.60 MiB | 24.52초 | 529.25 MiB | 26.46초 |
| PDF_ENCRYPTION_UNSUPPORTED.pdf | 9.54초 / 309.42 MiB | 23.41초 | 581.94 MiB | 17.68초 |
| f87pro.pdf | 10.19초 / 471.80 MiB | 53.56초 | 711.13 MiB | 139.95초 |

측정은 각 1회이며 다른 빌드와 일부 겹쳤다. Native CLI wall은 가져오기·HWPX 저장,
WASM 시간은 HWPX 저장과 편집기 재열기·HWP 저장/재열기를 포함하므로 서로 속도 비교값이 아니다.
WASM linear memory capacity는 현재 실제 사용량이나 프로세스 peak RSS가 아니다. 누적 decoded
8 GiB는 재사용·해제한 버퍼의 작업량 계상이며 8 GiB를 동시에 할당한다는 뜻이 아니다.
Native peak RSS는 `/usr/bin/time`으로 별도 측정했으며 부모와 자식 프로세스의 동시 RSS 합계가 아니다.

f87pro의 첫 Studio 자동화는 화면이 표시된 뒤 CDP 완료 대기에서 실패했다. 제품의 Worker
시간 한도를 우회한 것이 아니라 검증 도구의 통신 대기를 600초로 늘려 다시 실행했고,
`open-document-bytes:done.ok=true`, 1쪽 및 페이지 변환 100%를 확인했다. 실제 열기 139.95초는
많은 도형을 준비하는 초기 표시 지연이 남음을 보여 준다. 오류 해소를 초기 로딩 성능 개선으로
보고하지 않는다. 첫 실패와 재실행 로그·화면은 로컬 증거에 함께 남겼다.

기존 마스크 실패 검사는 #7974의 색 키 지원 → #8174/#8360(`f125c735c7`)의 bounded 거부
계약을 추적했다. 정상 DCT SMask가 새로 지원되므로, DCT라고 표시한 단 한 바이트 `0x80`는
`Unsupported`가 아니라 malformed image다. 해당 두 검사만 `Image` / `PDF_PARSE_FAILED`
진단으로 교정하며, 손상된 두 번째 쪽 때문에 전체 요청이 실패하는 원자성과 정상 첫 쪽만
선택했을 때 성공하는 계약을 유지한다. raw 표본에 DCT/JPX/JBIG2 등의 이름만 붙인 입력이
그대로 이미지로 채택되지 않는 검사도 유지한다. 정상 JPEG의 alpha는 별도 실제 대조군으로
확인했다.

새 관계 검사는 정식 suite 자동 수집 경로에서 곡선 내부/외부·구멍·q/Q·Form·affine alpha,
동일한 Flate/predictor/JPEG mask 표본, 회전 char ratio와 양쪽 저장, Type3의 검색 문자열 보존,
PDF 이미지 색 출처 및 signed rectangle의 양 끝점 포함을 검사한다. Type3 전체 glyph가
metrics-only라는 증거가 있어야 하며, 하나라도 paint/unknown/빈 procedure이면 숨김 판정을
하지 않는 반례도 포함했다. 개인 설치 글꼴 없이 실행한다.

### 자동 검사 환경과 최종 게이트

제품 구현은 `f17ca45e14`에 고정했다. 전체 회귀는 해당 커밋의 전용 검증 worktree에서
suite `--prepare` 후 base `bc9dfdd057258de59afebeb5121c4bee7398e07f`를 명시한
manifest 검사로 수집을 확인했다. 파생 suite는 제품 커밋에 포함하지 않는다.

첫 전체 실행은 개인 글꼴 경로가 노출된 환경임을 최종 조건 점검에서 발견해 중단했다.
그 결과를 정식 회귀 통과로 사용하지 않고, 동일 제품 코드·공용 target으로 `bwrap` 파일 시스템
격리 후 다시 실행해 통과했다. 시스템 글꼴 두 디렉터리를 빈 mount로 덮고 사용자 글꼴·외부 TTFS·Windows
드라이브·진단용 추출 글꼴 경로를 숨겼다. `RHWP_FONT_PATH`도 제거한다. Fontconfig뿐 아니라
파일 경로 존재 여부를 검사했고, 실제 목록은
[최소 글꼴 환경](../pr/assets/issue8413/curve-mask-validation-20261101/minimal-font-environment.json)에 보존한다.
전체·Native 집중 검사는 같은 격리를 사용한다. 독립 PDF 비교에 글꼴을 명시한 시각 검증과
개인 설치 글꼴 없는 회귀 검증을 구분한다.

SVG Print 경로도 공개 합성 13쪽을 실제 출력했다. 곡선·마스크 페이지의 내장 PNG에 투명·불투명·
중간 alpha가 남고, 색상 대조군의 회색 RGB 239가 보존됨을 확인했다. SVG는 Native Canvas·fresh
WASM 시각 검증을 대체하지 않는 추가 소비 경로 검사다.

최종 제품 커밋 `f17ca45e14`의 필수 로컬 검증을 모두 완료했다.
[명령·종료 코드·검사 수·로그 해시](../pr/assets/issue8413/curve-mask-validation-20261101/validation.json)를 보존한다.

| 검사 | 최종 결과 |
| --- | --- |
| 최소 글꼴 환경 release-test 전체 | 12,249 PASS / 0 FAIL / 45 skipped |
| Native Skia lib | 4,041 PASS / 0 FAIL / 12 ignored |
| Native 그림 누락 대조군 | 2 PASS |
| Native 직접 PDF 출력 | 22 PASS |
| Native PDF 자원 제한 | 134 PASS |
| Native 신규 곡선·회전·마스크 관계 검사 | 5 PASS |
| fmt, Native/WASM Clippy, workspace build/all-target Clippy | 모두 PASS |
| backend 선택적 rendering feature 빌드 | PASS |
| Studio 전체 단위·TypeScript | 2,705 PASS / 0 FAIL / 2 기존 skipped, TypeScript PASS |
| 공통 Worker 집중 검사 | 30 PASS |
| fresh Studio E2E | 암호·페이지 진행률·가져오기 대화창·busy 상태 4개 모듈 PASS |
| 실제 WASM 자원 잔량 경계 | 유효한 8 GiB 잔량 재열기 성공, 무효 값 7개 거부 |

Native lib 수는 rhwp 3,859건과 같은 명령의 라이브러리 15·165·2건을 합산한 값이다.
집중 nextest의 skipped는 선택하지 않은 같은 suite의 검사이며 전체 실행 결과와 구분한다.
nextest 0.9.137은 요구 최소 버전을 충족하지만 권고 0.9.140보다 낮아 버전 안내와 CI 전용
`report-skipped` 설정 무시 경고가 출력됐다. 기본 profile의 검사 실행·실패 판정에는 영향이 없었고
검사를 건너뛰는 옵션이나 허용치 변경은 사용하지 않았다.

제품 코드·테스트와 최적화 WASM은 검증 후 변경하지 않았다. 후속 커밋은 최종 결과·글꼴 격리
증적과 중복된 이전 PNG 정리만 담는다. 이번 요청의 곡선 clipping·회전 텍스트·마스크 오류는
수정·실제 재검증까지 완료했으며, 위에 적은 기존 글꼴/세선 표현 차이와 f87pro 초기 표시 지연을
완전한 피델리티·성능 개선으로 보고하지 않는다. PR·push·merge는 수행하지 않았다.


### PR 준비와 최신 devel 정합 — 2026-11-01

메인테이너가 남은 큰 문제의 백로그 기록과 PR Open·self-review를 승인했다.
[후속 백로그](../plans/task_m100_8413_backlog.md)에 초기 표시 지연, 본문 글꼴 굵기·간격,
가는 벡터 선 농도 차이를 재현·진입점·완료 조건과 함께 기록했다. 이번 제출의 지원 범위를
포괄적인 PDF 피델리티·성능 완료로 확대하지 않는다.

원본 PDF 4개도 공개 커밋을 명시적으로 승인받아 [재현 자료](../../samples/issue8413/README.md)에
원래 바이트로 추가했다. 기존 로컬 검증 입력과 SHA-256이 같으며 모든 파일이 50 MiB 미만이다.
앞선 기록의 원본 비공개 보존은 이 승인 전 상태다. 검증용 글꼴 추출물은 추가하지 않는다.
공개 합성 PDF에서 생성해 실제 검증한 HWP/HWPX도 동일 바이트로 포함했다.

새 devel `da39c536613a4bd543dc72b63a556183efc0383f`는 #8417의 CFB 시험 도우미 두 줄과
검토·오늘할일 문서만 추가한다. 원 문제 #8225 → #8233 → 통합 #8395의 선행 근거와
[독립 #8417 검토](../pr/archives/pr_8417_review.md)를 확인했다. 충돌 없이 리베이스했으며
이전 최종 head `c17f15f73e` → `25dc243a46`, 구현 `f17ca45e14` → `5ef817b434`다.
제품 소스·Cargo graph·WASM과 시각 입력의 해시가 같아 완료한 전쪽·전체 검사를 재사용한다.
새 base의 helper는 같은 CFB 입력을 여는 검증 코드이며 PDF paint 소비 경로를 바꾸지 않는다.
추가된 helper의 실제 그룹 payload 보존·중복 방지 검사와 해당 suite Clippy는 현재 후보에서
별도로 재실행해 3건과 해당 suite Clippy가 통과했다. 원 base의 결과는 #8417 독립 검토의 같은 소스·입력 실행과 대조한다.


제출 전 unit-tier 검사에서 초기 자원 정책 절편의 `lowering.rs::resource_boundary_tests`가
신규 source-side test 금지에 걸렸다. 허용 기준을 올리지 않고 기존 통합 자원 검사로 이관했다.
종전 검사는 빈 pixel payload의 내부 `PdfImage`를 helper에 직접 전달했다. 대체 검사는
실제 유효한 Gray/Flate PDF의 한 축이 HWP signed crop 좌표 범위를 넘도록 구성해 공개
`parse_pdf_with_policy`에서 `PDF_LOWERING_FAILED`와 정확한 경계 진단을 확인하고,
실패 후 정상 문서가 열리는지도 검사한다. 정상 출력·PNG 인코더·제품 한도는 변경하지 않는다.

이관 후 자원 정책 검사 12건이 통과했다. 첫 실행은 자동 생성 suite가 이관 전 배정이라
검사 0건으로 실패했으며 통과 근거에서 제외했다. suite를 다시 생성한 뒤 실제 12건 실행을
확인했다. 제품 코드의 `cfg(test)` 밖 바이트는 기존 시각 검증 구현과 같음을 직접 대조했다.

제출 전 코드 점검은 암호 인증·진단 redaction·재열기 설정의 암호 미보존, 요청별 자원 예산과
명시적 낮은 host cap, Worker의 실제 쪽 시작/완료 전달, clip의 q/Q·Form·좌표 변환과
mask 표본 검증, 회전 텍스트의 공통 IR 전달, PDF 이미지 색 출처를 대상으로 했다.
수치 맞춤·샘플 ID 분기·CI 허용치 변경은 추가하지 않았다. GitHub CI가 성공한 뒤 확정하는
정식 self-review와 구분하며, 별도 에이전트에 의한 독립 리뷰로 표기하지 않는다.

제출 후보의 추가 검증도 완료했다. 자원 정책 12건, #8417 CFB 보존 3건, source-side
unit-tier, fmt·Native/WASM Clippy·workspace build/all-target Clippy가 통과했다.
[제출 검증 기록](../pr/assets/issue8413/curve-mask-validation-20261101/submission-validation.json)에
명령·종료 코드·로그 해시와 이전 측정의 재사용 범위를 기록했다. 추가 HWP/HWPX 계약 fixture는
`tests/fixtures/issue8413/`에 두었고 두 파일의 hidden-text·injection·unicode 읽기 전용 검사
6건이 모두 clean이었다. 보안 검사 6건을 전체 보안 검증으로 확대 해석하지 않는다.
변경 문서 6개의 상대 링크와 `git diff --check`, 모든 공개 재현 자료의 크기·해시도 확인했다.
