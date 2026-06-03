# 타스크 3 - 최종 결과 보고서: HWP 파서 구현

## 목표

HWP 5.0 파일의 바이너리 데이터를 파싱하여 IR(Document Model)로 변환하는 파서를 구현한다.
`HwpDocument::from_bytes()` API를 통해 실제 HWP 파일을 로드하고, CLI로 문서 정보 확인 및 SVG 내보내기가 가능해야 한다.

## 단계별 수행 결과

| 단계 | 내용 | 테스트 | 상태 |
|------|------|--------|------|
| 1단계 | CFB 컨테이너 + 레코드 + FileHeader | 88 → 127 (+39) | 승인 완료 |
| 2단계 | DocInfo 파싱 (참조 테이블) | 127 → 152 (+25) | 승인 완료 |
| 3단계 | BodyText 파싱 (문단/텍스트) + 배포용 복호화 | 152 → 175 (+23) | 승인 완료 |
| 4단계 | 컨트롤 파싱 (표/도형/그림/머리말) + BinData | 175 → 175 (±0, 리팩토링) | 승인 완료 |
| 5단계 | API 연결 + CLI + 빌드 검증 + ctrl_id 버그 수정 | 175 → 177 (+2) | 승인 완료 |

## 구현 결과

### 파서 모듈 (src/parser/)

| 모듈 | 파일 | 라인 | 역할 |
|------|------|------|------|
| mod.rs | `src/parser/mod.rs` | 156 | 통합 파싱 파이프라인 `parse_hwp()` |
| cfb_reader | `src/parser/cfb_reader.rs` | 245 | CFB 컨테이너 + 압축 해제 |
| header | `src/parser/header.rs` | 270 | FileHeader 바이너리 파싱 |
| record | `src/parser/record.rs` | 224 | 레코드 헤더 파싱 |
| tags | `src/parser/tags.rs` | 296 | HWP 태그/컨트롤 상수 (ctrl_id BE 인코딩) |
| byte_reader | `src/parser/byte_reader.rs` | 250 | 바이너리 읽기 유틸리티 |
| crypto | `src/parser/crypto.rs` | 503 | 배포용 문서 복호화 (AES-128 ECB) |
| doc_info | `src/parser/doc_info.rs` | 773 | DocInfo 참조 테이블 파싱 |
| body_text | `src/parser/body_text.rs` | 920 | BodyText 섹션/문단 파싱 |
| control | `src/parser/control.rs` | 893 | 컨트롤 파싱 (표/도형/그림/머리말) |
| bin_data | `src/parser/bin_data.rs` | 91 | BinData 스토리지 추출 |
| **합계** | **11개 파일** | **4,621** | |

### 파싱 파이프라인

```
HWP 바이트
  │
  ├── CFB 컨테이너 열기 (cfb_reader)
  │
  ├── FileHeader 파싱 (header)
  │     └── 버전, 압축, 배포용, 암호화 플래그
  │
  ├── DocInfo 파싱 (doc_info)
  │     └── 폰트, 글자모양, 문단모양, 스타일, 테두리/배경, 탭정의
  │
  ├── BodyText 섹션 파싱 (body_text + control)
  │     ├── [배포용] ViewText 복호화 (crypto: AES-128 ECB)
  │     ├── 구역 정의 (secd → PageDef, FootnoteShape, PageBorderFill)
  │     ├── 단 정의 (cold → ColumnDef)
  │     ├── 문단 파싱 (텍스트 + 스타일 참조 + 줄세그먼트)
  │     └── 컨트롤 파싱 (표, 도형, 그림, 머리말/꼬리말, 각주/미주)
  │
  └── Document IR 조립
        │
        ├── WASM API (wasm_api.rs)
        │     ├── HwpDocument::from_bytes(data) → 파싱 + 페이지네이션
        │     ├── render_page_svg() → SVG 렌더링
        │     └── document_info_json() → JSON 문서 정보
        │
        └── CLI (main.rs)
              ├── rhwp info <파일.hwp>
              └── rhwp export-svg <파일.hwp>
```

### 지원 범위

| 항목 | 상태 | 비고 |
|------|------|------|
| CFB 컨테이너 | 구현 | BodyText, ViewText, DocInfo, FileHeader |
| 압축 해제 | 구현 | flate2 (zlib) |
| 배포용 문서 복호화 | 구현 | AES-128 ECB + LCG/XOR 키 생성 |
| FileHeader | 구현 | 버전, 압축/암호화/배포 플래그 |
| DocInfo | 구현 | 폰트, 글자모양, 문단모양, 스타일, 테두리/배경, 탭정의 |
| 문단 텍스트 | 구현 | UTF-16LE, 인라인/확장 컨트롤 코드 |
| 문단 스타일 참조 | 구현 | CharShapeRef, LineSeg, RangeTag |
| 구역/단 정의 | 구현 | PageDef, ColumnDef |
| 표 | 구현 | Table, Cell, 셀 내 문단 재귀 파싱 |
| 도형 | 구현 | 직선, 사각형, 타원, 그림 |
| 머리말/꼬리말 | 구현 | 문단 목록 재귀 파싱 |
| 각주/미주 | 구현 | 문단 목록 재귀 파싱 |
| BinData | 구현 | 임베디드 이미지 추출 |
| 암호화 문서 | 미지원 | ParseError::EncryptedDocument 반환 |
| 수식/차트/OLE | 미지원 | 향후 |

## 버그 수정 이력

### ctrl_id() 바이트 순서 버그 (5단계)

**근본 원인**: `tags.rs`의 `ctrl_id()` 함수가 little-endian 바이트 순서를 사용했으나, HWP 파일의 ctrl_id는 big-endian 문자열 인코딩으로 저장됨.

```
수정 전: (s[0] as u32) | ((s[1] as u32) << 8) | ...     → 0x64636573 ("secd" LE)
수정 후: ((s[0] as u32) << 24) | ((s[1] as u32) << 16) | ... → 0x73656364 ("secd" BE)
```

**영향**: 모든 컨트롤 ID(secd, cold, tbl 등)가 불일치 → SectionDef/PageDef 미파싱 → SVG viewBox="0 0 0 0"
**수정**: `tags.rs` 1줄 수정으로 전체 컨트롤 파싱 정상화

## 빌드 및 테스트 결과

| 항목 | 결과 |
|------|------|
| 네이티브 빌드 | 성공 (경고 0개) |
| 전체 테스트 | **177개 통과** (타스크 2 대비 +89개) |
| WASM 빌드 | 미확인 (네이티브 검증 완료) |

### 테스트 분포

| 모듈 | 테스트 수 |
|------|----------|
| parser/cfb_reader | 9 |
| parser/header | 8 |
| parser/record | 7 |
| parser/tags | 4 |
| parser/byte_reader | 11 |
| parser/crypto | 4 |
| parser/doc_info | 12 |
| parser/body_text | 20 |
| parser/control | 14 |
| parser/mod | 2 |
| model/* | 6 |
| renderer/* | 55 |
| wasm_api | 15 |
| 기타 | 10 |
| **합계** | **177** |

## 실제 HWP 파일 검증

### 검증 대상

예제 폴더(`/home/edward/vsworks/shwp/samples/15yers/`)의 실제 HWP 파일로 엔드투엔드 검증.
참조 데이터(`/home/edward/vsworks/shwp/outputs/15years/`)와 비교.

### info 명령 결과

```
파일: 통합재정통계(2014.8월).hwp
크기: ... bytes
버전: 5.0.3.4
압축: 예
암호화: 아니오
배포용: 아니오
구역 수: 1
페이지 수: 1
폰트(한글): 함초롬돋움, 함초롬바탕, ...
스타일: 바탕글, 본문, 개요 1, ...
총 문단 수: 17
```

### export-svg 결과

| 파일 | viewBox | 텍스트 | 참조 대비 |
|------|---------|--------|-----------|
| hwp_table_test.svg | `0 0 793.69 1122.51` (A4) | 11줄 정상 | 정상 |
| 통합재정통계(2014.8월).svg | `0 0 793.71 1122.51` (A4) | 8줄 정상 | 참조 .md와 텍스트 일치 |

### SVG 품질 (ctrl_id 수정 전후)

| 항목 | 수정 전 | 수정 후 |
|------|---------|---------|
| viewBox | `0 0 0 0` | `0 0 793.71 1122.51` |
| width × height | 0 × 0 | 793.71 × 1122.51 (A4) |
| 텍스트 x좌표 | 0 (여백 없음) | 94.49 (좌측 여백 적용) |
| 텍스트 y좌표 | 겹침 | 페이지 내 분산 배치 |

## 알려진 제한사항 (렌더러 타스크 범위)

| 항목 | 설명 |
|------|------|
| 폰트 매핑 | TextStyle에 DocInfo 폰트 정보 미반영 (기본 sans-serif) |
| 표 렌더링 | 페이지네이션/레이아웃에서 표 처리 미구현 |
| 도형 렌더링 | 페이지네이션/레이아웃에서 도형 처리 미구현 |
| 이미지 렌더링 | SVG <image> 태그 생성 미구현 |

## 프로젝트 전체 현황

| 타스크 | 상태 | 테스트 |
|--------|------|--------|
| 1. 개발환경 설정 | 완료 | - |
| 2. 뷰어 렌더링 엔진 설계 | 완료 | 88개 |
| 3. HWP 파서 구현 | **완료** | **177개** (+89) |

## 상태

- 완료일: 2026-02-05
- 상태: 승인 대기
