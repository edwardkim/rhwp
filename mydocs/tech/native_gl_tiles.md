---
kind: reference
status: active
canonical: mydocs/tech/native_gl_tiles.md
last_verified: 2026-11-07
---

# Native GL page tile와 자원 admission

[GL presenter](native_gl_presenter.md)는 GPU에 그린 page tile을 재사용한다. cache hit는
실제 GPU Surface의 pixels를 viewport scratch에 복사하며 Core/조판이나 page paint replay를
다시 실행하지 않는다. visible page의 새 tile이 필요할 때만 page snapshot을 strict replay한다.
viewport scratch를 모두 준비한 뒤 host FBO에 게시하는 기존 경계는 유지한다.

## key와 좌표

tile key는 context identity/device generation, document ID/revision/font generation과
fingerprint, page/profile, 정확한 raster scale, page raster의 정수 region, subpixel phase,
opaque page background를 포함한다. **view generation은 포함하지 않는다.** frame을
게시할 때는 P89의 전체 source/view/device stamp를 다시 검사하므로 cache hit가 stale
frame 게시를 허용하지 않는다. 원래 snapshot handle은 다른 view에서 재사용할 수 있다.

page 좌표를 매 tile의 작은 target에 다른 CTM으로 직접 replay하지 않는다. miss가 나면
제한된 canonical full-page GPU target에 먼저 strict replay하고 GPU copy로 별도 tile
Surface를 만든다. tile은 이 full-page texture의 단순 subset view가 아니다. 각 tile의
실제 RGBA payload가 별도 target reservation을 가진다. canonical 작업 target은 복사·
submit이 끝나면 해제되며 tile 뒤에 숨은 full-page backing으로 보유하지 않는다.

origin은 기존 Skia replay에 전달되는 실제 f32 값에서 `floor(origin)` 정수 이동과
`origin - floor(origin)` phase로 나눈다. canonical replay는 그 phase에서 실행하고,
각 tile은 정수 이동+region origin으로 viewport에 복사한다. 음수 origin도 floor 규칙을
사용한다. phase나 raster scale을 cache hit를 늘리기 위해 임의 반올림하지 않는다.
원래 f64 raster scale bits도 key에 보존한다. 정수 scroll에서 겹치는 page regions는
같은 key로 재사용되고 phase/scale/profile/문서·폰트가 달라지면 분리된다.

page `backgroundColor`의 `Some` COLORREF는 기존 replay가 alpha 1로 만드는 불투명
RGB다. `None`이거나 viewport의 ARGB `clearColor` alpha가 255가 아니면 기존 visible
strict replay를 사용한다. canonical page가 dimension/pixel/byte 한도를 넘거나 visible
tile 집합이 cache 한도를 넘을 때도 기존 viewport 경로를 사용한다. 높은 zoom 때문에
canonical page가 커졌다는 이유만으로 허용되던 viewport render 전체를 거절하지 않는다.

canonical strict resource 오류는 cache staging을 버리고 기존 visible strict replay를
시도한다. full-page 작업이 viewport 밖의 자원을 더 보았다는 이유로 원래 표시 경로에
새 실패를 만들지 않는다. visible replay도 실패하면 기존 `documentError` 등을 반환하고
이전 host FBO를 보존한다. 실제 GL allocation/device/submit 실패는 GPU 오류로 전파한다.
불완전한 page나 tile은 cache에 게시하지 않는다.

## native target과 renderer 한도

config에는 다음을 추가한다. 이전 P90 JSON config에 이 필드를 쓰지 않아도 기본값이
적용된다. `maxCachedTiles: 0` 또는 `maxTileBytes: 0`은 tile retention을 끄며 비교용
P90 viewport 경로로도 사용할 수 있다.

| 필드 | 기본값 / 공개 상한 |
| --- | --- |
| `tileSize` | 기본 512, 128·256·512·1,024 허용 |
| `maxCachedTiles` | 64개 |
| `maxTileBytes` | 64 MiB |
| `maxManagedTargets` | 67개 |
| `maxManagedTargetBytes` | 192 MiB |

scratch pool, 현재 frame scratch, canonical 작업 target, 준비 중인 새 tile과 cache의
GPU tile 모두 같은 RAII count/byte admission에 참여한다. full-page+missing tile의
작업량을 먼저 함께 확인하고 개별 reservation을 확보한 뒤 실제 GPU target을 만든다.
allocation/replay/submit/source 오류가 나면 해당 임시 reservation도 반환한다. 외부
host FBO는 호스트 소유이며 기존 target dimension/pixel/64 MiB geometry 한도를 적용한다.
native가 그 FBO 메모리를 할당하거나 전역 registry에 숨겨 보유하지 않는다.

tile을 그리는 현재 frame은 `Rc`로 Surface와 reservation을 submit까지 pin한다. LRU는
이 pin이 없는 owner-thread 자원만 matching current context에서 제거한다. 한 frame의
이전 page가 tile을 사용 중이면 뒤 page의 miss를 위해 그 tile을 빼앗지 않는다. admission이
부족하면 뒤 page는 기존 visible replay로 처리한다. 다음 frame에서 idle이 된 tile은
제거할 수 있다. idle scratch pool도 공통 target 한도를 맞추는 데 필요한 만큼 회수한다.

실패한 canonical/scratch의 일부 paint도 보이지 않는 작업 target에 cleanup submit한 뒤
관련 reservation/pin을 반환한다. 실패한 Skia 미제출 graph가 이미 반환한 tile backing을
숨겨 계속 보유하게 두지 않는다. cleanup의 GPU 실패는 abandon 후 GPU 오류로 전파한다.
이 submit도 GPU 실행 완료나 외부 consumer fence의 대체물은 아니다.

renderer의 명시적 per-face font-copy 추정 합계는 계속 64 MiB, renderer 수는 기본
2개/최대 8개다. renderer별 picture/glyph/image cache의 **합계** 추정량은 8 MiB
이하이며 세 종류에 균등 분배한다. renderer cache miss는 같은 frozen font bytes로
생성하고 새 font 객체를 만들기 전에 이전 renderer를 필요한 만큼 제거한다. 실제
보유 cache의 합계 추정값도 `rendererCacheEstimatedBytes`로 보고한다.

## 진단과 반환

`info.tileCache`는 다음의 실제 관찰값을 포함한다.

- `cachedTiles/cachedBytes`, key lookup의 `hits/misses`
- idle 제거의 `evictions/evictedBytes`
- `canonicalReplays/canonicalFailures`
- `viewportFallbacks`, `pinnedAdmissionFallbacks`, `targetAdmissionFallbacks`

`info.managedTargets`는 현재 reservation의 `targets/estimatedBytes`, owner 수명 동안
최대로 admission한 `peakTargets/peakEstimatedBytes`, 실제 reservation 거절 수를 보고한다.
미리 예약한 뒤 GPU allocation이 실패한 작업도 peak admission에는 포함될 수 있다.
동기 render가 끝난 뒤에는 canonical/pending tile reservation이 남지 않고 현재 native
target 계상은 scratch pool과 cached tile 합계다. frame별 foreground hit/miss를 구하려면
같은 owner의 연속 info counter 차이를 사용할 수 있다.

`gpuGl.tileCache/clearCaches` capability는 이 backend의 API 존재를 알린다. owner의
현재 cache 설정은 `info.config`로 확인한다. matching context를 current로 만든 뒤
다음 요청으로 idle GPU/cache 자원을 모두 비운다. source와 snapshot handle은 유지된다.

```json
{"op":"clearCaches","contextId":1,"deviceGeneration":1}
```

Python wrapper는 `presenter.clear_caches()`를 제공한다. current 없는 clear는 기존 자원을
바꾸지 않고 거절한다. 정상 release도 cache를 먼저 비우고, context loss/Drop은 먼저
Skia context를 abandon한 뒤 native tile Surface가 잘못된 GL context를 호출하지 않게 한다.
새 device/context의 presenter에는 별도 cache/budget이 생겨 이전 tile을 찾을 수 없다.

이 bytes는 **native가 관리하는 논리 RGBA target/font/cache payload**다. driver의 실제
VRAM·stride/alignment·metadata·working image/font decode·deferred GPU references나 OS font
manager의 전체 메모리가 아니다. Skia `resourceCacheBytes`는 purgeable cache에 요청하는
한도이며 pinned target 때문에 `resourceCacheBytes` 관찰값이 그 요청보다 커질 수 있다.
관찰된 GPU cache usage에는 위 managed target도 겹쳐 포함될 수 있어 둘을 정확한 VRAM
합계처럼 더하지 않는다. hard admission은 명시적으로 소유/예약한 target count·RGBA
추정량과 renderer/font/cache 계약에 적용한다.

whole-frame CPU readback은 추가하지 않는다. 같은 GL queue에서 submit한 draw의 순서와
외부 context/Flutter consumer sampling 완료는 다른 경계다. 새 GPU consumer lease나
외부 texture/fence capability를 광고하지 않는다.

current 없는 abandon으로 logical target 수가 0이 되었다고 old share-group의 driver
이름까지 해제되었다고 주장하지 않는다. live+orphan process context credit과 정상
retire/recreate, 독립 CPU batch 품질 비교는 [회복 계약](native_gl_recovery.md)을 따른다.

## 관계 검사

`tests/cases/issue_536_native_gpu_tiles.rs`는 공개 ceiling/overflow 거절, 최소 framebuffer
budget과 이전 JSON config 호환을 검사한다. 실제 GL 경로는 다음 probe로 검사한다.

```sh
QT_QPA_PLATFORM=xcb xvfb-run -a python3 bindings/Native/tests/native_gl_tile_contract.py /path/to/library --require-gl
```

probe는 tile을 끈 presenter의 frame과 GPU tile presenter의 pixels를 같은 실제 FBO에서
관계 비교한다. 겹치는 page 배치, 음수·fractional origin, scroll hit, phase miss,
transparent fallback을 포함한다. frame이 pin한 tile의 admission fallback과 다음 frame의
idle eviction bytes, 모든 canonical/scratch/pin reservation의 상한, strict 실패 후
host FBO 보존, current 없는 cache clear의 불변성, 실제 GL context 교체 뒤 cache 분리도
검사한다. readback은 호스트 검증으로만 수행한다. 이 합성 관계 검사는 전체 문서의
한컴 시각 일치율·실물 좌표나 지속 hardware 성능을 인증하지 않는다.

direct viewport replay와 canonical page replay의 비교에만 RGB 채널당 최대 1의 RGBA8
차이를 허용하며 alpha는 정확히 같아야 한다. 실제 Mesa 22.3.6 llvmpipe GL 검사에서
256×192 viewport와 397×562 canonical page의 차이는 작은 그림의 보간과 파란 outline의
antialias 강도에 나타났다. 최초 frame은 RGB 285채널이 최대 1만큼 달랐고 alpha 차이는
없었다. 별도 진단에서 GPU backing texture를 직접 읽어 canonical→각 tile→viewport
scratch→host FBO 복사는 바이트 단위로 일치함을 확인했다. 이 차이를 GPU 복사의 색상
양자화나 hardware 성능 검증으로 해석하지 않는다. 다른 크기의 raster target으로
replay할 때 관찰된 antialias·보간 강도 차이를 한정해서 검사한다.

이 허용값은 좌표나 경계를 느슨하게 비교하는 기준이 아니다. probe는 같은 그림의 PNG를
동일 크기의 투명 RGBA로 교체한 대조군도 실제 GL로 그린다. 음수·fractional origin과
겹친 page에서 파란 그림 outline의 **픽셀 집합**이 정확히 일치해야 하며, outline 이외의
표 테두리·종이·viewport 배경·clip pixels는 모든 RGBA byte가 같아야 한다. 투명 그림
대조군에서도 outline의 antialias 강도에만 위 RGB 1 차이를 허용한다. 정수 scroll과 음수
이동으로 재사용한 겹침 영역은 tile seam을 포함해 정확히 비교한다. 캐시의 cold/warm과
각 frame의 재사용 반복, transparent/partial-alpha·framebuffer-only fallback, 잘못된 요청과
strict 실패 후 기존 host FBO 보존에도 허용값을 적용하지 않는다. 출력의
`directTileComparisons`는 각 비교의 RGB 차이와 정확한 geometry 검사 여부를 기록하며,
추가 readback은 `hostVerificationReadbacks`에만 포함한다.
