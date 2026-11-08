# PDF 가져오기 공동 검증 입력 (#7957–#7963)

작업지시자가 재현용으로 제공한 원본 PDF를 변경 없이 보존했다. 기존 동일 파일은 재사용한다.
원본 PDF 자체의 독립 Poppler 출력이 PDF 가져오기 paint의 기준이며, 원본 HWP 재조판과 구분한다.

| 이슈 | 저장소 경로 | 쪽수 | 바이트 | SHA-256 |
| --- | --- | ---: | ---: | --- |
| #7957 | `samples/issue7957/resource-limit.pdf` | 162 | 1920186 | `b5dd80e561c055c028b5297c7b5ad04c1c1de6de6b246282635b5915680e11e7` |
| #7958 | `pdf/pr7704/6778-square-2020.pdf` | 12 | 930679 | `d4c1fe7a8a3c025b2be3648ba6e2ad1123eff3dde008983edadcef7e8f7b6656` |
| #7959 | `samples/issue7959/bounded-import-path.pdf` | 4 | 595334 | `0def50f6b391130c745754b44de320e14a3fd1c88a2dc35dce09e2b3c3bb418d` |
| #7960 | `samples/issue7960/path-budget.pdf` | 107 | 3331810 | `1372d5a2e4edf596d8d622026049fe6980130103a93a5c317b20716c6e0a9428` |
| #7961 | `samples/issue7961/extraction.pdf` | 14 | 6130211 | `9c99058abf2d22571117ce58a123c32bbc3065756ff4e3d88d3ec5f034331bb3` |
| #7963 | `samples/issue7963/decoded-bytes.pdf` | 14 | 2194591 | `26acc6a382ec7c5732b5d2d89518d25531494722ccd128ce707288c82e87413a` |

출처: 작업지시자 제공, 각 이슈 본문의 원 파일명·해시와 대응한다. 원 다운로드 URL은 별도 확인되지 않았다.
16 MiB 경계 생성본은 `samples/issue7962/`를 따른다. 50 MiB 이상 PDF와 Windows 폰트는 커밋하지 않는다.
시각 검증은 Native Skia와 Docker WASM Canvas, 96 DPI, Print/compat 2022, 같은 공급 글꼴로 수행했다.
결과는 `mydocs/working/task_m100_7957_stage7.md`와 통합 단계 보고에 연결한다.
