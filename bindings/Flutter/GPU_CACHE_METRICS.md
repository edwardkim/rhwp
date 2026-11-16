# GPU 캐시 producer 진단과 비교

GL·Metal·D3D의 실제 Native `info`에서 읽은 캐시 hit/miss/eviction, 보관 타일 수·바이트,
managed target 수·추정 바이트를 같은 `producerDiagnostics` schema로 전달합니다. Metal의
`info.tiles`와 GL/D3D의 `info.tileCache`는 같은 공통 정책의 계수입니다. 플랫폼 adapter는
producer thread에서 Native 완료 뒤 값을 읽고, 현재 stamp의 프레임 게시와 함께 보관합니다.
status 조회가 Native presenter를 다른 thread에서 호출하지 않습니다.

```json
{
  "schemaVersion": 1,
  "backend": "openGlGanesh",
  "scope": "publishedFramePostRenderSnapshot",
  "stamp": {
    "documentId": 1, "revision": 0, "fontGeneration": 0,
    "viewGeneration": 1, "deviceGeneration": 1, "deviceScale": 1.0
  },
  "counters": {
    "cacheHits": 7, "cacheMisses": 4, "cacheEvictions": 1,
    "cachedTiles": 3, "cachedBytes": 196608,
    "managedTargets": 4, "managedBytes": 262144
  }
}
```

이는 **게시한 프레임의 렌더 직후 snapshot**입니다. hit/miss/eviction과 canonical replay·fallback은
presenter 수명 누적 계수이고 cached/managed 항목은 그때의 gauge입니다. 이후 memory-pressure
trim이 실행되어도 이 프레임의 측정값을 바꾸지 않습니다. 현재 trim 결과는 별도의 trim receipt로
확인해야 합니다. managed bytes는 Native가 예약한 BGRA 픽셀 추정치이며 Skia 내부 cache·driver
heap·소비자·전체 프로세스 VRAM을 합친 메모리 사용량이 아닙니다.

지원하지 않거나 형식·범위를 확인할 수 없는 값은 생략합니다. 실제로 측정한 0만 0으로 표시합니다.
두 타일은 각 프레임의 stamp와 backend가 모두 맞고 각 타일에 같은 계수가 있을 때만 합산합니다.
하나의 측정값만 있으면 viewport 전체 합계를 만들지 않습니다.

`NativeGpuTrace`는 controller의 현재 요청이 producer-ready로 확인된 때에만 이 sample을 받습니다.
다른 document/revision/font/view/device/DPR 결과와 닫힌 status를 거절하며 snapshot의 유효성·전체
tile receipt 확인을 통과한 기존 표시 절차를 유지합니다. optional trace가 없으면 Dart 계수 추출도
실행하지 않습니다. 진단 누락·오류는 프레임 수명과 게시를 실패시키지 않습니다.

## cold / warm / scroll / zoom 비교

같은 Native 바이너리·문서·backend·크기·DPR에서 다음 순서로 기존 참조 host를 실행합니다.
실제 제품 앱 UI는 추가하지 않습니다. 각 구간은 별도의 recorder 또는 잘라낸 trace JSON으로
보존하며 warm/scroll/zoom에서는 같은 presenter를 유지합니다.

1. cold: 새 presenter 또는 `targetBytes: 0` trim을 완료한 상태에서 첫 viewport를 요청합니다.
2. warm: 같은 viewport를 반복 요청하여 캐시 재사용을 관찰합니다.
3. scroll: 정수 픽셀 단위로 viewport를 움직여 동일 phase의 hit와 새 영역 miss를 확인합니다.
4. zoom: 배율 또는 소수 phase를 바꾸어 miss·canonical replay 증가를 확인합니다.

```bash
python3 -B tool/gpu_cache_benchmark.py \
  --cold /absolute/cold.json --warm /absolute/warm.json \
  --scroll /absolute/scroll.json --zoom /absolute/zoom.json \
  --output /absolute/cache-comparison.json
```

CLI는 각 구간의 `rhwp-gpu-producer-trace` JSON을 최대 8 MiB·512 events로 읽고 입력 해시와
표준 `rhwp-gpu-cache-benchmark` JSON을 생성합니다. GPU 작업이나 화면 캡처는 실행하지 않으며
cold/warm이라는 파일 label의 실제 수행 조건을 대신 검증하지 않습니다. 계수 차이는 같은 source·
device epoch의 연속된 두 개 이상 알려진 sample에서만 계산합니다. 중간 누락·계수 reset·presenter
교체는 unknown으로 남기며 감소값을 0으로 보정하지 않습니다. 단일 cold 프레임도 실제 누적 계수의 `lastCounters`를 보존하지만 구간 증가량은 만들지 않습니다.
gauge의 observed min/max와 마지막 게시 프레임 값을 구분합니다. trace ring에서 빠진 sample의 전체 수명 분포를 추측하지 않습니다.

기존 request→producer-ready 시간은 capture·RPC·Native producer·게시·status 확인을 포함합니다.
Native draw/replay/compose 구간별 시간은 현재 제공하지 않으므로 CLI도 생성하지 않습니다.
이 결과로 compositor 표시 시각·consumer GPU 완료·물리 GPU FPS나 성능 향상을 판정하지 않습니다.
실제 Apple/Windows SDK·GPU 실행과 한컴 전체 문서 대조는 별도 검증입니다.
