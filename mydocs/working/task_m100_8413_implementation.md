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
