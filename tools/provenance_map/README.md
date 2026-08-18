# provenance_map — 출처 표지 소비자 픽스처

기존 CLI `export-provenance-map` 만 사용한다. 새 명령을 추가하지 않는다.

```bash
python tools/provenance_map/fatten_provenance_map.py
python tools/provenance_map/test_fatten_provenance_map.py
```

단일 출처: `crates/rhwp-contracts/src/provenance.rs`.
이 폴더는 소비자 경계(금지 자리·모드 표본·작업 문서)를 고정한다.
