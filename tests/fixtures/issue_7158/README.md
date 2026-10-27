# Issue #7158 원본의 시각 회귀 이관

그림을 보존한 원본은 [#8236 재현 자산](../../../mydocs/pr/assets/issue_8236/regulatory_sandbox/README.md)으로 옮겼다. 바이트·SHA-256은 유지한다. 고정 devel과 후보의 전 17쪽 Native/fresh WASM 최저값이 각각 75% 전후여서 정상 출력 회귀로 등록하지 않는다.

기존 최소본은 parser·serializer가 공유하므로 `samples/issue4090/156492236_규제샌드박스_min.hwpx`에 보존한다. 원본과 Contents XML은 같지만 BinData/Preview 스텁은 원본 그림의 시각 증거를 대신하지 않는다. 렌더링 축의 자동 수집은 명시적으로 제외했다. [manifest](MANIFEST.json)와 이관 자산의 복원 원장을 따른다.
