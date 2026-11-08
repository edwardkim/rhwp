# 개인정보 두 원본 프레임의 물리 종료와 후속 원점 보존

Issue: #7445. Stage 50과 같은 원본 범위다. 추가 문서·회귀 검사·기대값·허용치·지침·원격 변경은 없다.

## Producer

- parent HEAD: `546e140784c431074e85af9a08b7409073c28611`. 런타임 패치 SHA-256: `a63adcdf5371c78365c9a88fae8dfc02c0b40386c2e5bf917d9dd39f9634ea82`.
- frozen Native CLI SHA-256: `99ec6ced652763930262591e3e204872e5b9c644696cf75f3acfd0f2b9310f8f`.
- frozen library SHA-256: `7dff2ac99bb73d4ea6282a6fbb930004e4cf8551e90a1d9ef50ab7e5f8df9118`.
- 입력 `samples/task2097/75544_pii_bunseok.hwpx`: `f429768ec02e5a849d08c06678cbee029fdc8618ca9e2bdcdbba485087fe71f1`.
- 독립 한컴 2020 Print `pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf`: `0fef38f44b3825f1c97869c2720112536504eaf30cec2a2d2af6d4399ffe8770`.
- 선언 환경 `mydocs/pr/assets/issue7445/resolution-20261005/pii5846-reference-font-environment.json`: `21e5c5a0b081bf3101749e5b80c1b053fa3eeff4fcb1ba3f8aa33fd3e2a4d544`.
- 같은 글꼴 713개 hash, Native Skia Print/compat2022/96DPI, canonical PDFium152.0.7947.0 smooth RGB, raw 2px/90%, capture 8이다. 후보 전 66쪽을 새로 출력했다. base와 수정 전은 동일 조건으로 고정한 기존 producer의 전쪽 증거다.

- `src/renderer/float_placement.rs`: `220c5570bf52b2d694ee576531e37ad348730438845b6ab7d47273f8449b47df`
- `src/renderer/layout/table_layout/stored_ordinary_row_frame.rs`: `2c8234a1c891b3d53f8347f6c6463495024267046948c6a12a86993284a26789`
- `src/renderer/typeset/table/continuation/fragment/emit.rs`: `e33bf1e23215baaaa226871c38bf0571dd9403d6f476c3376bc156308c540ea7`
- `src/renderer/typeset/table/continuation/fragment/scan.rs`: `f7bbc90cc8de3b2e6012461449f6a28e96801db0014164cc176792ad604251ad`

## 원인과 소비

원본 517번 표의 셀 높이 83771 HU, 첫 개체 상자 42265 HU, 상하 바깥여백 각 283 HU, 후속 줄 원점 42072 HU는 두 물리 프레임을 독립적으로 닫는다. 마지막 내용 프레임 538.16px만 남기면 빈 종료 공간이 사라져 가운데 정렬도 약 7.63px 위로 이동한다.

`stored_single_cell_two_frame_opening`은 실제 한 번의 저장 재시작과 같은 후속 종료식을 검증한다. scan은 이미 늘어난 페이지 잔여가 아닌 실제 컷 내용 하한과 가용 예산으로 이 원본 상자를 수용한다. emit도 같은 query를 소비하고, `end_row_owns_stored_frame`으로 그 물리 상자의 소유를 partial paint까지 전달해 끝 간격을 다시 빼지 않는다. 기존 잉크 끝 종료는 유지하며 별도의 두 프레임 빈 밴드는 기존 closing query의 정확한 종료식으로만 인정한다. 새 accepted row 장부를 유지한다.

초기 P37은 57쪽을 100%로 개선했지만 원본 소유 표시를 끝까지 전달하지 않아 56쪽 괘선 높이를 다시 9.33px 줄였다. 전쪽 비교에서 100→98.11628% 하락을 검출해 미수용으로 보존했다. P38은 같은 인증 결과를 최종 소비자로 전달해 56쪽을 회복했다. 픽셀 공차나 소유 기준을 완화하지 않았다.

## 실행 결과

| 범위 | 수정 전 P36 | 후보 P38 |
| --- | ---: | ---: |
| 56쪽 Native | 100.00000% | 100.00000% |
| 57쪽 Native | 69.57002% | 100.00000% |
| 60쪽 Native | 98.32834% | 98.32834% |
| 전쪽 최저 | 46.20358% | 46.20358% |
| 90% 미만 | 57·59·66쪽 | 59·66쪽 |

후보는 66/66쪽이다. PNG 변경은 56·57쪽이며 나머지 64쪽은 byte 동일하다. 전쪽 점수 하락은 0쪽이고 소유별 본문 키 1826개의 연결 문자열 변화는 0이다. 이는 전역 읽기 순서의 단독 증거가 아니다. 56·57쪽 review와 overlay를 직접 열어 앞뒤 표 경계·이어받기 본문·다음 제목을 확인했다. 일부 잉크 가장자리와 선 굵기 차이는 남으므로 raw 100%를 픽셀 완전 일치라고 부르지 않는다.

기존 집중 회귀는 113건 중 112 PASS / 1 FAIL, test 23.416초다. #5846의 65·66쪽 설명 칸 (4,1) 소실이 남아 함수 전체는 FAIL이다. #4138·#7752 및 #7095·#5714 대조군은 PASS다. 기존 함수가 누적한 57·58쪽 종료 관계는 공개 SDK에서 별도로 같은 원본·환경·frozen lib로 실행했다. 수정 전은 57쪽 545.706666667 대 후속 원점 560.96으로 FAIL, 후보는 560.96 대 560.96으로 PASS이며 58쪽은 양쪽 848.0을 유지했다. 이 SDK 진단은 정식 전체 회귀 PASS가 아니다. 첫 SDK의 usize/u32 컴파일 오류는 보존 후 수정하고 실제 실행 결과만 인용했다.

같은 base 대비 41쪽 99.23712→97.93657% 하락과 59·66쪽 미달을 계속 보류한다. fresh WASM 전쪽·전체 nextest·최종 필수 게이트·actual36는 이 결과로 대체하지 않는다.

## 전쪽 TSV와 직접 판독

[base](base-native.tsv) · [수정 전](before-native.tsv) · [후보](candidate-native.tsv)

| 조건 | review | overlay |
| --- | --- | --- |
| 수정 전 57쪽 | [review](before-native-p057-review.png) | [overlay](before-native-p057-overlay.png) |
| 후보 56쪽 | [review](candidate-native-p056-review.png) | [overlay](candidate-native-p056-overlay.png) |
| 후보 57쪽 | [review](candidate-native-p057-review.png) | [overlay](candidate-native-p057-overlay.png) |

## 제출 파일 SHA-256

- `base-native.tsv`: `9c7fd36cbb4e8eb417cf142df62d1f6dfd886574069c399aae3844e71a74ceac`
- `before-native.tsv`: `63dedb9763bfdd4396c2e10cdc17a6f04252052cd78f80a48b7ed6c00c44b774`
- `before-native-p057-review.png`: `2a775ad6cb3b219d2781a60aaf21031079c592a0d4b4f19834ae72d116aa7cd8`
- `before-native-p057-overlay.png`: `4af4b9763b4d42d1adb68b7e0817bc8160280c51096d9fc3283c9512b9b1399a`
- `candidate-native.tsv`: `7f9cca563333858d85a9fe15a7b3107462606dd68b30bee605216b8f49269a53`
- `candidate-native-p056-review.png`: `d51f9f0daf43ada1074493190df21be26e47cd60ced65a028373ea788b184ab2`
- `candidate-native-p056-overlay.png`: `e678074a5a0961931f6d031b24b469012cafc9decf3c2c0cc000759149d1c59e`
- `candidate-native-p057-review.png`: `b77722f57a3f266a5c0a6e58a12fba00045d6bf1233c319bfe46812f07bbb457`
- `candidate-native-p057-overlay.png`: `0109428a0676716cf850ad3c07bce982267e0490153d1a8510700d1079f1d4fc`

실행 로그·중간 JSON·SDK·미수용 P37은 ignored output의 `repair-focused-v37/v38`, `pii-two-frame-native-v37/v38`, `pii-two-frame-cause-v37/v38`에 보존한다.
