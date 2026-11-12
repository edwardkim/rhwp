# P125 Windows private producer target 재사용

Windows producer는 Native D3D12 출력과 D3D11의 최종 GPU 복사가 모두 끝난 private 입력 표면을
다음 프레임에 재사용합니다. Flutter가 여는 legacy D3D11 표면은 매번 새로 생성한 immutable
resource입니다. descriptor opening이나 CPU callback 종료를 GPU 완료로 해석하지 않습니다.
기존 `targetReuse: false`, `consumerGpuCompletionVerified: false`는 이 공개 표면의 계약입니다.
`privateTargetReuse: true`는 내부 producer 입력에만 적용됩니다.

## 완료와 소유권

private target은 `Reserved → NativePending → NativeComplete → CopyPending → Complete`
단계를 따릅니다. Native 응답의 full stamp·producer fence·COMMON 상태와 나머지 출력 메타데이터가
맞아야 Native 완료를 인정하고, bridge의 D3D11 event query가 실제 완료를 반환해야 복사 완료를
인정합니다. Native가 안전한 미게시 target 폐기를 증명했거나 소스가 변경되어 복사를 시작하지 않은
경우에는 Native 완료 뒤 명시적으로 copy를 생략할 수 있습니다.

producer마다 완료된 idle target을 하나만 보유합니다. device generation·물리 크기가 모두 같고
다른 작업이 참조하지 않을 때만 꺼냅니다. 크기나 device가 바뀌면 기존 idle resource를 반환합니다.
공개 frame bundle에서 private 참조를 분리하는 시점은 이 완료 검증 뒤이며, pool은 `Pixels` 전체나
그 프레임의 reservation을 보유하지 않습니다. 입력의 D3D12 resource·NT handle·D3D11 opened
resource·copy query만 재사용합니다.

Native 제출이나 D3D11 copy 완료가 불명확하면 pool 반환을 거부합니다. frame의 기존 quarantine이
private resource와 독립 byte reservation도 함께 보존합니다. close·취소·예외·geometry 변경으로
진행 중인 resource를 idle로 바꾸지 않습니다. producer 종료 시 pool은 확인된 idle 항목만 반환합니다.

## 같은 64 MiB 장부

기존 frame admission은 여전히 최대 네 개이며 하나의 프레임은 두 표면 합계 최대 16 MiB입니다.
행 정렬과 64 KiB 단위의 보수적인 allowance를 `S`라 하면 다음 순서로 계산합니다.

1. 새 작업은 기존처럼 `2S`를 예약합니다.
2. 처음 private 표면을 만들 때 이를 frame 예약 `S`와 독립 private 예약 `S`로 분할합니다.
   전체 예약 바이트는 변하지 않습니다. 실제 D3D12 allocation size가 private `S` 이내인지도
   `GetResourceAllocationInfo`로 검사합니다.
3. 이미 private 예약을 가진 표면을 재사용할 때는 새 frame의 중복 `S`만 제거합니다.
4. public creator 참조와 frame reservation이 반환돼도 idle private `S`는 계속 장부에 남습니다.
   private resource의 마지막 참조가 없어지고 resource가 파괴된 뒤에야 그 예약을 반환합니다.

두 종류의 예약 합계는 기존 process-wide 64 MiB를 넘지 않습니다. private 예약은 최대 네 개,
idle 항목은 기존 두 producer 슬롯에 맞춰 최대 두 개입니다. 별도 private pool 예산을 더하지
않으며, 작은 예산에서는 admission을 거절합니다. consumer 소유 참조·driver 내부 지연 할당·
driver bookkeeping·Native 내부 Skia 캐시의 메모리는 기존과 같이 이 bridge 장부 밖입니다.
Native 내부 캐시는 [P124의 별도 상한](INTERNAL_GPU_TILE_CACHE.md)을 따릅니다.

상태의 `reservedTargetBytes`는 항상 `frameReservedBytes + privateTargetBytes`입니다.
`frameReservedBytes`에는 아직 분할하지 않은 작업의 two-surface allowance도 들어갑니다.
`producerOwnedBytes`는 기존처럼 ownership-transfer mode의 frame 예약만 세므로 private 값과
분리해 읽어야 합니다. `privateTargets`, `idlePrivateTargets`, `idlePrivateTargetBytes`,
`privateTargetReuses`로 pool의 보유·재사용을 관측합니다. 정식 close 완료에서는 private 보유도 0입니다.
이 계수는 실제 consumer fence나 전체 VRAM의 정확한 상한을 뜻하지 않습니다.

## 검증

기존 portable Windows bridge suite에 실제 frame 장부와 private pool 상태 전이를 검사하는 회귀를
추가했습니다. frame 반환 후 idle 바이트 유지, 128회 연속 재사용, fresh public 참조 반환,
foreign credit·이중 분할 거절, Native/copy 미완료의 pool 거절, geometry/device 불일치, 마지막
resource 파괴 전 charge 유지, 네 frame·64 MiB·두 idle 상한을 다룹니다.
회계·phase·geometry 보호를 제거한 음성 대조는 실패해야 합니다.

Windows continuous 수집기는 공개 frame 바이트와 private 바이트의 합계를 검사하고, close 뒤
남아 있는 pool이나 감소한 재사용 계수를 거절합니다. 물리 크기가 같은 연속 publication에서는
재사용 계수가 증가해야 합니다. 이전 pool 미지원 영수증과 구분하며 새 private 계수를 일부만 제출한
영수증은 받아들이지 않습니다. 기존 두 타일 capture도 이 공통 예산 판정을 사용합니다.

Linux portable 검사는 D3D12/11 API나 Flutter Windows 실행의 증거가 아닙니다. 실제 Windows에서는
동일 geometry의 연속 publication에서 private reuse가 증가하고 공개 target은 계속 fresh인 것,
resize·device recreation·close의 반환과 실제 capture 결과를 확인해야 합니다. Windows SDK·물리
GPU 실행과 최종 한컴 출력 검증은 별도로 남겨 둡니다.
