# 구형 OLE 문자열·첫 다단 원점 회귀 근거

2026-10-20 검증. 이 증거의 범위는 **문자열 정본 선택, 저장 첫 줄의 시작 밴드와 본문·이월 OLE 원점**이다.
차트 내부의 축·막대·범례 차이는 보정 중이며 이 자료로 차트 전체의 일치 완료를 선언하지 않는다.

## 독립 기대값과 검출 경로

- `ole_chart_unicode_copy.rs`: 원본의 제목 ANSI 사본만 `연금`에서 `기금`으로 바꾸고 UTF-16·자료·저장 캐시는 보존했다.
  한컴 Print는 원래 UTF-16 제목을 출력했다. 원본 Print와 변경 입력 Print는 모든 글자 위치와 도형이 같다.
  공개 자료 파서의 제목과 실제 차트 SVG가 이 정본을 공유하는지 검사한다. 수정 전 실제 라이브러리에서 제목 불일치로 FAIL,
  수정 후 정식 Cargo/nextest에서 PASS했다. helper의 통과만으로 판단하지 않는다.
- `stored_first_column_band_ownership.rs`: 원본에 있는 용지 위·머리말 여백, 첫 저장 줄 vpos,
  OLE 세로 오프셋·바깥여백과 최종 OLE 원점의 관계를 검사한다. 본문은 자기 저장 줄 vpos·기준선과 실제 glyph y를 대조한다.
  원본 양형식과 시작 밴드가 없는 정상 분리본 양형식을 함께 실행한다. 문서별 절대 px 좌표는 고정하지 않는다.
- 원본의 첫 줄 vpos는 283HU이고 다음 줄은 1000HU 높이와 600HU 간격으로 이어진다.
  본문과 다음 단 OLE가 이 밴드를 한 번 소비하고 PAPER 고정 개체는 이동하지 않는지 직접 판독했다.

## 전체 페이지 Native/fresh WASM TSV

모든 입력과 PDF는 1쪽이며 export 메타데이터와 독립 PDF 쪽수를 대조했다.
판정값은 색상 경계 보정 전 `silhouette_raw_match_percent`다. 표의 값은 평균이 아닌 전쪽 최저값이다.
TSV는 실행 결과를 바이트 그대로 복사했다. TSV 안의 로컬 산출물 경로는 아래 입력/PDF 해시와 대응한다.

| 입력 | Native 최저 | fresh WASM 최저 | Native TSV | WASM TSV |
|---|---:|---:|---|---|
| original-hwp | 98.59109% | 98.59437% | [원문](native-original-hwp-silhouette.tsv) | [원문](wasm-original-hwp-silhouette.tsv) |
| original-hwpx | 98.80042% | 98.79404% | [원문](native-original-hwpx-silhouette.tsv) | [원문](wasm-original-hwpx-silhouette.tsv) |
| isolated-hwp | 99.96037% | 99.72135% | [원문](native-isolated-hwp-silhouette.tsv) | [원문](wasm-isolated-hwp-silhouette.tsv) |
| isolated-hwpx | 99.96037% | 99.72135% | [원문](native-isolated-hwpx-silhouette.tsv) | [원문](wasm-isolated-hwpx-silhouette.tsv) |
| title-ansi-only | 98.80042% | 98.79404% | [원문](native-title-ansi-only-silhouette.tsv) | [원문](wasm-title-ansi-only-silhouette.tsv) |

검증 기준 `upstream/devel`은 `efc52afb738bd408ac821c4e156101b8d26e5cbd`로 고정했다.
그 기준과 런타임이 같은 `d7d4d2ea6ed427f2f3bc8b59a936a5f99aca852b` 생산자로 같은 입력·PDF·DPI·글꼴 조건을 비교했으며 0.1%p 이상 하락은 없다.
시각 비교는 96DPI, rhwp Print 프로필이다. `--compat 2022`는 rhwp의 호환 프로필이고 변환 한컴의 제품 연도가 아니다.
분리 저장본은 `--compat 2024`를 썼다.

## 입력·PDF 원문과 출처

한컴 2020 MCP Print의 출력 모드는 `hancom2020_pdf_driver_one_up`, `pdf_print_method=0`,
실제 제품은 11.0.0.9136이다. Save As PDF로 만들지 않았다.
정상 분리본은 사용자가 한컴 2024에서 수동 Print한 기존 PDF를 재사용한다.

| 입력 | 입력 SHA-256 | 기준 PDF | PDF SHA-256 |
|---|---|---|---|
| [original-hwp](../../../../samples/143E433F503322BD33.hwp) | `1bcdc7a2de38680cdb715556f40c8477f8242131d28de949b09e945022bf35fa` | [Print PDF](original-hwp-2020.pdf) | `5bb1416b4850e9e1d4b9abaec3c68dd8322a7480a1c0f98d1b892a71ff44f18b` |
| [original-hwpx](../../../../samples/hwpx/143E433F503322BD33.hwpx) | `1ee1e9045be4334fb9f940d7226cfda2884df9fecac6fa7fca76fb81c08a4a81` | [Print PDF](original-hwpx-2020.pdf) | `3eedafdf07e534af434c6dff58e6aefea0c2b1abe9d286419b410328e6aee44d` |
| [isolated-hwp](../../../../samples/hwpx/143E433F503322BD33-ole.hwp) | `9f7975a341f9bf0be89e3d6dc1385b59de5e61e079eeb0048db0090c932ad14c` | [Print PDF](../../../../pdf/143E433F503322BD33-ole-2024.pdf) | `584902f845c4b0d88d4e827a8a7dde23bf3ee3d92bd843f4c1f925b7a7498b20` |
| [isolated-hwpx](../../../../samples/hwpx/143E433F503322BD33-ole.hwpx) | `83815ff764b8ab40e6e232e57bc20b6dcb6f0b2c824f7673148fe63d965e1255` | [Print PDF](../../../../pdf/143E433F503322BD33-ole-2024.pdf) | `584902f845c4b0d88d4e827a8a7dde23bf3ee3d92bd843f4c1f925b7a7498b20` |
| [title-ansi-only](ansi-copy-only.hwpx) | `c94a78437329110a9ae07d5f2900775ef2c21da93e4723178ea362f701f60a5e` | [Print PDF](ansi-copy-only-2020.pdf) | `84a927f68c04910473fa31b91de54af8e501e2908f4f5fb980906ddcb163b012` |

변경 입력은 원본 ZIP의 `BinData/ole3.ole` 안 `Contents`에서 같은 길이의 ANSI 제목 사본만 치환했다.
UTF-16 사본과 다른 ZIP entry는 원본과 동일하다. 테스트도 원본에서 같은 변경을 수행하므로 수동 캐시 좌표를 기대값으로 사용하지 않는다.
`ansi-copy-only.hwpx`와 그 Print PDF를 함께 보존한다.

## 생산자와 소스 바이트

- Native 바이너리 SHA-256: `a2c0d6d929374a06e914657284843376dc402ce6db35c6a8e26143baac0db04f`
- fresh WASM SHA-256: `b759d6eae152a46171f078451016f18c3245c4ddeac6e72c586284d82dbd1e05`
- WASM wrapper JS SHA-256: `b74d131d6b401b8a442c1f93e2149d9aeb1b19fe6b3be8cbe5dadfc9fb774d71`
- 두 빌드는 같은 동결 소스를 사용했고 root web package와 Studio public 산출물의 SHA 일치도 확인했다.
- 빌드 기준 HEAD는 `6b70d40197476fc6efd45c78b810fc2b828e652d`이며 아래 미커밋 보정 파일을 포함해 생산했다.
  아래 해시는 실제 빌드한 원문이며, 마지막 서식 정리 한 곳의 차이는 아래에 별도로 기록한다. 테스트·기준 PDF만으로 소스 동일성을 대신하지 않는다.

| 제품 소스 | SHA-256 |
|---|---|
| `src/ole_chart/grid.rs` | `da2c89ff467d5366f0df8c02f93408e84b73cc94b69ed6db552b2c2c7edbfdc6` |
| `src/ole_chart/legacy_column_renderer.rs` | `881d7e3e8279bca52ca562c716c182c2581fe6b8bdb3f77f4ce862f67d1ed1ee` |
| `src/ole_chart/legacy_legend_layout.rs` | `85bbab3ee4e3dc8cec254827f2a01cc27561d2e15ab7d4cada873fe563df85c1` |
| `src/ole_chart/legacy_presentation.rs` | `3fa84ac6de10c251ac959d1d3022cbffcdee86f9876238f561e98e6c9590642b` |
| `src/ole_chart/legacy_string.rs` | `5eb75a324e639729698aa3610a7d34cc8c08be11961dea99d9bc0ac89043542c` |
| `src/ole_chart/mod.rs` | `fa17d49401cd6a726b948819ea1c9f7e4b3b20c5e2938150ea8b2f62497ff242` |
| `src/ole_chart/parser.rs` | `7875cbcf6b89aeda19e93235dfe00544653d0d091205bc7c8c341d6bfd602ebf` |
| `src/renderer/layout.rs` | `91246d44798c36131ce7184b0a7212b3fdce1d5d059ecb8d11d04f90cf9ba566` |
| `src/renderer/stored_page_origin.rs` | `50595e07153bae5b25e9fdc92f176360f945e14b515c5d84a7f6001cfa274f30` |
| `src/renderer/stored_page_origin/initial_column_band.rs` | `8e66e34a856b78b211f8618846f7c25615e89e6d75e5236688cc8fc9f0646925` |
| `src/renderer/typeset/section.rs` | `d146fc405d1e515811e1b508175353095aafaea592c7f4d0ba6f3fc2a59ea1bf` |
| `src/renderer/typeset/state/commands.rs` | `e9d9998a21241b8e852c26a62fb44dd2125e293b54e83af7b93ceb68e52d075c` |
| `src/renderer/typeset/state/data.rs` | `450973ee43904ae8a5179d8318e21290a14e8db90174001f05060685a9831fc0` |
| `src/renderer/typeset/state/transition.rs` | `dc2138a2553374458681a28d403132ff8600bef9774a659505b8d2293540f13a` |

실행 로그·중간 JSON·임시 스크립트는 ignored `output/pr-review/semanticist21-20261019/`에만 보관한다.
최종 PR에는 전체 필수 검증을 마친 head의 대표 review·overlay PNG를 별도로 연결한다.

### 빌드 후 서식 정리의 동일성

`src/renderer/layout.rs`의 `initial_column_band::for_paint` 호출만 인자 7개를 한 줄씩 배치했다.
함수 이름·인자 순서·표현식은 같고 파일 전체의 비공백 문자도 동일하다. 다른 생산자 소스 13개는 원문 바이트가 같다.
- 실제 빌드 원문 SHA-256: `91246d44798c36131ce7184b0a7212b3fdce1d5d059ecb8d11d04f90cf9ba566`
- 서식 정리 후 SHA-256: `b94b6eea7e8109634a07a2dbbb70dc2f9665402d65a74dcd4c5ce593fb5b22a8`

이 단계의 증거로 최종 통합 head의 전체 검증을 대신하지 않는다. 내부 차트 보정 뒤 최종 생산자·전체 필수 검증을 다시 고정한다.
