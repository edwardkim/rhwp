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

## P77: 실행 중인 타일 viewport의 협력적 취소

오래된 타일 render는 최신 render가 실제로 제출·수락되면 다음 타일 경계에서 중단한다.
큐에 남은 요청을 취소하는 기존 동작에 더해, `workerCancel`은 실행 중인 `renderTiledRegion`도
중단 요청을 받을 수 있다. `close`는 대기·완료 큐를 비우며 실행 중 타일 작업을 같은 경계에서
멈춘다. 실행 중인 개별 Skia replay나 픽셀 읽기는 끝까지 수행한다. 원본 `renderRegion`,
문서 열기·편집·저장 등의 동기 호출을 강제로 중단하지 않는다.

Renderer는 출력 할당 전, 각 타일 전, 최종 frame 반환 전에 continuation callback을 검사한다.
중단하면 `None`으로 전체 출력 버퍼를 폐기하므로 부분 frame을 반환하지 않는다. Native session의
`render_tiled_region_cancellable`은 같은 callback 계약을 제공하며, callback에서 해당 세션의
cache·renderer를 재진입하거나 변경해서는 안 된다. 완성된 immutable tile은 같은 revision과
font generation의 합산 LRU에 남길 수 있다. Capabilities의 `cooperativeTiledCancellation`은
이 동작의 지원 여부를 나타낸다.

Worker는 명시 취소에 `cancelled`, 새 render에 의한 중단에 `skipped/supersededRender`를
반환한다. 최종 phase 전환과 취소를 원자적으로 경합시켜 취소 성공 후 frame이 나오는 경우를
막는다. 완료 전환이 먼저 끝났으면 기존 `tooLate`를 반환한다. 취소된 요청도 completion
수신까지 기존 credit과 바이트 예약을 유지한다. FIFO 편집·저장 명령은 coalescing 대상에
추가하지 않는다.

`issue_536_native_region_cache`의 공개 callback 검사는 첫 타일 뒤의 중단, 부분·최종·off-page
출력 폐기, 완성 타일 재사용과 정상 재실행의 픽셀 일치를 확인한다. 기존 worker 검사는 FIFO
편집·저장, coalescing과 completion credit을 보호한다. FFI 검사는 취소가 성공하면 frame
버퍼가 반환되지 않고, 완료가 앞서면 `tooLate`와 완성 frame을 반환하는 경합 결과를 확인한다.
실제 worker를 특정 타일 경계에 강제로 멈추는 동시성 hook은 추가하지 않는다. 따라서 최신
render·close가 실행 중 타일과 경합하는 모든 순서를 결정적으로 재현했다고 주장하지 않는다.
Unit test support, 개수 한계와 integration target 정책은 변경하지 않는다.
