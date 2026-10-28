# Nimbus Roman 실제 프로그램 전진폭

`source.json`은 URW Base35 Nimbus Roman 네 정체의 파일 SHA-256·name 레코드·UPEM·cmap 범위를
확인한 출처다. `src/renderer/font_metric_nimbus_roman.rs`는 이 프로그램에서 읽은 Unicode별
정수 hmtx 전진폭만 보존한다. 글꼴 bytes·윤곽·문서별 장평/자간 보정값은 포함하지 않는다.

```bash
venv/bin/python scripts/extract_nimbus_roman_metrics.py \
  --font-dir /usr/share/fonts/opentype/urw-base35
```

이 경로는 검증 호스트의 설치 예시다. 실행 전 해당 네 파일의 존재·source JSON 해시를 확인한다.
같은 파일로 생성기를 두 번 실행하면 Rust 데이터와 JSON이 같아야 한다. family·PostScript 이름과
family+subfamily의 실제 정체 이름을 함께 조회하며 다른 family를 이름 유사성으로 대신하지 않는다.

원본 프로그램은 [Artifex URW Base35](https://github.com/ArtifexSoftware/urw-base35-fonts)에서
관리하며 [기존 PDF 포함 고지](../../../pdf/issue8174/fonts/URW-FONT-NOTICE.txt)를 보존한다.
이 메트릭은 프로그램 설치·실제 paint 공급의 증거가 아니다. 독립 CFF PDF와 Native/fresh WASM의
원점·간격·윤곽 직접 판독은 [T03 기록](../../../mydocs/working/task_m100_8174_t03_font_fidelity.md)을 따른다.

배포 archive에는 같은 디렉터리의 `URW-FONT-NOTICE.txt`를 함께 포함한다. 저장소의
검증용 글꼴·PDF는 포함하지 않는다.
