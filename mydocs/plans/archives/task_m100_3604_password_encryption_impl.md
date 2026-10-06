---
kind: implementation-plan
status: active
issue: 3604
last_verified: 2026-08-01
---

# #3604 후속 구현 계획: 공통 비밀번호 암호 모듈

## 모듈 경계

`src/password_crypto.rs`는 다음을 소유한다.

- HWP3 UTF-16LE 비밀번호 키 유도, DES-ECB, raw-DEFLATE/CRC32/ISIZE 검증과 암호화
- HWP5 `EncryptVersion=4` SHA-1 키 유도와 AES bit-CFB 스트림 암·복호화
- HWPX SHA-256 start key, PBKDF2-HMAC-SHA1, AES-256-CBC no-padding, raw-DEFLATE,
  ODF manifest `encryption-data` 조립과 검증

parser와 serializer는 파일 형식의 레이아웃, CFB/ZIP 읽기·쓰기를 담당하고 위 모듈에
평문·암호문 바이트와 비밀번호만 넘긴다. 포맷별 crypto 파일에는 알고리즘 구현을 두지
않는다.

## Stage 4 변경

1. 공통 오류형과 HWP5 공개 암호 스트림 함수를 도입한다.
2. HWP5 serializer가 FileHeader의 암호 플래그/버전과 DocInfo, BodyText, BinData, 추가
   스트림의 암호문을 생성하도록 별도 저장 entrypoint를 추가한다.
3. parser의 기존 HWP5 복호화 호출을 공통 모듈로 바꾼다.
4. 암호 HWP5 저장 후 일반 열기 거부, 올바른 비밀번호 재열기, 오류 비밀번호 거부,
   embedded BinData와 extra stream 보존을 검증한다.

## Stage 4 결과

- HWP5 key derivation과 AES bit-CFB는 `src/password_crypto.rs`로 이관했다.
- `src/parser/crypto.rs`에는 배포용 ViewText 특수 암호 처리와 공통 모듈 adapter만
  남겼다.
- HWP5 저장 entrypoint는 `serialize_hwp_with_password()`로 추가했다. 아직 public CLI와
  MCP에는 노출하지 않는다.

## Stage 5 결과

1. `src/password_crypto.rs`가 HWP3 UTF-16LE/DES-ECB/raw-DEFLATE/CRC32/ISIZE,
   HWP5 SHA-1/AES bit-CFB, HWPX ODF manifest/SHA-256/PBKDF2/AES-256-CBC를 모두
   소유하도록 이관했다.
2. HWP3 parser와 HWPX parser의 기존 crypto 파일은 기존 오류 유형으로 바꾸는 얇은
   adapter만 남겼다.
3. `encrypt_hwp3_password_document()`는 압축되지 않은 raw HWP3를 password
   payload로 만들고, `encrypt_hwpx_package()`는 한컴 ODF 보호 대상 엔트리를
   raw-DEFLATE 후 AES-256-CBC no-padding으로 암호화해 ZIP `Stored`로 기록한다.
4. `serialize_hwpx_with_password()`를 추가했다. salt와 IV는 `getrandom`의
   운영체제 난수로 생성하며 password·파생 키는 함수 밖에 보관하지 않는다.
5. HWP3 실제 password fixture 재암호화/재열기, HWPX 실제 평문 fixture 암호화,
   manifest/Stored 검사, 올바른·오류 비밀번호 재열기를 테스트로 고정했다.

## Stage 6 결과

1. `rhwp convert`와 `rhwp export-hwpx`에 전역 `--output-password`와
   `--output-password-stdin`을 추가했다. 출력 암호는 input `--password`와 분리된다.
2. 두 stdin option이 함께 있으면 input 암호를 첫 줄, output 암호를 둘째 줄로 읽는다.
3. HWP5 output은 HWPX-to-HWP adapter 적용 뒤 EncryptVersion 4로, HWPX output은
   ODF AES-256-CBC/PBKDF2로 저장한다.
4. `--verify`와 `--verify-pages`의 output reload는 output 암호를 사용한다.
5. 실제 HWP5/HWPX fixture에서 평문 거부, 정답 재열기, 암호 HWPX re-key를 확인했다.
6. WASM facade와 Studio HWP/HWPX 저장 대화상자의 암호 설정을 추가했다. Studio는 새 암호·확인을
   받고 HWP5/HWPX만 보호 저장하며 HML에는 암호 설정을 노출하지 않는다.
7. Studio는 암호 값 대신 보호 저장 여부 boolean만 메모리에 보관하고, 이후 Ctrl+S에서
   재입력받는다. public WASM JS/type declaration도 새 binding을 제공한다.

## Stage 8 결과

1. Finder/Explorer document drop에서 `DataTransferItem.getAsFileSystemHandle()` capture를
   제거했다. 암호 HWPX에서 관측된 Chromium renderer 종료를 피하기 위해 `File` bytes만
   `loadFile()`에 전달한다.
2. 드롭 문서는 파일 메뉴와 동일한 `loadDocumentForOpen()` 및 password dialog 경로를 사용한다.
   사용자 열기 확인 이전에는 bytes를 읽지 않는다.
3. 드롭 문서는 writable handle을 보관하지 않는다. 이후 Ctrl+S는 save-as 흐름을 사용하며,
   파일 메뉴로 연 문서의 기존 handle 저장 동작은 유지한다.
4. 관련 TypeScript 테스트 26개, typecheck, production build를 통과했다. 실제 native Finder
   drag/drop은 검증 호스트의 브라우저 실행 환경 부재로 사용자 수동 확인을 대기한다.

## 다음 단계

- hwp-convert MCP가 input `password`와 output `output_password`를 분리해 받고,
  Hancom 변환 결과를 CLI stdin 경로로 password-protect한다.
- Hancom Office 2020 직접 열기 교차 검증은 MCP server가 새 rhwp binary를 포함한 배포본으로
  갱신된 뒤 운영 환경에서 별도 evidence로 고정한다.

## 보안 규칙

- HWP5 암호화 알고리즘은 한글 파일 호환용 legacy 형식이다. 새 난수나 salt가 없는
  format 계약을 임의로 추가하지 않는다.
- HWPX는 salt와 IV에 운영체제 난수를 사용한다. deterministic fixture는 테스트 전용
  난수 주입으로만 만든다.
- password와 파생 키는 함수 범위 밖에 보존하지 않고, 오류 문자열에도 포함하지 않는다.
