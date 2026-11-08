# P108 Linux consumer 실행 증거

[`evidence.json`](evidence.json)은 고정된 Flutter/GTK source에서 실행한 실제 X11 desktop
32회 표시와 자원 정리를 요약한다. `sourceRevision`은 실행에 사용한 코드이며, 이 증거 파일을
추가하는 후속 commit과 구분한다. 전체 실행 로그와 raw RGBA는 별도로 보존한다.

| 단계 | Native CPU | 실제 X11 desktop | 독립 4배 CPU reference |
| --- | --- | --- | --- |
| 첫 표시 | [CPU](cpu-01.png) | [화면](display-01.png) | [source](reference-01.png) |
| 편집 revision | [CPU](cpu-02.png) | [화면](display-02.png) | [source](reference-02.png) |
| 물리 target resize | [CPU](cpu-03.png) | [화면](display-03.png) | [source](reference-03.png) |
| 32번째 게시 | [CPU](cpu-32.png) | [화면](display-32.png) | [source](reference-32.png) |

PNG는 저장된 RGBA를 크기·색상 변경 없이 인코딩한 자료다. 원본 RGBA와 PNG의 SHA-256을
함께 기록한다. CPU/reference는 같은 immutable snapshot·full stamp를 독립 재생하며 reference의
target/placement만 확대한다. 화면 캡처는 GPU나 CPU의 expected 색상을 만드는 데 사용하지 않는다.

첫 편집 후에는 같은 revision에서 viewport 위치를 번갈아 바꾼다. 32개의 view generation과
이웃한 CPU 이미지의 변화를 확인하며, 문서를 32번 편집했다는 뜻은 아니다.
균일 내부 채널 오차 1·한 pixel 경계 지원·13방향 독립 source palette·유일한 zero-shift 정렬을
검사하고 실제 캡처에 대한 잘못된 색상·이동·빈 화면의 음성 대조도 기록한다.

실행 환경은 Xvfb와 Mesa llvmpipe 소프트웨어 GL이다. **최신 devel Rust를 재빌드한 결과가
아니다.** Native source `ac393935254caa9fd27c1b0c92346913d4f015af`로 선언된 기존 SO
`5c992d8ccd54c85a8ad74ea58a30be33051055681ee129c317ea622341a3e537`를 사용한다.
따라서 최신 devel의 Rust clip 변경, 하드웨어 GPU, Wayland, Windows/macOS consumer,
공개 consumer GPU fence나 최종 한컴 비교를 이 자료로 검증했다고 주장하지 않는다.

실행 도구와 비교 계약은 [Linux qualification 설명](../../../../../bindings/Flutter/LINUX_QUALIFICATION.md)에 있다.
