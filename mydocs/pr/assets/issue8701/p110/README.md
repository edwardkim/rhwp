# P110 macOS Native Metal view 실행 증거

[`evidence.json`](evidence.json)은 실제 macOS WindowServer에서 수행한 AppKitView/CAMetalLayer
32회 표시·편집·resize·mounted held-cover·종료 결과를 요약한다.
[GitHub Actions 실행](https://github.com/seo-rii/rhwp/actions/runs/42455059831/job/128652215657)은
성공했으며 원본 artifact ID는 `13607656144`다. `sourceRevision`은 실제 실행한
`ebd10e523d8f33b864791d3fdf893e00c29f87cd`, `evidencePreparedAgainstRevision`은
동일한 Flutter runtime/검증 도구를 포함한 공개 준비 코드 `0b67183ed3120a480ccbe87ca0a27db3d10b7f9b`다.
후속 증거 commit과 실제 실행 source를 구분한다. 차이가 있는 두 파일은 fork CI 전용 classifier/test다.

검증 입력은 저장소의 [desktop-consumer.hwpx](../../../../../bindings/Flutter/test/fixtures/desktop-consumer.hwpx)
1쪽이다. 8,082-byte 원본의 SHA-256은
`648dade82585fa2845b80ec510b5f55c67e8b6127688dd3748b9a2068d87a3d9`다.
첫 게시 뒤 Native 문서의 그림 앞에 빈 문단 여섯 개를 삽입해 revision을 올리고,
viewport를 320×240에서 352×256으로 바꾼다. 이후 같은 revision에서 viewport 원점을
번갈아 변경해 32개의 view generation을 게시한다.

| 단계 | Native CPU | 실제 WindowServer viewport | 독립 4배 CPU reference |
| --- | --- | --- | --- |
| 첫 표시 | [CPU](cpu-01.png) | [화면](display-01.png) | [source](reference-01.png) |
| 편집 revision | [CPU](cpu-02.png) | [화면](display-02.png) | [source](reference-02.png) |
| 물리 target resize | [CPU](cpu-03.png) | [화면](display-03.png) | [source](reference-03.png) |
| 32번째 게시 | [CPU](cpu-32.png) | [화면](display-32.png) | [source](reference-32.png) |

12개 PNG를 직접 확인했다. 빨간 도형·파란 테두리·PHOTO 글자가 CPU와 실제 화면에서
같은 위치에 표시되며 편집 뒤에는 도형이 아래로 이동한다. resize 뒤에는 fit scale이 바뀐다.
32번째의 왼쪽 4px·위쪽 2px 회색 영역은 기록된 viewport 원점 `(4,2)`에 대응한다.
PNG는 원본 RGBA를 크기·색상 변경 없이 lossless 인코딩했고, 다시 디코딩한 RGBA가 원본과
byte 단위로 일치한다. display는 실제 640×480 창의 `(16,16)`에서 시작하는 viewport crop이다.
같은 immutable snapshot·full stamp의 CPU와 4배 reference를 독립 재생했으며,
대표 8개 실제 sample의 비교를 원본 CPU/reference bytes로 다시 실행했다.
전체 67개 sample의 CPU 판정과 32개 선택 쌍은 원본 영수증에 기록되어 있다.

각 sample은 기존 `opaque-interiors-1-source-palette13-edge1px-zero-integer-shift`
계약으로 검사한다. 균일 내부 채널 오차 1, 한 pixel 경계와 독립 source palette,
유일한 zero-shift 정렬을 유지한다. 같은 게시의 여러 화면 사이에서는 metadata·alpha가 같고
RGB 전체 범위가 1 이내여야 하며, 최대 네 번 안에 연속 두 장이 byte 단위로 같아야 한다.
15·21·24번째에서만 세 장을 수집했고 각각 마지막 두 장이 정확히 일치했다.
다른 29개 게시에는 두 장씩 사용해 총 67장이다. CPU·metadata·alpha·범위·deadline 실패를
건너뛰지 않는다. cover는 같은 AppKitView를 mount한 상태에서 Native hide와 opaque Flutter
cover를 함께 적용하고, 정확히 동일한 단색 화면 두 장을 확인했다.

실제 Apple SDK plugin/bridge compile·link, Foundation CV close 상태 회귀와 독립 Metal의
opaque·premultiplied alpha·stale target 보존 검사가 모두 통과했다. Native dylib는
source `4f738e766ccbba8faf938f2421ca6b516e939019`의 기존 빌드이며 SHA-256은
`dbeafcf42729be197364c1502434e4eeb61a91e9966b938df2906aeca886dcad`다.
이번에 Rust를 재빌드하지 않았다. Git mode/OID로 Native 입력 81,672개의 동일성을 확인해
재사용했다. Flutter revision·engine/framework hash와 원본 영수증 hash는 JSON에 보존한다.
종료 시 `ownerFinished=true`, `registered=false`, `nativeViewLayerDetached=true`이며
snapshot·frame·lease·source 등 Native 계수 9개가 모두 0이다.

실행 환경은 GitHub hosted `macos-15`의 **Apple Paravirtual device**다.
`consumerGpuCompletionVerified`, `hardwareAccelerationVerified`, 실물 GPU 검증은 false다.
현재 결과는 이 환경의 실제 SDK/Metal/WindowServer Native view 경로를 검증한다.
기존 CV texture 연속 표시, Flutter overlay 하나만으로 보이는 NSView를 가리는 경우,
실물 IME/AT와 최종 한컴 비교는 별도 범위다.

재현은 [Native view 설명](../../../../../bindings/Flutter/macos/NATIVE_VIEW_PRESENTATION.md)의
build/capture 순서를 따른다. 같은 고정 Flutter SDK/framework와 검증된 Native dylib를 준비한 뒤
`tool/check_macos_metal_sdk.py --enable-metal-experiment`로 SDK/producer 영수증을 만든다.
그 영수증으로 `macos/tests/qualify_native_view.py build`를 실행하고 실제 foreground
WindowServer 세션에서 `capture --frames 32 --timeout 600`을 실행한다.
Screen Recording 권한과 실제 Metal device가 필요하며 TCC 우회나 producer readback을
화면 캡처의 대체로 사용하지 않는다.
