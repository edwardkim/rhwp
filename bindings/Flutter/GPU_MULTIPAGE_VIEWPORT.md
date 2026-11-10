# 다중 페이지 GPU 참조 viewport

P111은 기존 `NativeDocumentViewport`의 authoritative page metadata·배치·입력 좌표를
같은 GPU target의 여러 placements로 연결한다. 문서 parser·측정·조판·pagination·Native renderer와
플랫폼 plugin ABI는 바꾸지 않는다. 기본 CPU와 단일 페이지 texture API를 유지한다.

```dart
NativePagedDocumentView(
  document: document,
  controller: pageNavigation,
  gpuController: textureController,
  onControllerChanged: observeEditor,
)
```

`textureController.document`는 같은 `NativeDocument`여야 한다. 생성 호스트는 controller를
수명 동안 한 번 만들고 view를 mount한 상태로 `close()`의 retirement 결과를 확인한 뒤 owner를
닫고 notifier를 dispose한다. GPU와 CPU fallback을 위해 별도의 입력·IME·semantics owner를 만들지 않는다.

## 게시와 소유권

- `NativeTextureController.configureDocument(layout)`는 보이는 1–4쪽을 batch로 계획한다.
  화면 target·DPR·좌표·scale을 기존 한도로 검사한 뒤 전체 snapshot credit을 먼저 예약한다.
  isolate의 caller-owned/pending/ambiguous snapshot 합계 4개 제한은 그대로다.
- owner FIFO에서 각 page를 순차 capture한다. 모든 snapshot의 document·revision·font·view·device·DPR
  stamp가 같고 전체 source check를 통과해야 한 번의 `submit`으로 보낸다. 배치 일부만 표시하지 않는다.
- 여러 page에는 모든 snapshot ID가 있는 `acceptedSnapshots`가 필요하다. 누락·transport timeout은
  거절로 추정하지 않는다. 이후 status로 개별 소유권을 확인할 수 없으면 ambiguous credit을 유지한다.
  확정 거절·capture 실패의 caller-owned snapshot만 한 번 해제한다. 늦은 원 RPC/capture 관측도 남긴다.
- publication의 stamp뿐 아니라 전체 page ID receipt와 각 snapshot의 full source check를 확인한 뒤에만
  cover를 벗긴다. 편집·navigation·resize·DPR 변경 중에는 이전 geometry를 새 화면으로 노출하지 않는다.
- `suspendViewport()`와 `NativeTextureView.document(documentViewport: null, ...)`는 등록 view를
  mount한 채 즉시 cover한다. 새 layout을 공급할 때까지 capture·publication polling을 멈춘다.
  close·device generation·process-wide plugin budget을 초기화하거나 consumer 완료를 만들지 않는다.

## CPU fallback과 범위

GPU가 정상인 동안 page metadata만 읽고 CPU region raster/decode는 만들지 않는다. 확정된 지원 불가·
장치 실패·backpressure 뒤에는 동일한 viewport와 editor owner로 기존 bounded CPU 경로를 실행한다.
문서/resource 오류는 실패 진단으로 유지한다. CPU가 기본인 경우 기존 P105 동작과 image batch 해제를 유지한다.

Native/Flutter의 page transform과 DPR은 한 번만 적용하며 hit-test·caret/selection·IME도 같은 배치를 쓴다.
현재 target 4096 dimension·4,194,304 pixels, visible region 16 MiB·4쪽 한도는 유지한다.
Linux·Windows의 명시적인 P112 tiled controller는 [최대 두 GPU 타일](README.md#최대-두-gpu-타일-p112)의 별도 geometry 정책과
낮은 DPR의 전체 CPU fallback을 사용한다. 이것이 GPU consumer fence·기존 target 재사용·물리 GPU 성능이나
전체 driver 메모리 상한을 제공하지 않는다. Windows/macOS legacy immutable credit과 연속 opt-in 계약도 유지한다.

## 검증

고정 Flutter 3.47.9에서 다음 source-level/widget 계약을 실행한다.

```sh
flutter analyze --no-pub
flutter test --no-pub --concurrency=1 \
  test/native_texture_multipage_test.dart \
  test/native_texture_view_test.dart \
  test/native_texture_continuous_test.dart \
  test/native_metal_view_test.dart \
  test/native_multipage_view_test.dart \
  test/native_presentation_test.dart \
  test/native_presentation_capture_test.dart \
  test/native_document_lifecycle_test.dart
```

새 검사는 Linux·CV macOS·AppKit Metal·Windows의 같은-stamp batch와 placement 관계,
전체 credit 예약, 일부 capture 실패/timeout/late reply, latest-only navigation, suspend cover,
부분 receipt와 published page 누락, CPU fallback의 lazy 전환과 기존 editor owner 보존을 확인한다.
플랫폼 channel·owner transport를 대역으로 실행한 계약 검사이며 실제 compositor의 여러 페이지
sampling·최종 한컴/전체 코퍼스·실물 GPU/IME·AT 검증과 구분한다. 기존 단일 페이지 실제 run을
새 다중 페이지의 실제 화면 실행 증거로 바꾸지 않는다.
