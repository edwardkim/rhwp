# #8415 암호 인증 제어군

저장소 작업에서 직접 생성한 공개 합성 PDF다. 사용자 PDF·사용자 암호·외부 글꼴 프로그램을
포함하지 않는다. 동일한 영문 한 줄과 빨간 그림 하나를 가진 300×200pt 문서이며,
인증과 stream·ObjStm 복호화·내용 보존을 대조한다. 실제 사용자 문서의 시각 검증을 대신하지 않는다.

| 파일 | 열기 암호 | 차이 |
| --- | --- | --- |
| password-control.pdf | 없음 | 평문 대조군 |
| password-r2.pdf ~ password-r6.pdf | `synthetic-open` | Standard Security R2–R6, R4 이상은 ObjStm 포함 |
| password-empty.pdf | 빈 암호 | 복사·편집·인쇄 제한 플래그; 메인터너의 가져오기 허용 정책 검증 |
| password-unicode.pdf | `합성-암호-8415` | UTF-8 R6 인증 |
| password-long-unicode.pdf | `가` 43개 | 127바이트 경계를 가로지르는 UTF-8 암호 |
| password-metadata-clear.pdf | `synthetic-open` | EncryptMetadata=false |
| password-padded-fields.pdf | `synthetic-open` | V5 키 필드 뒤에 추가 바이트가 있는 incremental 암호화 사전 |

소유자 암호는 모두 테스트 전용 `synthetic-owner`다. 재생성은 `generate.py`를 별도 출력 경로로
실행한다. 암호화 난수 때문에 바이트가 달라질 수 있으므로 현재 입력은 `manifest.json`의
SHA-256으로 식별한다. R5 파일 키는 SHA-256 중간 키로 UE/OE를 AES-256-CBC(영 IV, 패딩 없음)
복호화해 얻는다. 독립 참조 구현은
[QPDF Encryption::recover_encryption_key_with_password](https://github.com/qpdf/qpdf/blob/main/libqpdf/QPDF_encryption.cc)다.
실행 결과는 [공통 작업 기록](../../../mydocs/working/task_m100_8413_implementation.md)에 기록한다.
