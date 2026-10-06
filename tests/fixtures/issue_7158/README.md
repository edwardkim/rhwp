# Issue #7158 전체 그림 시각 검증 입력

- `156492236-regulatory-sandbox-full.hwpx`: 중소벤처기업부의 규제샌드박스 시행 3주년 보도참고자료 원본. [이슈 #7158](https://github.com/edwardkim/rhwp/issues/7158)과 [기존 자료 출처](../../../samples/issue4090/README.md)를 따른다.
- 기존 `samples/issue4090/156492236_규제샌드박스_min.hwpx`는 BinData/Preview를 스텁으로 바꾼 회귀 입력이다. 원본의 Contents XML과 바이트가 같지만 그림의 시각 검증을 대체하지 않는다.
- 이 파일은 원본 바이트를 보존하며, 기존 Git 추적 HWP/HWPX에 같은 SHA-256이 없는 것을 확인한 뒤 추가했다. 구조 회귀는 기존 최소본을 재사용하고 전체 사진 비교에는 이 입력을 사용한다.
- 기준은 기존 [한컴 PDF](../../../pdf/156492236-regulatory-sandbox-full-2020.pdf)다. 동일 PDF를 다시 복사하거나 변환하지 않았다.
- 원본 이름, SHA-256, byte 수는 [manifest](MANIFEST.json)에 기록한다.
