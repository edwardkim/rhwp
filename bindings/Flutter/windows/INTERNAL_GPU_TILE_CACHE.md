# P124 D3D 내부 GPU 타일 캐시

Native `NativeD3dPresenter`가 Skia GPU 표면에 그린 문서 타일을 다음 프레임에서 재사용합니다.
Flutter에 등록한 두 타일의 개수나 외부 DXGI 프레임의 소유권을 바꾸지 않습니다. 각 producer의
캐시는 해당 presenter와 D3D device generation에 한정됩니다.

## 렌더링과 재사용

공통 `gpu_tiles` 정책이 문서·revision·글꼴 generation과 fingerprint·페이지·화면 출력 옵션·
device generation·실제 DPR·scale·정확한 f32 소수 픽셀 phase·배경색을 키에 포함합니다.
정수 스크롤은 같은 타일을 다른 위치에 합성하고, 소수 phase나 콘텐츠가 바뀌면 cache miss가
됩니다. `viewGeneration`은 키에서 제외하지만 매 render의 snapshot borrow, cache 사용,
외부 target 복사 직전과 producer fence 이후에는 현재 full stamp를 다시 확인합니다.

처음 필요한 타일은 페이지 전체를 임시 canonical GPU 표면에 strict replay한 뒤 각 타일의
독립 GPU 표면으로 복사합니다. 캐시는 canonical 표면의 부분 image를 보관하지 않습니다.
모든 strict replay·타일 GPU 복사와 입력 유효성 검사가 성공한 뒤 캐시에 게시합니다.

이후 cache hit는 타일을 viewport scratch에 GPU 합성합니다. 기존과 동일하게 완성된 scratch를
외부 BGRA target으로 한 번 복사하고 D3D12 queue fence를 확인합니다. `resourceCommon`과
`producerGpuComplete`의 기존 의미를 유지하며, 성공에도 `consumerGpuComplete`는 `false`입니다.
이 캐시로 Flutter의 소비 완료나 외부 프레임의 재사용 가능 여부를 추론하지 않습니다.

배경이 투명하거나 canonical 크기·타일 수·총 예산이 허용 범위를 넘으면 기존 viewport strict
replay를 사용합니다. 전체 페이지 staging 중 생긴 리소스 오류도 GPU 작업을 정상 drain한 경우에만
viewport 경로로 돌아갑니다. 최종 화면을 CPU로 읽는 경로는 추가하지 않습니다.

## 메모리와 수명

새 설정은 기존 JSON config와 호환되는 기본값을 가집니다.

| 설정 | 기본값 | 허용 상한 |
| --- | ---: | ---: |
| `tileSize` | 512 | 128·256·512·1024 중 하나 |
| `maxCachedTiles` | 64 | 64 |
| `maxTileBytes` | 16 MiB | 64 MiB |
| `maxManagedTargets` | 66 | 66 |
| `maxManagedTargetBytes` | 48 MiB | 96 MiB |

기존 target당 4096 차원·4 Mi pixels·16 MiB 상한은 유지합니다. 전체 managed 예산은 scratch,
canonical, 준비 중인 타일, 캐시 항목, 현재 프레임이 pin한 항목의 실제 수명을 함께 계산합니다.
하나의 표면을 여러 곳에서 참조해도 예약은 한 번이며 마지막 참조가 사라질 때 반환합니다.
추정 바이트는 BGRA 픽셀 수 기준이며 driver VRAM·heap 정렬·프로세스 RSS의 상한은 아닙니다.

LRU는 다른 프레임 작업이 pin하지 않은 항목만 제거합니다. 오래된 revision 항목도 새 키로
재사용되지 않고 이 예산 안에서 제거됩니다. admission이 불가능하면 cache 크기를 늘리지 않고
viewport replay로 돌아갑니다. `maxCachedTiles: 0` 또는 `maxTileBytes: 0`이면 캐시를 끕니다.

GPU 작업을 기록하기 전에 scratch·canonical·타일 참조와 immutable snapshot 입력을 owner에
보존합니다. 중간 tile completion이 확인되어도 최종 scratch 작업을 drain하기 전에는 프레임
pin을 반환하지 않습니다. 오류 경로는 pending flag와 관계없이 scratch drain을 수행합니다.
panic이나 확인 불가능한 GPU 완료에서는 기존 process context credit와 함께 캐시·작업 표면·입력·
예약을 quarantine합니다. 취소나 owner 종료만으로 예약을 환급하지 않습니다.

`rhwp_d3d_presenter_request`의 `{"op":"info"}`는 `managedTargets`, `tileCache`,
`workingTargets`, `frameTilePins`를 반환합니다. hit/miss·canonical replay·eviction·fallback
계수는 실행 관측값이며 실제 하드웨어 가속·성능 향상·Flutter 소비 완료의 증거가 아닙니다.
정상 프레임 종료 후 working target과 frame pin은 0이고 캐시와 재사용 scratch만 남습니다.
정상 release·abandon은 이 표면들을 반환하며, 새 presenter는 비어 있는 캐시로 시작합니다.

## 검증 범위

공통 GPU 정책 검사는 key·phase·DPR·revision 분리, LRU·pin·총 예산을 검사합니다.
실제 producer가 사용하는 `gpu_tiles/frame_resources.rs`는 Rust 표준 라이브러리만으로 검사할 수
있으며 정상 drain, 반복 프레임, panic unwind, quarantine 이후 해제 거절을 다룹니다.
자원 보존을 깨뜨리는 음성 대조는 반드시 실패해야 합니다. D3D config 단위 검사는 Windows의
`native-skia-d3d` 빌드에서 기존 JSON 호환·직접 replay 설정·상한 거절을 검사합니다.

실제 Windows에서는 같은 문서·viewport로 캐시 활성/비활성 결과를 독립 CPU 기준과 비교하고,
정수 스크롤에서 hit 증가, phase·DPR·revision 변경에서 miss 증가, 작은 예산에서 fallback,
종료·재생성 후 빈 캐시를 확인해야 합니다. 기존 [두 타일 소비 검증](../WINDOWS_TILE_QUALIFICATION.md)의
실제 Flutter/Win32 캡처와 producer fence·자원 종료 검사도 유지합니다.
Linux의 정책·수명 회귀는 Windows SDK 컴파일이나 실제 D3D/Flutter 실행을 뜻하지 않습니다.
실제 Windows 출력·물리 GPU 성능·최종 한컴 자료 검증은 별도로 남겨 둡니다.
