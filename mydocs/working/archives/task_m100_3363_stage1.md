# Task #3363 Stage 1 — HWP3 내장 OLE payload 추출 (수행계획서)

## 배경 (이슈 #3363 조사 확정 사실)

`samples/SO-SUEOP.hwp` 1쪽 이미지는 외부 연결 그림이 아니라 **내장 한컴 글맵시(HMapsi)
OLE 개체**다: 그림 레코드 pic_type=1(표 44 offset 74), 이름 `00000000.OOO`는 참조명
(12.1절), 실데이터는 추가 정보 블록의 OLE CFB 스토리지(3.8절/표 82 — 압축 해제 본문
offset 469,900에 CFB 시그니처 실측). 한컴 변환 `SO-SUEOP.hwpx`는 같은 개체를
`BinData/ole1.ole`(19,972B)로 추출·보존한다.

rhwp 파서의 추가 정보 블록 순회(mod.rs:3202~)는 **id=1(포함 그림)·id=3(하이퍼링크)만
처리하고 id=2(OLE 정보)는 미처리** — 결손 지점. 그 결과 pic_type=1 BinData가 payload
없는 Link로 남아 `populate_link_image_paths()`가 외부 파일 경로로 노출한다
(사이드카 워크어라운드의 발생 지점).

## 코퍼스 스코핑 (2026-07-26 실측)

V3 파일 271개 전수 압축 해제 스윕: 내장 OLE(CFB) 보유는 **SO-SUEOP.hwp 단 1개**.
영향 범위는 좁지만 스펙 12절의 결손 보완이며, 1쪽 미표시의 근본 해결이다.

## 기존 인프라 (재사용)

- `cfb` crate 0.14 + `src/parser/cfb_reader.rs` — CFB 열람.
- `src/parser/ole_container.rs` — `parse_ole_container()`(preview 추출),
  `is_hmapsi_ole_container()`(글맵시 판별).
- HWPX 경로: BinData `*.ole` → extension "ole" → 렌더러(shape_layout)의 OLE
  preview/HMapsi clip 노드. **HWP3도 같은 확장자·같은 소비 경로에 태우는 것이 목표.**

## 수행 방침

1. **추가 정보 블록 id=2 처리 추가** (`src/parser/hwp3/mod.rs` 순회 루프):
   스펙 12.1절 — 인식 정보(4B)를 건너뛰고 CFB 스토리지를 얻는다.
2. **스토리지 → 개체별 payload 분해**: CFB 내부 구조(개체별 서브 스토리지 명명)를
   실측으로 확정(Stage 2 첫 작업). 그림 코드 참조명(`00000000.OOO` 등)과 매칭해
   `pic_name_to_id`의 bin_data id로 `BinDataContent { extension: "ole" }` 주입.
   단일 개체 케이스(SO-SUEOP)를 1차 목표로 하되, 다중 개체 구조도 스펙(모든 OLE를
   하나의 스토리지에 모음)대로 순회 설계.
3. **렌더 경로 무신설**: HWPX OLE와 동일한 ext "ole" 소비 경로 재사용. HWP3 전용
   분기를 렌더러에 추가하지 않는다(CLAUDE.md 파서 경계 규칙). 필요 시 서브 스토리지
   재포장(standalone CFB) 여부는 `parse_ole_container()`의 입력 요구를 실측해 판단.
4. **스펙 보완 주석**(한글문서파일구조3.0.md 표 44 아래): pic_type=1/2의 이름은 내부
   참조명 — 외부 파일 취급 금지, 데이터는 추가 정보 블록 id=2(표 82). 실측 근거 명시.
5. **Link 노출 정리**: payload 주입 성공 시 해당 BinData는 Link가 아니게 되므로
   `populate_link_image_paths()`/`getExternalImageBasenames()` 대상에서 자연 제외
   (코드 수정 없이 소거되는지 실측 확인).

## 검증

1. 단위: id=2 블록 파싱(합성 CFB 아님 — SO-SUEOP 실블록 기반 고정 fixture 우선 검토).
2. **payload 대조**: 추출 바이트를 한컴 변환 `SO-SUEOP.hwpx`의 `BinData/ole1.ole`와
   대조(동일성 또는 구조적 등가 — 한컴 재포장 여부에 따라 판단 기준 확정).
3. **1쪽 표시**: 사이드카 파일 없이 CLI(svg/render-tree) + studio(wasm)에서 글맵시
   표시. `getExternalImageBasenames()` 빈 배열 확인(#3348 가드와 정합).
4. 회귀: HWP3 코퍼스 271개 스모크(파싱 크래시 0) + `cargo test --tests --profile
   release-test` + `fmt --check`.
5. **시각 판정 게이트(작업지시자)**: 1쪽 렌더 vs 한컴.

## 비범위

- 진짜 pic_type=0 문서의 사이드카 공급 UX(#3313 코멘트로 재정의된 잔여) — 별도 과제.
- OLE 본문 편집·저장 왕복(HWPX 저장 시 내장 OLE의 BinData 방출)은 표시 확인 후
  범위 판단(커지면 후속 이슈).

## 릴리즈

0.8.1 포함 여부는 구현 크기 실측 후 작업지시자 판단(작은 단위 PATCH 회전 방침).
#3303·#3348만으로 0.8.1을 먼저 내는 선택지 유지.

## 다음 단계

승인 시 Stage 2(구현계획서): CFB 내부 구조 실측 → 주입 지점 코드 설계 → 검증 커맨드
확정.
