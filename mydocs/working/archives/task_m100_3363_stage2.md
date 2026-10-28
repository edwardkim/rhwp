# Task #3363 Stage 2 — 구현계획서

## Stage 2 착수 실측 (설계 확정 근거)

1. **블록 구조**: 압축 해제 본문 끝에서 추가 정보 블록 **id=2, length=20,996** 확인
   (인식 정보 4B `685599f8` + CFB 20,992B), 직후 8바이트 0 종결 마커 — 스펙 3.8/12.1절
   그대로.
2. **CFB 내부**: root에 `00000000.INF`(128B) 스트림 + **`00000000.OOO` 서브 스토리지**
   (streams: CompObj 82 / Ole 20 / Ole10Native 1,348 / **OlePres000 15,102**).
   서브 스토리지명 = 그림 레코드 참조명 — 매칭 키 확정.
3. **한컴 변환본 등가성**: `SO-SUEOP.hwpx`의 `BinData/ole1.ole` = 4바이트 크기
   프리픽스 + standalone CFB이고, 그 스트림 4개가 내장 서브 스토리지의 스트림과
   **md5 완전 동일**. 즉 한컴의 변환 = "서브 스토리지의 root 승격 재포장".
4. **rhwp 소비 계약**: hwpx `normalize_ole_bytes()`가 프리픽스를 벗겨
   **BinDataContent.data = CFB(시그니처부터)** 로 저장, 렌더는
   `ShapeObject::Ole` 분기(shape_layout:1938~)에서 `parse_ole_container()` →
   OlePres000 preview / HMapsi clip. `parse_ole_container`의 `walk()`는 중첩
   스토리지도 순회하지만, 다중 개체 오매칭을 피하려면 개체별 분리가 정도다.

## 구현

### 1. `src/parser/hwp3/ole_info.rs` (신규 모듈)

```rust
/// [#3363] 추가 정보 블록 id=2(OLE 정보, 표 82) payload 분해.
/// 입력: 인식 정보(4B)를 제외한 CFB 바이트.
/// 출력: (개체명, standalone CFB 바이트) 목록 — 각 서브 스토리지의 스트림을
/// root로 승격한 새 CFB로 재포장(한컴 HWPX 변환과 동형, 실측 md5 등가).
pub fn extract_ole_payloads(cfb_bytes: &[u8]) -> Vec<(String, Vec<u8>)>
```

- `cfb::CompoundFile::open`으로 열고 root 서브 스토리지를 순회, 각 스토리지의
  스트림을 `cfb::create`로 만든 인메모리 CFB root에 복사.
- root 직속 스트림(`*.INF` 등)은 무시. 실패 개체는 건너뛰고 나머지 진행(관대 읽기).

### 2. `src/parser/hwp3/mod.rs` — 블록 순회에 id=2 분기 추가

id=1 골격과 동일하게: `extract_ole_payloads(&block.data[4..])` → 각 (name, bytes)를
`pic_name_to_id` 매칭 → `BinDataContent { id, extension: "ole", data }` +
BinData(Embedding) 등록, `processed_ids` 반영.

### 3. `fixup_hwp3_ole_pictures(&mut doc)` (mod.rs 후처리)

bin_data_content에 ext "ole" payload가 **실제로 주입된** 그림 컨트롤만
`ShapeObject::Picture` → `ShapeObject::Ole`(OleShape: common·크기·bin_data_id 이관)로
변환. payload 없는 pic_type=1(코퍼스 외 파일)은 현행 유지(Picture+Link) — 행동 변화를
payload 확보 케이스로 한정하는 가드. 효과:
- 렌더: 기존 OLE preview/HMapsi 경로 그대로 진입(렌더러 무수정, HWP3 분기 없음)
- 선택: #3319/#3321 OLE 선택 경로 상속
- HWPX 저장: hp:ole 방출 — 한컴 변환과 동형

### 4. 스펙 보완 주석 — `한글문서파일구조3.0.md` 표 44 아래

pic_type=1/2의 이름은 내부 참조명(외부 파일 취급 금지), 실데이터는 추가 정보 블록
id=2(표 82)의 스토리지 — 서브 스토리지명=참조명, 한컴 HWPX 변환은 root 승격 재포장
(md5 등가 실측). #3363 근거 명시.

## 검증

1. **단위**: `extract_ole_payloads` — samples/SO-SUEOP.hwp 실블록 기반
   (개체 1건, 이름 `00000000.OOO`, 재포장 CFB에서 `parse_ole_container()` 성공 +
   `is_hmapsi_ole_container()` true).
2. **통합**: SO-SUEOP.hwp 파싱 → ① bin_data ext "ole" 주입 확인, ② 1쪽 컨트롤이
   Ole로 변환, ③ `getExternalImageBasenames()` 빈 배열(#3348 정합, Link 소거),
   ④ 재포장 payload의 스트림 md5 = 한컴 `ole1.ole` 내부와 동일.
3. **렌더**: export-svg/render-tree 1쪽 — preview 노드 출현(수정 전 대비).
   studio(wasm 재빌드) 1쪽 표시 — **작업지시자 시각 판정 게이트**.
4. **회귀**: HWP3 코퍼스 271개 파싱 스모크(크래시 0·기존 파일 IR 무변화 —
   id=2 블록 보유 파일이 SO-SUEOP뿐이므로 구조 보장) +
   `cargo test --tests --profile release-test` + `fmt --check`.

## PR

- 커밋 1개(코드+테스트+스펙 주석+working/report), `Closes #3363`, 본문 한국어.
- PR 생성은 별도 승인 후. 릴리즈 포함 여부는 작업지시자 결정 대기.
