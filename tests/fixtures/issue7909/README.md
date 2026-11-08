# #7909 공개 재현 입력

출처: [#7472 공개 ZIP](https://github.com/user-attachments/files/32746158/repro_footer_picture.zip).
사용자가 해당 공개 ZIP을 이 작업 채팅에 첨부했고, 압축 해제 이외에 변경하지 않았다.
실제 사용자 문서가 아닌 이슈에 공개된 더미 문서다.

- ZIP SHA-256: `1c4415a9b550fc865d79a832424510e3d656d21ccbafc0856b1961c02ab25918`
- `repro_footer_picture.hwp`: `fc8a3f77a60368e2cd2db578456d32e3b73fee8b90a920c0ba2304bd12bf2078`
- `control_footer_nopicture.hwp`: `6fd1e3de8fe899982d2ebaba37718bea91741bbf07828d7f7234dd07e8abb7c4`

검사는 자동번호 표시 문자열 export와 모델 오프셋 분리를 확인한다.
#7910의 그림·사각형 배치나 한컴 Print PDF와의 시각 일치를 검증하는 fixture/golden은 아니다.

## 자동번호가 없는 첨부 대조본과 보존한 최소 변형

공개 `control_footer_nopicture.hwp`는 꼬리말의 도형·그림뿐 아니라 `atno`도 없다.
파싱 결과 해당 문단의 controls는 0개이며, `dump-records`에도 footer의
`CTRL_HEADER atno`(`6f 6e 74 61`)가 없다. 원본에는 도형·그림·자동번호가 하나씩 있다.
이 첨부 대조본에서 번호 `1`을 기대하지 않고 일반 문구와 새 표시 필드 부재를 검사한다.

`footer-no-picture-keep-auto.hwpx` / `.hwp`는 위 원본에서 도형·그림만 제거한 **rhwp 생성 대조**다.
정확한 생성 코드는 `tests/cases/issue_7909_text_layout_display.rs`의
`no_picture_with_auto_number`다. 선행 두 개체의 확장 제어 슬롯을 각각 8 UTF-16 유닛씩
제거하고, 관련 char_offsets/char_shapes/char_count를 맞춘다. atno와 일반 문자열은 보존한다.
저장 LineSeg를 무효화한 뒤 HWPX로 직렬화하며, 그 HWPX를 다시 열어 HWP로 내보낸다.
한컴 생성본이나 기존 첨부의 바이트 그대로인 파일로 취급하지 않는다.

- 생성 HWPX SHA-256: `71e3ef523a26b1b5397b573c30a37e05113b001be951712a255f5865a203425b`
- 생성 HWP SHA-256: `e79af8d03f5e39dbbd246072eab3203b39ecdb01aed9aa4221ca217214016a9e`

fresh WASM API 검사는 저장소 루트에서 다음으로 재현한다.

```bash
node tests/fixtures/issue7909/verify-wasm.mjs pkg
```
