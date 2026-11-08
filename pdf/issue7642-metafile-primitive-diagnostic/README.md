# WMF/EMF 그리기 연산의 독립 Print 진단

Issue: #7642

이 자료는 기존 합성 회귀의 원본 그리기 명령을 한컴에서 확인하기 위한 **실패 진단 자료**다.
정상 출력의 회귀 fixture나 golden으로 승인하지 않았다.

## 입력과 Print 출처

[입력 HWPX](metafile-primitive-input.hwpx)는 기존 WMF/EMF primitive와 흑백 DPa 마스크
14건을 한 페이지에 한 그림씩 배치했다. WMF는 FileSize·MaxRecord·Placeable checksum만
정상 프레이밍으로 보정했다. 원래 잘못된 헤더의 출력과 같다고 주장하지 않는다.
EMF 원문 bytes는 바꾸지 않았다. 입력의 생성 조건과 그림 상태의 적절성은 검토 중이다.

| 파일 | SHA-256 |
|---|---|
| [입력 HWPX](metafile-primitive-input.hwpx) | `f77c89f7e535ff197cf9b2a0dd0e48e4542970dd9b46f8c6018e2a5eec9dd069` |
| [한컴 2024 저장 HWP](metafile-primitive-2024.hwp) | `4694ee199d7fc477064b570737c3be221582e8716b42e9cef391238a865c21cc` |
| [한컴 2024 Print PDF](metafile-primitive-2024.pdf) | `b417010ede3bcee974aa9db23cc75d3ca28202c14e26ecfdd9ceb1490b0d1484` |

- 한컴 2024 실제 버전: `13.0.0.3901`
- Save job: `2cd7844e-4843-461e-9edb-edb8082c7af7`
- Print job: `a16033db-4dc7-4f12-9fa5-c5426ce995e2`
- Print method: `0`, `frame_print_to_pdf_ex_one_up`
- 생성·실행·해시 기록: ignored `stage175-metafile-input-provenance.json`,
  `stage175-metafile-primitive-auto-print.json`

## 쪽별 원문 그림

| 쪽 | 원문 그림 | 헤더 보정 | 입력 SHA-256 |
|---:|---|---|---|
| 1 | [mask_equal.wmf](mask_equal.wmf) | 있음 | `f8d1c00cb97cda11fa5e34f4cfdb52b466c259a768289d252c660a30b2b6caf1` |
| 2 | [mask_reversed.wmf](mask_reversed.wmf) | 있음 | `18af365aab208f1a1b25c2f806507b25d4799d0e2b530c10013014b38500591e` |
| 3 | [mask_overscan_top_left.wmf](mask_overscan_top_left.wmf) | 있음 | `7773f2313f0cf747d6652fd6eb7cf71e01668124dd745c5c4ecae7ef529bef07` |
| 4 | [mask_shifted_bottom.wmf](mask_shifted_bottom.wmf) | 있음 | `42b1ce2aacd06d4a4339702059d13dfffb09c108350db38c4b901e1058b97252` |
| 5 | [mask_contained.wmf](mask_contained.wmf) | 있음 | `3e2ca12cc6b42db5b0b3fc6cf61b33d0cc9bf4d32d9c0b20c1640b3366e32fc4` |
| 6 | [wmf_minimal_placeable.wmf](wmf_minimal_placeable.wmf) | 있음 | `94c672be4a1bf1de5b99a91eda7f3bb5843d263ca90cfa5c2bb807f886789f72` |
| 7 | [wmf_placeable_rect.wmf](wmf_placeable_rect.wmf) | 있음 | `13851452ee6d44b8bbca7767666df63f73ac9b5b83eb4ca8f4386950b3243b20` |
| 8 | [wmf_placeable_ellipse.wmf](wmf_placeable_ellipse.wmf) | 있음 | `15fdff336db82f0a4602ad0d672edd8c509dcf79dc7bf033571549a9c2a0a44e` |
| 9 | [wmf_placeable_line.wmf](wmf_placeable_line.wmf) | 있음 | `be68ebdc6e969366334eb18281314310141bc9da96b6b98c4ff81265b2c57695` |
| 10 | [wmf_standard_header_eof.wmf](wmf_standard_header_eof.wmf) | 있음 | `7f5467a08b4fbdf80a0b29448d0e5550fdc8bccc08f982c42bac707a0b3059ff` |
| 11 | [emf_header_eof.emf](emf_header_eof.emf) | 없음 | `eda0712188dc1d36c295f923e9d69d49ecf22cbc8e35f35db1680c99ab6903a4` |
| 12 | [emf_rectangle.emf](emf_rectangle.emf) | 없음 | `2179ad5267849f25f03d9733fbc6c7510d19143f013bd398a7f4d67463029dea` |
| 13 | [emf_ellipse.emf](emf_ellipse.emf) | 없음 | `1df837e37460ca0dbd29f8cb3e3241d06ef524ff6717baae87d40f5390346190` |
| 14 | [emf_line.emf](emf_line.emf) | 없음 | `90547c96987036fe3183346bc4bd839a458f0e469f87412ee7d41a45ca19450f` |

## 현재 판정

생산 SHA `8e4f09c48`의 Native 전체 14쪽은 최저 **13.16977%**이며,
1·2·3·4·5·7·9·12·13·14쪽이 90% 미만이다. 마스크 계조 차이와
Native에는 있으나 Print에는 보이지 않는 EMF 도형을 직접 확인했다.
fresh WASM은 이 진단에서 아직 실행하지 않았다.

한컴 저장 HWP의 BinData 14개를 압축 해제해 입력과 대조한 결과 모두 원문
WMF/EMF와 SHA-256이 같다. 그림 bytes의 저장 누락은 관측되지 않았다.
Print PDF의 EMF 도형 세 쪽(12·13·14)은 이미지·벡터 그리기 항목이 없고
제목 텍스트만 있다. 원문의 그리기 상태와 한컴의 재생 조건을 추가 확인해야 한다.
이 관측만으로 RHWP의 도형을 숨기거나 EMF 원본의 정상 출력을 확정하지 않는다.
실행 원장은 ignored `stage175-primitive-saved-bin-reconciliation.json`과
`stage175-primitive-print-paint-inventory.json`에 보존한다.

입력 생성 조건·그리기 상태와 실제 출력을 먼저 검토·보정한다. 정상 원문과
독립 Print에서 전체 Native/fresh WASM 90% 이상 및 확대 판독을 충족하기 전에는
관련 기존 검사·golden의 기대값을 갱신하지 않는다. TSV·로그는 Git에 넣지 않는다.

## Stage 176 한컴 저장 속성과 헤더 대조군

[한컴 정상 저장 HWPX](metafile-primitive-roundtrip-2024.hwpx)는 그림 14개와 원문
bytes·crop/imgDim을 보존한다. curSz는 0으로 저장된다. WMF에도 같은 값이 있으므로
이 값으로 EMF만 숨기는 근거로 삼지 않는다.

- Save job: `e0a60073-24d8-41b3-bdaa-fa6637b78f67`
- HWPX SHA-256: `b9669a74b89fcce396cf8893b0b56e68c385b9aa4730572516b411c20519ab5c`

88/108바이트 헤더와 WHITE_BRUSH·BLACK_PEN 명시 선택 여부의 조합을 사각형·타원·
직선 각각에 적용한 12쪽이다. 한컴 정상 저장본의 그림 컨테이너를 사용하고 그리기
명령·Bounds·Frame·장치 크기는 유지했다. 108바이트 헤더에는 pixel format/OpenGL
값 0과 장치 크기 508000×286000 micrometers를 기록했다. 원문 기본 상태의 88바이트
세 도형만 Print에 없다. 나머지 아홉 도형은 출력됐지만 변경 전 Native도 모두 미달이다.

- [추가 입력 HWPX](metafile-emf-header-v2-input.hwpx): SHA-256 `4d7e267a7eeac60f114ca51bdecf0a37e3461e16770bf830469a963d4b4c4b19`
- [한컴 2024 저장 HWP](metafile-emf-header-v2-2024.hwp): SHA-256 `9142fd8700ee668b6b0df0e493a59cdbed878c1b6f0b93af7f7df410dfcb7f2b`
- [한컴 2024 Print PDF](metafile-emf-header-v2-2024.pdf): SHA-256 `95ec63ef997a1a96779bb97fa3044e85c732cc96c771e087c7d8299c5f55f62c`
- Save job: `884406df-35fb-41fb-b5a1-c69b3d6c1dcb`
- Print job: `80f1f330-f335-4077-a20e-dbf17416e023`
- 실제 한컴 버전: `13.0.0.3901`, method `0`, `frame_print_to_pdf_ex_one_up`

변경 전 생산 SHA `8e4f09c48`의 Native 전체 12쪽 최저 33.06477%, 모두 미달이다.
Windows GDI에서는 12개 헤더 대조군 모두 재생됐다. GDI 진단은 Print 기준 대체가 아니다.
서로 다른 결과와 입력을 보존하며 새로운 회귀의 기준 출력으로 승인하지 않았다.

기존 `Bounds → 출력 사각형` 매핑을 Frame·참조 장치 크기로 바꾸고, WMF의
중간 마스크가 XOR 영역을 넘는 경우의 검은 칠을 보정한다. 변경 후 Native/fresh WASM
검증은 별도로 기록한다. 선의 cosmetic 단위와 패턴의 장치 주기도 남은 검토 대상이다.
