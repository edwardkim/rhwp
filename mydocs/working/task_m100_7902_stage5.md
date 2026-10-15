# #7902 5단계 — raster 정상 대조군과 이어받기 물리 공간

Issue: #7902. [4단계](task_m100_7902_stage4.md)의 원본 HFT 공급과 전체19문서·835쪽 범위를 유지한다.
두 backend의 전쪽90%, devel 대비0.01%p 무회귀, 완료 쪽98%, 쪽수·내용 보존과 직접 판독을 모두 적용한다.
2026-10-16 fetch에서도 `upstream/devel`은 `2efefc6a1`로 같아 리베이스하지 않았다.

## 완료한 전체 검증과 채택할 수 없는 결과

- production Rust `484c9bfa1`의 release-test 전체 nextest: **10,377 PASS, 45 skipped**, exit0.
  실행1222.347초, 빌드 포함2599.34초. Native/fresh WASM 빌드, fmt, manifest·unit-tier,
  default/Native/WASM/workspace Clippy, workspace build, 문서 링크도 모두 통과했다.
  검증 도중 추가한 `5376e9ce0`은 CI 연결, `5dbca682a`는 독립 Python CFF bearing 수정이며
  해당 Rust producer에는 반영할 runtime 변경이 없다. 이 구분은 `final-484c-source.json`에 있다.
- Native/fresh WASM 각각19문서835쪽, 전쪽90% 이상, 독립 PDF 쪽수 일치, 누락0이다.
  그러나 같은 원본 HFT를 공급한 devel과 Decimal로 대조하면 **Native131쪽·WASM203쪽**에서
  0.01%p 이상 하락했다. 최대0.22981%p다. 이 상태는 **PR 보류**다.
  `final-484c-validation-result.json`, `final-484c-develop-comparison.json`에 명령·원본 TSV와 SHA가 있다.
- 전체 본문에 적용한 baseline-snap(false)/geometricPrecision 가정을 철회한다.
  현재 코드에서 동일한 Noto glyph의 TTF 기존 grid 대조군이0.200871px 이동으로 실패했다.
  `source-font-origin-contract-before-{result.json,build.log,test.log}`에 public DocumentCore→PaintOp,
  원본 종류, 입력 이동량과 실제 잉크 이동량을 연결했다. 공유 library는 읽기만 했다.
- 원본 HFT 종류만으로 새 정책을 제한하는 안도 채택하지 않았다. 중첩 표 HWP/HWPX 문서의
  WASM 하락이 각각75쪽이므로 소스 종류 분기만으로 전체 무회귀를 설명할 수 없다.

## 독립 Print 장치 좌표 진단 — 일반 renderer에 적용하지 않음

표적 문서와 독립적인 HFT 알파벳 표본의 용지만 A3·Letter로 바꾸어 실제 한컴2020
PrintMethod0 PDF를 만들었다. 입력·PDF SHA, 실제 engine/version/job과 행렬은
`print-device-independent-counters/result.json`에 기록했다. 원본/공유 target은 수정하지 않았다.

- A4의595×841pt, A3의841×1190pt, Letter의612×790pt를 관측했다.
  최초 `floor(원본 pt)` 가설은 Letter에서 **반증**됐다. 그 실패 기록도 보존한다.
- 정수 mm와600dpi 장치 픽셀을 거치는 수정 모델은 세 용지의 PDF 행렬을 소수6자리까지 재현했다.
  이 반올림 규칙은 독립 출력에서 얻은 **경험적 추론**이며 공식 한컴 사양이라고 주장하지 않는다.
  GDI의 mm/pixel 정수 반환형은 [GetDeviceCaps 공식 문서](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-getdevicecaps)를 참고했다.
- 정확한 논리 기준선908.84px에 이 모델을 적용한104쪽 차이는 약0.04px다.
  앞서 compact tree의 반올림된908.8px로 계산한0.001px 주장은 정정한다.
  `precise-paper-coordinate-source/result.json`은 좌표 판독용 SVG 한 쪽이며 SVG sweep 증거가 아니다.
- 출력 root에 이 장치 변환을 적용한 별도 Canvas 진단은 nested104쪽98.10359%,
  TTF40쪽95.34801%, rowbreak7쪽88.68001%로 악화됐다. geometricPrecision을 함께 써도
  nested104쪽98.11018%다. **이 변환을 renderer나 비교기 정규화로 채택하지 않는다.**
  `print-device{,-gp}-diagnostic-result.json`에 보존한다. 독립 PDF와 원래 DPI·측정값은 그대로다.

## 현재 원인 추적과 다음 수정

1. 과도한 raster 변경 네 파일을 4e2c554b3의 기존 정책으로 복구한다. 생성 HFT 번호가 이미
   사용하던 fixed_glyph_advances 소수 원점 계약은 유지하고, 일반 TTF grid 정상 대조군을 추가한다.
   실패한 일반 본문 quarter-pixel 가정을 정확한 기존 계약으로 교정하며 source-side quota는 올리지 않는다.
2. 중첩 표104쪽의 부모4,361개 유닛 높이는 base/candidate가 전부 같았다.
   consumed_px 누적 손실 가설은 반증됐다.140HU는 문단 앞 간격이 아닌 자식 표의 바깥여백이다.
3. `cell_units`의 마지막 terminal band와 `nested_continuation_header_height`의 시작 띠,
   `nested_table_split_from_cut_units`가 생성하는 높이, `with_nested_header_reservation`의 반환값,
   `table_partial`의 실제 자식 원점과 후속 para_y를 함께 대조한다. 같은 helper 호출만으로
   공간이 한 번 소비됐다고 단정하지 않는다. 빈 밴드·내용 유닛·후속 문단을 분리해 기록한다.
4. 원인 보정 후 TTF14·40쪽, nested104쪽, 번호29·100쪽, 표 시작/이어받기/종료를 먼저
   Native/fresh WASM으로 확인한다. 이후 전쪽 무회귀·확대 판독·해당 필수 gate를 새 source로 확인한다.
   비교 점수의 완화, 문서/좌표 전용 보정, 여백 숨김으로 통과시키지 않는다.

현재는 원인 추적과 정상 raster 복구 단계이며, PR 준비 완료나 최종 승인으로 보고하지 않는다.
사용자가 수정한 `rhwp-studio/public/rhwp.js`는 그대로 보존한다.


## 과도한 raster 정책 복구 검증

- Native 일반/폴백 font의 baseline-snap과 Canvas painter/provider 네 파일을
  `4e2c554b3` 정책으로 복구했다. 생성 번호의 `fixed_glyph_advances` 계약은 유지한다.
- 정식 `native_fractional_glyph_origin` 통합 검사2개 모두 PASS, exit0이다.
  실제 Cargo Native 빌드136초·검사0.84초이며 일반 TTF의 기존 격자 대조군도 보호한다.
  앞의 standalone TTF FAIL은 같은 Noto 입력 이동에 대한 수정 전 검출 증거다.
- Studio 전체1,875개 중1,873 PASS·2 skipped, TypeScript noEmit exit0이다.
  `raster-normal-control-{after,studio-test,studio-tsc}.log`에 보존한다.
- 이 변경은 최종 시각 무회귀 완료가 아니다. restored producer의 전체 TSV와 남은104쪽
  원인 보정·판독을 마치기 전에는 이전484c 결과를 새 source의 승인 증거로 사용하지 않는다.
