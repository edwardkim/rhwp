---
kind: canonical
status: active
canonical: mydocs/tech/parser_architecture.md
last_verified: 2026-08-12
---

# 포맷 파서와 공통 Document IR 경계

rhwp의 HWPX, HWP5, HWP3 파서는 포맷별 입력을 하나의 공통 `Document` IR로 변환한다. 렌더러,
레이아웃, 편집 코어는 입력 포맷을 다시 판별하지 않고 이 IR의 의미를 소비한다.

| 포맷 | 파서 위치 | 출력 |
| --- | --- | --- |
| HWPX (ZIP+XML) | `src/parser/hwpx/` | `Document` |
| HWP5 (OLE 복합 문서) | `src/parser/hwp5/` | `Document` |
| HWP3 (고전 바이너리) | `src/parser/hwp3/` | `Document` |

## 책임 경계

- 각 파서는 원본 포맷의 표현을 공통 IR 의미로 정규화한다.
- 포맷별 레코드, XML, 인코딩 차이는 해당 파서 경계를 넘기지 않는다.
- 공통 모듈은 포맷 이름이 아니라 IR 속성과 의미를 기준으로 동작한다.
- 저장 포맷별 직렬화 차이는 해당 serializer 또는 명시적인 변환 계층에서 처리한다.

## 문서 열기 자원 예산

문서 열기 과정에서 공격자 제어 압축 스트림이 무제한 출력을 만들지 않도록 다음 결정적 byte 예산을
적용한다. 이 수치는 HWP 또는 OWPML 규격의 유효성 상한이 아니라 rhwp 구현의 자원 정책이다.

| 경로 | 단일 출력 상한 | 문서 누적 상한 | 초과 동작 |
| --- | ---: | ---: | --- |
| HWP5 `DocInfo`, 각 `BodyText/SectionN` | 256 MiB | `DocInfo`와 모든 본문 섹션 합계 512 MiB | 문서 열기 오류 |
| HWP3 압축 본문 | 256 MiB | 별도 누적 없음 | 문서 열기 오류 |

- HWP5의 strict·lenient, 일반·배포용·비밀번호 암호 경로는 같은 예산을 사용한다.
- 예산의 **선택**은 완전 문서 열기 진입점(`parse_hwp*`, `parse_document*`,
  `hwp3::parse_hwp3*`)에만 있다. CFB/crypto의 일반 decode API는 제품 기본 상한을 import하지
  않으며, 호출자가 준 `max_bytes`를 기계적으로 적용하는 `_limited` API만 제공한다.
- HWP5 비압축 `DocInfo`와 본문도 같은 길이 검사를 받는다. 따라서 이 계약은 압축 폭탄 방지인 동시에
  해당 핵심 스트림의 일반 크기 정책이다.
- 초과 출력을 잘라서 부분 문서로 열거나 빈 섹션으로 대체하지 않는다. 예산 초과를 명시적 파싱 오류로
  반환해 조용한 내용 손실을 막는다.
- `BinData`, preview와 기타 보조 스트림은 이 누적 예산의 대상이 아니다. 각 소비 경로의 별도 자원
  정책을 적용한다. 특히 암호 BinData materialize의 상한은 그 consumer인 parser가 명시적으로
  전달하며, 핵심 문서 열기 예산과 섞지 않는다.
- `dump-records`와 등록 HWP5 raw-record diagnostics처럼 문서를 독립적으로 여는 consumer는
  문서 열기 예산을 import하지 않고, 각 consumer 소유의 이름 붙은 상한을 limited CFB/crypto API에
  명시적으로 전달한다.

2026-08-12 이전 검토 code head에서의 비공개 10k 코퍼스 전수 검증에서는 제한 초과가 0건이었다.
이는 현재 correction commit을 다시 전수 실행한 결과가 아니라, 같은 256/512 MiB 수치의 호환성
근거다. HWP5 단일 스트림 최대는
21,606,061 bytes(20.61 MiB), `DocInfo`와 본문 누적 최대는 21,678,881 bytes(20.67 MiB)였다.
이는 현재 코퍼스의 호환성 근거이며 모든 실문서가 상한 안이라는 규격 증명은 아니다.

## HWP3 불변식

HWP3 바이너리 해석과 HWP3 전용 보정은 `src/parser/hwp3/` 안에서 완료한다. 다음 공통 영역에는 HWP3
전용 분기를 추가하지 않는다.

- `src/renderer/`
- `src/renderer/layout.rs` 및 하위 레이아웃 모듈
- `src/document_core/`

공통 영역에 추가 정보가 필요하면 HWP3 여부를 직접 전달하지 않고, 다른 포맷에도 적용 가능한 IR 속성으로
정의한다. 이 규칙은 포맷별 예외가 렌더링과 편집 계층으로 확산되는 것을 막는다.

## 소스 출처와 레이아웃 호환 정책 (#2403)

소스 포맷·변환 계보 판단은 파싱 시점에 `Document.provenance`
(`src/model/provenance.rs` 의 `SourceProvenance`)로 한 번 확정된다 — 쓰기
지점은 파서로 한정한다. 렌더러·레이아웃·편집 계층은 boolean 필드를 직접
읽지 않고 `Document::layout_profile()` 이 돌려주는
`LayoutCompatibilityProfile` 질의(`hwp3_layout`/`hwp3_native_layout`/
`hwpx_stored_layout`/`hwp5_origin_hwpx`)를 사용한다.

- **신규 소스분기는 profile 질의 추가로만 연다** — 흩어진 boolean 전달이나
  포맷 이름 직접 비교를 새로 만들지 않는다.
- 문서군 판별 신호(생성기·재저장 서명 등)가 확정되면 `SourceProvenance` 의
  서명 필드와 profile 질의로 수용한다 (#2373 잔여 판별자 트랙).
- 기존 `is_hwp3_variant`/`is_hwpx_variant` 필드는 shim 이며 파서의 같은 쓰기
  지점에서 provenance 와 동기된다 — 신규 코드에서 읽지 않는다.
