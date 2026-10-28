---
kind: working
status: active
canonical: mydocs/working/task_m100_4055_stage2.md
last_verified: 2026-08-06
---

# #4055 Stage 2·3 — 변이 생성기와 중첩 CFB 재포장 (S2·S3 재료, S4)

- **Issue**: [#4055](https://github.com/edwardkim/rhwp/issues/4055)
- **결론**: **S4의 스트림 payload는 보존되지만, `mini_cfb`는 루트 CLSID를 잃는다.** 변종
  10개 조립 완료. S2·S3은 이후 전수 한컴 판정과 HWP 2020 MCP 독립 재현으로 닫혔다.

## 1. S4 — 중첩 CFB 재포장은 무손실인가

**답: 스트림 바이트에는 그렇지만, CFB 전체에는 아니다. 코퍼스 28종 전건.**

`nested_cfb_repack_preserves_every_stream` 이 코퍼스 전건에 대해
스트림 전수 열거 → `mini_cfb::build_cfb` 재포장 → 재열거 후 **스트림 집합과 각 스트림
바이트의 동일성**을 단언한다. 재포장본이 `parse_ole_container` 소비 경로도 그대로 탄다.
그러나 이 검사는 CFB 디렉터리 메타데이터까지 보존한다는 뜻이 아니다. Stage 4에서 확인한 대로
`mini_cfb::build_cfb`는 중첩 OLE Root Entry CLSID를 0으로 만들며, 한컴 호환 재포장에는
원본 CLSID를 전달·보존하는 API가 선행돼야 한다.

함께 확인한 것: **`parse_ole_container` 가 모르는 스트림은 코퍼스에 하나도 없다.**
그 4종(`Contents`·`OOXMLChartContents`·`\x02OlePres000`·`\x01Ole10Native`)이 전부다.
따라서 "4종만 뽑아 재포장하면 나머지가 소실된다"는 착수 전 우려는 **이 코퍼스에서는
현실화되지 않는다**. 다만 프로브는 전수 열거 방식을 쓰므로 코퍼스 밖 문서에도 안전하다.

### 이 과정에서 잡은 함정

Windows 에서 `cfb` crate 는 경로를 **구분자를 섞어** 돌려준다.

```text
"/BinData\\BIN0001.OLE"
"/BodyText\\Section0"
"/\u{5}HwpSummaryInformation"
```

`mini_cfb::build_entries` 는 `path.trim_start_matches('/').split('/')` 로만 쪼갠다
(`src/serializer/mini_cfb.rs:330`). 정규화하지 않고 넘기면 스토리지가 사라지고
`BinData\BIN0001.OLE` 라는 **이름에 역슬래시가 든 루트 스트림 하나**로 뭉개진다.
조용히 깨지는 종류라, 재포장을 프로덕션에 들일 때 반드시 짚어야 한다.

## 2. Stage 2 — 변종 10개

기준 샘플 `samples/chart/세로막대형/묶은세로막대형` 의 **첫 계열 첫 값**을 `4.3` → `91.7`
로 바꿨다. 원본 최대값이 `5` 라 반영되면 첫 막대가 차트를 뚫고 솟는다 — 확대해서 숫자를
읽을 필요가 없다.

| 파일 | ① zip | ② 중첩OOXML | ③ 레거시 | ④ EMF | 크기 |
|---|---|---|---|---|---|
| `00-control-원본.hwpx` | 4.3 | 4.3 | 4.3 | 유지 | 40,484 B |
| `00-control-원본.hwp` | — | 4.3 | 4.3 | 유지 | 36,864 B |
| `X-A-zip파트만.hwpx` | **91.7** | 4.3 | 4.3 | 유지 | 40,385 B |
| `X-B-레거시만.hwpx` | 4.3 | 4.3 | **91.7** | 유지 | 37,605 B |
| `X-C-셋다.hwpx` | **91.7** | **91.7** | **91.7** | 유지 | 37,506 B |
| `X-D-셋다+EMF제거.hwpx` | **91.7** | **91.7** | **91.7** | **제거** | 25,369 B |
| `H-A-중첩OOXML만.hwp` | — | **91.7** | 4.3 | 유지 | 33,792 B |
| `H-B-레거시만.hwp` | — | 4.3 | **91.7** | 유지 | 33,792 B |
| `H-C-둘다.hwp` | — | **91.7** | **91.7** | 유지 | 33,792 B |
| `H-D-둘다+EMF제거.hwp` | — | **91.7** | **91.7** | **제거** | 21,504 B |

산출 위치: `output/issue_4055_b1_spike/` (gitignored) + 판정표 `PANJEONG.md`.

### 조립 방식

- **OOXML 패치**: 첫 `c:val` 안 첫 `c:v` 텍스트만 바이트 수술. 전체 재직렬화를 하지
  않는 이유는 `src/ooxml_chart/parser.rs` 가 `c:pt idx`·`c:f`·`c:externalData`·`extLst`
  를 읽지 않아 왕복시키면 모델에 없는 것이 전부 사라지기 때문이다.
- **레거시 패치**: Stage 1 로케이터가 준 오프셋에 8바이트 제자리 덮어쓰기. 길이 불변.
- **HWPX**: zip 엔트리 교체. 손대지 않는 엔트리는 `raw_copy_file` 로 압축 방식까지 보존
  (`mimetype` 은 stored 여야 한다).
- **HWP5**: 바깥 CFB 를 전수 열거 → 해당 `BinData` 스트림만 교체 → 재포장.
  OLE Storage 페이로드는 4바이트 LE size prefix + 중첩 CFB 를 raw deflate
  (`src/serializer/cfb_writer.rs:232-255` 규약).

### 자기 검증 — 한컴에 넘기기 전에

각 변종은 내보내기 직전 두 가지를 스스로 통과해야 파일로 나간다.

1. **rhwp 가 다시 연다** (`parse_document`).
2. **라벨대로 조립됐다** — 바꾸기로 한 표현만 `91.7` 이고 나머지는 `4.3` 그대로이며
   EMF 유무도 의도대로다. `Representations` 구조체로 4중 표현의 첫 값을 각각 읽어
   기대값과 통째로 비교한다.

두 번째가 없으면 변종이 라벨과 다르게 조립돼도 모르고, 한컴 판정 결론이 통째로 뒤집힌다.

## 3. 곁가지로 고정한 것

`editing_only_the_zip_part_diverges_from_the_nested_copy` — `bin_data_content` 의
`ooxml_chart` 항목만 갈고 `export_hwpx_native()` 하면 편집이 `Chart/chartN.xml` 로
저장된다(직렬화기 수정 0). **그런데 중첩 CFB 안의 사본은 `4.3` 그대로 남는다.**
4중 표현이 갈리는 지점을 테스트로 못박았다.

## 4. 다음

`output/issue_4055_b1_spike/` 의 변종 8개와 대조군 2개는 전수 판정했다. 세부 결과와 HWP
2020 MCP 독립 재현은 [Stage 4](task_m100_4055_stage4.md)와 최종 보고서에 기록했다.

## 5. 검증

```
cargo test --profile release-test --test issue_4055_b1_chart_edit_probe   → 9 passed, 1 ignored
cargo clippy --profile release-test --all-targets -- -D warnings          → 통과
cargo fmt (해당 파일)                                                      → 통과
```

프로덕션 코드 변경 0.
