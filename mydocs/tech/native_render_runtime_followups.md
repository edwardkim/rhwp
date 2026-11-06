---
kind: reference
status: active
canonical: mydocs/tech/native_render_runtime_followups.md
last_verified: 2026-11-06
---

# Native 렌더링 런타임 후속 계약

## P74: 타일 viewport와 합산 raster cache 제어

`renderTiledRegion`은 동기 세션 JSON과 worker command 양쪽에서 사용한다. 동기 요청은
`renderRegion`처럼 `page`, `x`, `y`, `width`, `height`, `scale`을 받는다. Worker 요청은
`target`, `fontGeneration`, `page`, `region`, `scale`을 받으며 기존 문서 revision과 글꼴
세대를 검사한다. 반환 frame과 버퍼 수명은 기존 계약을 재사용한다. 원점은 배율을 적용한
left/top의 floor이며, 크기는 right/bottom의 ceil까지다. 반환 픽셀은 premultiplied RGBA8,
sRGB이고 `byteLength = stride × height`를 만족한다.

`renderRegion`은 전체 페이지에서 정확히 crop하는 기존 API다. 새 타일 API는 고배율 대형
페이지에서도 제한된 고정 격자 타일로 viewport를 준비하며 두 API의 의미를 바꾸지 않는다.
`setRasterCacheLimit`의 `maxBytes`는 full-page snapshot과 tile snapshot을 합산한 보존
한계다. 0은 보존을 끄고 기존 raster를 비우며, 64 MiB 초과는 거부한다. Worker 제어 요청은
추가로 현재 `target`을 요구한다.

`rasterCacheStats`는 `entries`, `tileEntries`, 합산 `retainedBytes`, 타일 `tileHits`,
`tileMisses`, `tileReplays`, `tileEvictions` 등을 반환한다. 기존 `cacheStats`는 Skia
picture/font/image 등 renderer 자원의 통계로 유지한다. 이 값들은 출력 버퍼나 프로세스
RSS를 포함하지 않는다. Capabilities의 `renderTiledRegion`, `rasterCacheControls`는
`native-skia` 빌드에서만 true이며, feature가 없으면 새 요청은 `featureDisabled`를 반환한다.

검증 소스는 기존 `issue_536_native_runtime` case와 Native `session_contract.py`,
`native_worker_contract.py` 경로에 추가한다. 합성 blank 문서로 배율 8의 작은 viewport,
음수·소수 원점, warm tile 재사용, 보존 한계, 오래된 revision/글꼴 세대와 feature-off 경로를
검사한다. 최종 한컴 문서와 실제 OS GPU 표시 검증은 작업지시자가 별도로 수행한다.

## P76: 실제 RGBA 출력 크기에 따른 worker 예약

Worker의 두 region 요청은 고정 64 MiB 대신 실제 enclosing window의 `width × height × 4`
바이트를 예약한다. 예를 들어 논리 원점 `(-0.3, 0.2)`, 크기 `(3.4, 2.9)`, 배율 `1.25`는
정수 원점 `(-1, 0)`, 크기 `(5, 4)`가 되므로 80바이트를 예약한다. 이 계산은 feature에
의존하지 않는 `RasterRegion::pixel_window`로 추출했으며 Skia의 exact/tiled plan도 같은
함수를 호출한다. 실제 반환 버퍼의 capacity가 예약량을 넘는지 검사하는 기존 worker 방어는
유지한다.

유한하지 않은 값, 0 이하의 크기·배율, 지원하지 않는 배율, 좌표·픽셀 수 한계 초과는 제출
단계에서 `invalidArgument`로 거부한다. 음수 원점은 정상 지원한다. 거부된 요청은 기존
render를 대체하지 않는다. 바이트 예산 부족은 `byteCapacity`, 미수신 completion을 포함한
요청 수 초과는 `queueFull`이다. 예약은 completion 수신 때 반환하고, 취소된 요청도 자신의
completion을 수신할 때까지 credit을 유지한다.

이 예산은 host에 넘기기 전의 입력·출력 버퍼에 대한 한계다. Skia의 full-page/tile raster,
합산 보존 LRU와 작업·출력 bitmap의 128 MiB 한계는 기존과 같이 별도로 적용한다. 합성
runtime 검사는 작은 예산의 음수·소수 viewport, 다수 소형 요청, credit 반환과 잘못된
요청의 대체 방지를 다룬다. Native feature 경로는 기존 CI가 실행하는
`issue_536_native_region_cache`의 inline worker 검사와 FFI worker 계약에서 확인한다.
새 integration target이나 CI 호출은 추가하지 않는다.
