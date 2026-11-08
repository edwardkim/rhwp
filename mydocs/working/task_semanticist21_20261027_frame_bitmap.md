# 통합 PR 준비: 저장 프레임과 투명 비트맵 보정

현재 단계의 시작 head는 `5a12bf20e2e0aca40d2edf11d0a44a2bdfb6c5c8`, 고정 base는
`49bc935a149c20b942d3045c2f5c733aeccf57d0`다. 기존 45개 PR·리베이스 기능만 검증한다.
새 실제 HWP나 contributor PR은 추가하지 않는다. Cargo는 공용 `target/pr-review`,
8 jobs를 순차 사용하며 test threads는 12다. 중간 JSON·TSV·실행 로그는 ignored
`output/pr-review/semanticist21-20261019/`에 둔다.

## 독립 근거와 공통 원인

#8032의 실제 RenderTree는 x113.4/y153.6/w512/h204.8이며 별도 부모 변환은 없다.
독립 한컴 Print PDF의 RGB XObject는 FlateDecode이며 1비트 `/Mask`를 참조한다.
이 입력을 JPEG·soft mask라고 설명한 이전 가정은 잘못됐으므로 실제 PDF 객체로 정정했다.

4378~4387의 같은 입력 변형 Print에서 정렬·앞 문장 길이·PNG 메타데이터는 결과에
영향이 없었다. 동일 RGBA 재인코딩은 RGB·마스크 바이트 해시도 같았다. 불투명 대조군은
원래 프레임을 유지했다. 원본 픽셀 16·32·48, 종횡비·알파 경사를 분리해 확인했다.

연속 알파 그림은 600 DPI 원본 픽셀의 정수 확대 격자에 RGB 운반체의 시작을 올림한다.
원래 16픽셀 그림은 장치 시작709→800, 마스크3200×1280이며 48픽셀 변형은 시작737,
RGB48×47·마스크3216×1269다. 알파는 원래 장치 프레임에서 Catmull–Rom으로 보간하고
전역16×16 threshold로 이진화한다. 운반체의 새 원점에서 디더를 다시 시작하지 않는다.
저알파 RGB·고정소수점 보간은 완전 일치하지 않으며 이를 정확한 한컴 codec 복제로 주장하지 않는다.

## 구현과 실제 소비

`PrintRasterFrame::new → image_resolver/print_alpha.rs → PaintOp::image의 출력 bbox·payload →
bitmap_raster의 공통 premultiplied 면적 축소 → Skia/Canvas`가 실제 소비 경로다.
IR·문단 소속·줄 구성과 저장 속성은 변경하지 않았다. 기존 Catmull–Rom 가중치는
공통 함수로 재사용한다. 2020 Print·효과 없는 연속 알파 PNG·전체 crop·축 정렬 확대에
적용하며 이진 알파·회전·축소·2024·부분 crop은 기존 계약을 유지한다.
600 DPI 출력 버퍼는 4,194,304픽셀, 메모는 기존 16 MiB 상한을 적용한다.

## 실제 실행 결과

| 범위 | Native266 | 최적화 fresh079 |
| --- | --- | --- |
| 원래 넓은 그림 | raw98.78322%, 수정 전90.69451% | raw98.78261%, 수정 전89.93941% |
| 원래 좁은 그림 | raw100% | raw100%, 수정 전99.95997% |
| 앞의 두 입력을 포함한 대조13조건 | 전부90% 이상 | 전부90% 이상 |
| #8188 상속4입력×64쪽 | 전256쪽 raw100 | 전256쪽 raw100 |

512개 상속 PNG는 직전 자격을 갖춘 Native265/fresh078과 파일 해시도 모두 같다.
`print-alpha-inherited-both-preservation-4419.json`은 정수 미일치0/0과 이전 해시를 대조했다.

Native 빌드는 `print-alpha-native-build-4394-result.json`, 바이너리 SHA-256은
`24a3930317c9af84a8483aaa2fc1003213a2915a36e26644067fcaf1b97ae781`이다.
표준 release·실제 wasm-opt -O 빌드는 `print-alpha-wasm079-build-4400-result.json`이며
17분53초 뒤 exit0을 확인했다. WASM SHA-256은
`24fd7c188e0386a65ed29506db8e4ed56caeb432305575cfad8c5536266e5670`이다.
`pkg`·Studio public·고정 fresh079의 해시가 같다. 두 빌드의 런타임 소스 해시를 기록했고
그 뒤 변경이 없음을 재확인했다.

Native266/fresh079의 넓은·좁은 입력 review와 standalone overlay를 직접 열었다.
큰 위치 차이는 해소됐고 앞 문장의 줄 소속·순서와 그림이 보존된다. 흐린 픽셀 가장자리는
독립 PDF에도 있으며 일부 알파/RGB·글자 raster의 미세 차이는 남는다. raw와 엄격 색상
일치를 혼동하지 않는다. 입력·PDF·전쪽 TSV·대표 PNG는
[정식 증거](../pr/assets/pr_8032_print_alpha/README.md)에 보존했다.

## 새 회귀의 검출력

기존 검사는 유지하고 `tests/cases/print_alpha_source_grid.rs`를 새 이름으로 추가했다.
양쪽98% 이상·직접 판독한 두 입력에서 독립 PDF 운반체를 ±2px로 검사하고 원문·그림의
개수·순서·줄 관계, 이진 알파와 화면 프로필의 원본16×16 표본 보존을 함께 보호한다.
기대값은 PDF의 CTM·명시적 Mask에서 읽었다. helper 계산값과 전체 SVG 해시를 재인용하지 않는다.

동일한 최종 검사 소스를 standalone rustc로 각각 고정 producer에 연결했다.
수정 전 Native265는 x113.379023 대 독립 기준127.930664에서 FAIL(exit101),
수정 후 Native266은 PASS(exit0)이며 양쪽 컴파일은 exit0이다.
`print-alpha-red-green-4425-result.json`과 `logs/print-alpha-{red,green}-4425.log`에 있다.
정식 자동 suite 경로의 결과는 실행 종료 뒤 아래에 추가한다.

## 다음 보정의 원인 분리 — 원본 통과로 보고하지 않음

#7833 원 합성 입력은 본문 문단을 셀로 복사하면서 저장 폭42520과 첫 vpos3200 등을
유지했다. 한컴 저장 진단에서는 폭19956·첫 원점0으로 바뀌었다. 저장본은 속성 분석용이고
원래 Print PDF를 기준으로 계속 사용한다. 폭 초과·양수 첫 원점의 조합은 기존 #8188
HWPX3190개 셀 문단에서0건, 원 입력10개 중8건이다. 이것만으로 수용 조건을 바꾸지 않는다.

초기4401의 `clear-cell-lines`는 정규식 tail 제거로 셀 뒤의 바깥 호스트 캐시까지 제거했다.
이를 셀 전용 변경으로 설명한 것은 잘못됐다. JSON에 정정했고 DOM 범위를 분리한4404로
다시 확인했다. 원래 Print 대비 실제 진단 결과는 다음과 같다.

| 같은 원본의 진단 사본 | raw |
| --- | ---: |
| 모든 저장 줄 제거 | 100.00000 |
| 바깥 문단만 제거 | 28.69008 |
| 셀 문단만 제거 | 21.20614 |
| 저장 폭만 물리 내폭으로 변경 | 18.37387 |
| 셀별 첫 원점만 차감 | 19.02624 |
| 폭이 맞지 않는 문단의 캐시만 제거 | 19.26652 |

폭이 맞지 않는 셀 프레임과 이에 의존하는 바깥 호스트 캐시를 함께 제거한4405의8개
진단 사본은 전부100%였다. 원본·기준 PDF는 그대로이며 진단 사본 통과를 원본 통과로
바꾸지 않는다. 다음 단계는 부모·셀의 저장 프레임 소유를 함께 판단하는 공통 로직이며
유효한 저장·정상 대조군과 실제 원본에서 다시 검증해야 한다. 아직 생산 코드를 바꾸지 않았다.

#8031/#8039의 진단용 한컴 HWP Save(4406~4408)는 0% Percent 문단에서 표 줄 높이4000,
후행 간격−1000, 다음 문단 원점3000을 기록한다. #8039의 두 저장 사다리는 같은 원점으로
정규화된다. 누락 글자 속성의 암묵 크기는1000 HU이며 저장 face는 한컴바탕 계열,
원래 Print에 실제 내장된 프로그램은 Gulim이다. 기본 스타일·줄 전진·저장 캐시의 실제
소비를 함께 확인하며 임의 글꼴·좌표 핀으로 대신하지 않는다. `minimal-ir-hancom-normalization-4409/`와
`minimal-ir-independent-fonts-4410.json`에 실제 관측값을 보존했다.

원래 #7833 입력8조건, #8031/#8039 최소 입력과 최종 전체 Rust·lint·Studio·전쪽 base/head
대조 및45개 리뷰의 최종 갱신은 남아 있다. 현재 결과를 통합 PR 준비 완료로 확대하지 않는다.


## 정식 종료 확인

자동 수집은 `regression_suite_005::print_alpha_source_grid`로 연결됐고 정식 nextest는
1 PASS·0 FAIL·275 filter-skipped, exit0이다. 캐시를 재사용한 최종 실행 15.85초 뒤 최종 summary를
확인했다. 실제 새 검사 함수는 `continuous_alpha_print_keeps_text_order_and_independent_carrier_frame`,
소스 SHA-256은 `6aafd2eeff54b6b666bfcdad02b78783ae3cce530d305b8767234a2d8dd59972`다.
`print-alpha-auto-regression-4426-result.json`, `logs/print-alpha-auto-regression-4426.log`에
전체 명령과 종료가 있다. 새 전체 회귀 통과로 확대하지 않는다.

고정 base의 manifest policy4422는 PASS이며 4개 변경 Markdown의 링크 오류는 0건이다.
새 구현 216줄·새 검사 160줄로 각각 1000줄 이하다. 빌드·시각 검증 이후 런타임 소스 해시가
변하지 않았음을 확인했다. 파생 suite를 준비한 뒤의 최종 cargo fmt --all -- --check, 고정 base manifest check,
4개 문서의 링크 검사와 git diff --cached --check를 모두 exit0으로 확인했다.
증거는 logs/print-alpha-final-fmt-check-4427.log·print-alpha-final-manifest-check-4427.log와
print-alpha-final-markdown-links-4424.log에 있다.

다음 #7833 검토를 위해4423에서 정상 상속 입력의 유효 패딩을 적용해 다시 조사했다.
3190개 저장 셀 문단 중 내폭보다2 HU 초과인 줄은 196건이며 모두 첫 원점0이다.
따라서 **폭 초과만으로 캐시를 무효화하면 정상 저장 프레임도 바꿀 수 있다.**
수치 조합을 덧붙여 통과 대상을 좁히지 말고, 원본의 프레임 소유와 재구성 이유를
독립 출력·정상 대조군으로 먼저 입증해야 한다. 아직 그 생산 코드를 추가하지 않았다.
