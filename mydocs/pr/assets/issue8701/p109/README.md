# P109 Windows consumer 실행 증거

[`evidence.json`](evidence.json)은 실제 Windows SDK로 plugin·Flutter 앱·owner를 빌드하고,
기본 `legacyImmutable` 3회와 opt-in `immutableOwnershipTransfer` 32회를 실제 Win32 desktop에서
캡처한 결과를 요약한다. [GitHub Actions 실행](https://github.com/seo-rii/rhwp/actions/runs/42455059831/job/128652215624)은
성공했다. 전체 receipt·로그·raw RGBA는 해당 실행의 artifact로 별도 보존한다.

실행 소스는 `ebd10e523d8f33b864791d3fdf893e00c29f87cd`다. P109 제출 준비 소스
`daf9e7d40aeffbda430d3c3d5631e4bf3e7a3453`의 Windows plugin·공통 Dart·owner·수집 도구·
Native header 42개는 실행 소스와 Git mode/blob이 같다. Windows checkout의 CRLF와 로컬 LF는
별도로 구분했으며, 실제 빌드에 복사된 소스 hash는 Windows 원본 소스 hash와 일치한다.
이 증거를 추가하는 후속 commit은 실행 코드를 변경하지 않는다.

입력은 [고정 실행 소스의 합성 HWPX](https://raw.githubusercontent.com/seo-rii/rhwp/ebd10e523d8f33b864791d3fdf893e00c29f87cd/bindings/Flutter/test/fixtures/desktop-consumer.hwpx)이며,
SHA-256은 `648dade82585fa2845b80ec510b5f55c67e8b6127688dd3748b9a2068d87a3d9`다.

| 연속 모드 단계 | Native CPU | 실제 Windows desktop | 독립 4배 CPU reference |
| --- | --- | --- | --- |
| 첫 표시 | [CPU](cpu-01.png) | [화면](display-01.png) | [source](reference-01.png) |
| 편집 revision | [CPU](cpu-02.png) | [화면](display-02.png) | [source](reference-02.png) |
| 물리 target resize | [CPU](cpu-03.png) | [화면](display-03.png) | [source](reference-03.png) |
| 32번째 게시 | [CPU](cpu-32.png) | [화면](display-32.png) | [source](reference-32.png) |

PNG 12장은 저장된 RGBA를 크기·색상·alpha 변경 없이 lossless 인코딩했고, 다시 읽은 픽셀이
원본과 같음을 검사했다. 원본 RGBA와 PNG SHA-256을 함께 기록한다. 대표 이미지를 직접 판독해
첫 표시, 편집 후 위치 변경, 320×240 → 352×256 resize와 후속 viewport 이동을 확인했다.
기본 모드 1·2·3회의 CPU/display/reference raw도 연속 모드의 해당 단계와 byte-identical하다.

CPU와 reference는 같은 immutable snapshot·full stamp를 독립 재생하며 reference의
target/placement만 4배 확대한다. 화면에서 expected 색상을 만들지 않는다. 기존 균일 내부
채널 오차 1·한 pixel 경계 지원·13방향 독립 source palette·유일한 zero-shift 정렬 판정을
사용한다. 각 게시는 0.1초 간격의 실제 캡처 두 장이 정확히 같아야 통과한다. 대표 7개 raw 비교
(기본 3개·연속 4개)를 다시 실행해 원래 receipt와 같은 비교 결과를 얻었다.

32회는 문서를 32번 편집했다는 뜻이 아니다. 첫 편집 뒤 같은 revision에서 resize하고 viewport
위치를 번갈아 바꾼다. 32개의 서로 다른 view generation과 이웃한 CPU hash의 변화를 확인했다.

연속 모드는 held stage에서 producer-owned target 최대 1개·1 MiB, descriptor ticket 최대 1개를
관측했다. 이는 전체 실행의 순간 최대치 측정이 아니다. 제한은 target 4개·64 MiB와 ticket 8개이며
consumer·driver의 별도 allocation을 포함하지 않는다. 종료에서는 `ownerFinished:true`,
`registered:false`, `unregistered:true`, `acknowledged:true`와 creator/import/ticket/reserved bytes
0, `handedOffFrames:32`, `descriptorOpenedNotifications:32`를 확인했다.

기본 모드는 종료 후에도 불변 target 3개·3 MiB를 process lifetime으로 유지한다
(`legacyRetainedFrames:3`, `outstandingFrames:3`, `reservedTargetBytes:3145728`).
두 모드의 Native source/snapshot/frame handle·lease 통계가 0이라는 사실은 이 기본 모드의
GPU target 보존과 구분한다. close acknowledgement와 descriptor callback은 consumer GPU fence가 아니다.

Native DLL은 이번 실행에서 재빌드하지 않았다. source
`9452099dc796ca5626f1c4b52396a9ca07dc445e`에서 `native-skia-d3d`로 성공한 DLL
`79dac58c30367d477e87c8c13f962f30a2d8f336a1be57016aac7428ca4c1895`를 사용했다.
seed 실행의 전체 SDK qualification은 실패했으며, 이번 실행에서는 새 plugin·Flutter 앱 빌드와
두 consumer 검증이 성공했다. Native build 입력 81,672개를 mode/type/blob/path로 비교한 manifest
`4c23b6eaca5a87cd7199a200905782885ddd531dbb086f0c5f9027dea45ee350`이 seed·실행·제출 준비
소스에서 동일함을 확인했다.

환경은 `windows-latest`와 고정 Flutter 3.47.9/engine
`89c39edfdfae2949f6806e4a315c0c08522238b7`이다. 실제 adapter 이름·종류는 기록하지 않았으며
`hardwareAccelerationVerified:false`를 유지한다. consumer GPU 완료, 동일 target 재사용,
consumer·driver 전체 메모리 상한, 별도 standalone Native capture나 최종 한컴 비교를 검증했다고
주장하지 않는다. 실행 도구와 계약은 [Windows qualification 설명](../../../../../bindings/Flutter/WINDOWS_CONTINUOUS_QUALIFICATION.md)에 있다.
