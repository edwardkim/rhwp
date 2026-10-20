# CMYK JPEG의 요청별 예약 제한

jpeg-decoder 0.3.2 기반이다. 상류 소스 hash·crate checksum·revision은 UPSTREAM.json,
원래 라이선스·manifest는 LICENSE-*와 Cargo.toml.orig에 보존한다. 비공개 local package
0.3.2-rhwp.1이며 publish=false다. 제한 PDF import의 CMYK 경로만 사용하고, 일반
registry decoder와 기존 image 소비자는 유지한다.

- `new_with_allocation_budget`으로 생성자 table·header보다 먼저 빌린 Sync 요청 context를
  연결한다. 크기 연산을 검사하고, Vec의 fallible reserve 전에 개별·누적 예산을 계상한다.
  교체·해제한 버퍼도 누적 비용에서 빼지 않는다.
- APP/COM metadata, 반복 EXIF/XMP/PSIR, ICC chunk, Huffman scratch/table, frame/scan
  collection, progressive/MCU/lossless plane과 row, interleaved 출력, 행별 upsampling이 대상이다.
- worker 출력 plane은 호출 스레드에서 예약한 뒤 소유권을 전달한다. Immediate·Rayon·
  std-thread의 선택·IDCT·색 변환 알고리즘은 유지한다. Immediate의 고정 4-slot container는
  배열로 두고 scope Box를 계상한다. Rayon 행별 할당 실패는 try_for_each, 미완료 progressive
  row 할당 실패는 scoped task iterator의 Result로 전달한다.
- 고정 크기 quantization Arc·worker Box·generic upsampler Box도 payload를 먼저 계상한다.
  이 작은 구조체는 stable Rust의 infallible Arc/Box 생성자를 유지한다. allocator overhead,
  스케줄러·thread/channel 저장소, 오류 문자열 및 process RSS 제한을 입증한 것은 아니다.
- `icc_profile()`의 legacy metadata 조립 accessor는 제한 PDF 소비자가 호출하지 않는다.
  이 accessor의 추가 예약은 이번 감사 범위 밖이다.
- Adobe 반전·색 변환·플랫폼별 IDCT 선택은 유지한다. 같은 의미의 정수 clamp/contains,
  사용처와 일치하는 SIMD/thread 모듈 cfg, 오래된 asmjs 표기 제거로 현재 Clippy에 대응한다.
  workspace rustfmt에 따른 상류 소스 서식 변경이 있다.

이 hook은 명시한 decoder 예약의 제한이다. 전체 PDF parser·CPU 격리·임의 reader 구현의
안전성을 뜻하지 않는다. 기본 Rayon 기능과 WASM의 기존 immediate 경로를 유지하며
backend 용량 축소 작업은 하지 않는다.
