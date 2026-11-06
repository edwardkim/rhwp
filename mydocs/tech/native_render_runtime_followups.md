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
