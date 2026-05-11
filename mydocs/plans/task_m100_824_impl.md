# Task #824 구현 계획서

**선행 문서**: [task_m100_824.md](task_m100_824.md) (수행계획서, 승인 완료)
**브랜치**: `local/task824`

## 개요

HWP3 그림 파서 [src/parser/hwp3/mod.rs:935-952](../../src/parser/hwp3/mod.rs#L935-L952) 의 `pic_type` 분기 누락 결함 정정. RED → GREEN → 회귀 → 시각 검증 4단계.

## 단계별 상세

### 단계 1 — RED (회귀 테스트 작성 + FAIL 확인)

**목적**: 결함이 테스트로 객관적으로 잡히는지 먼저 입증한다.

**작업**:
1. `tests/issue_824.rs` 신규 작성:
   - `issue_824_embedded_picture_no_external_path` — sample11.hwp 의 첫 임베디드 그림 → `external_path == None` 단언 (현행 FAIL 예상)
   - `issue_824_external_picture_keeps_external_path` — sample10.hwp 의 외부 file path 그림 → `external_path == Some(_)` 단언 (현행 PASS — 회귀 가드)
2. fixture 추가: `git add samples/hwp3-sample11.hwp samples/hwp3-sample11-hwp5.hwp samples/hwp3-sample11-hwpx.hwpx pdf/hwp3-sample11-hwpx-2022.pdf`
3. `cargo test issue_824` 실행 → embedded 테스트 FAIL 확인 (external 테스트 PASS 확인)
4. `mydocs/working/task_m100_824_stage1.md` 작성 (FAIL 메시지 캡처 포함)
5. 커밋: `Task #824 Stage 1 (RED): 회귀 테스트 + sample11 fixture + FAIL 확인`

**산출물**: `tests/issue_824.rs`, fixture 파일들, `_stage1.md`

**테스트 골격**:
```rust
//! Issue #824: HWP3 임베디드 그림이 외부 파일 참조로 잘못 표시됨
//!
//! 본질: src/parser/hwp3/mod.rs 의 pic_type 분기 누락 — pic_type 0/1/2 를 동일
//! 처리하여 임베디드 그림(2)에도 external_path 가 설정됨.
//!
//! 정정: pic_type == 0 (외부 파일) 만 external_path 설정.

use rhwp::model::control::Control;
use std::fs;
use std::path::Path;

fn first_picture_external_path(hwp: &str) -> Option<String> {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(hwp);
    let data = fs::read(&path).expect("read hwp");
    let doc = rhwp::parser::hwp3::parse_hwp3(&data).expect("parse hwp3");
    for sec in &doc.sections {
        for para in &sec.paragraphs {
            for ctrl in &para.controls {
                if let Control::Picture(pic) = ctrl {
                    return pic.image_attr.external_path.clone();
                }
            }
        }
    }
    None
}

#[test]
fn issue_824_embedded_picture_no_external_path() {
    let path = first_picture_external_path("samples/hwp3-sample11.hwp");
    assert!(
        path.is_none(),
        "임베디드 그림(pic_type=2)은 external_path 가 None 이어야 함 \
         (현행 결함: Some(\"E$$00000.jpg\")). got: {:?}",
        path,
    );
}

#[test]
fn issue_824_external_picture_keeps_external_path() {
    let path = first_picture_external_path("samples/hwp3-sample10.hwp");
    assert!(
        path.is_some(),
        "외부 file path 그림(pic_type=0)은 external_path 가 Some 이어야 함",
    );
}
```

> 테스트가 첫 그림만 검사 — sample11/sample10 은 첫 그림이 각각 embedded/external 임을 사전 dump 로 확인했음.

---

### 단계 2 — GREEN (pic_type 분기 수정)

**목적**: 최소 변경으로 단계 1 의 RED 테스트를 PASS 시킨다.

**작업**:
1. [src/parser/hwp3/mod.rs:935-952](../../src/parser/hwp3/mod.rs#L935-L952) 수정:
   ```rust
   let pic_type = info_buf[74];
   if pic_type == 0 || pic_type == 1 || pic_type == 2 {
       let pic_name_buf = &info_buf[83..83+256];
       let mut pic_name = crate::parser::hwp3::encoding::decode_hwp3_string(pic_name_buf);
       pic_name = pic_name.trim_end_matches('\0').to_string();

       let _block_num = ...;
       let _pic_info_size = ...;

       if !pic_name.is_empty() {
           // [Task #824] pic_type == 0 (외부 파일) 만 external_path 설정.
           // pic_type == 1 (OLE), 2 (Embedded) 는 pic_name 이 내부 참조명이므로
           // external_path 미설정 (한컴오피스 2022 정합).
           if pic_type == 0 {
               pic.image_attr.external_path = Some(pic_name.clone());
           }
           let next_id = (pic_name_to_id.len() + 1) as u16;
           let id = *pic_name_to_id.entry(pic_name).or_insert(next_id);
           pic.image_attr.bin_data_id = id;
       }
   } else if pic_type == 3 {
       ...
   }
   ```
2. `cargo test issue_824` → 두 테스트 모두 PASS 확인
3. `mydocs/working/task_m100_824_stage2.md` 작성 (수정 diff + 테스트 결과)
4. 커밋: `Task #824 Stage 2 (GREEN): pic_type == 0 만 external_path 설정`

**산출물**: 수정된 `mod.rs`, `_stage2.md`

---

### 단계 3 — 회귀 검증

**목적**: 본 수정이 다른 HWP3 sample 의 출력에 영향이 없음을 확인.

**작업**:
1. `cargo test` 전체 통과
2. `cargo clippy --all-targets -- -D warnings`
3. baseline (head before stage 2) vs patched 비교 — 보유 HWP3 sample 5개 전체 page SVG 출력 비교:
   - hwp3-sample.hwp / sample4 / sample5 / sample10 / sample11
   - **sample10**: 외부 file path 그림 placeholder 정상 (회귀 없음)
   - **sample11**: 임베디드 그림 placeholder 미발현 (실제 image 표시) ← 본 수정의 의도 효과
   - 기타 sample: bit-identical 또는 정상 변화만
4. `mydocs/working/task_m100_824_stage3.md` 작성 (회귀 결과 표 + diff 발생 page 분석)
5. 커밋: `Task #824 Stage 3 (회귀): cargo test + HWP3 sample 5개 SVG 회귀 검증`

**검증 스크립트**: 단계 1 의 PR #796 회귀 검증과 동일 패턴 (git stash + cargo build + export-svg + cmp).

**산출물**: `_stage3.md`

---

### 단계 4 — 시각 검증 + 최종 보고서

**목적**: 작업지시자가 rhwp-studio UI 에서 한컴오피스 2022 정합을 시각 확인.

**작업**:
1. WASM 재빌드 (`docker compose --env-file .env.docker run --rm wasm`)
2. rhwp-studio dev 서버 실행 → 작업지시자 시각 검증 요청:
   - `samples/hwp3-sample11.hwp` 로드 → 첫 그림 더블클릭 → 그림 탭
     - 파일 이름: 빈 값 ✓
     - 문서에 포함: 체크됨 ✓
   - `samples/hwp3-sample10.hwp` 로드 → 외부 file path 그림 정상 (회귀 없음)
3. 시각 판정 통과 후:
   - 최종 보고서 `mydocs/report/task_m100_824_report.md` 작성
   - 오늘할일 `mydocs/orders/20260511.md` 본 task 상태 갱신
4. 커밋: `Task #824 Stage 4 (최종): 시각 판정 통과 + 보고서 + closes #824`

**산출물**: `_report.md`, orders 갱신, 최종 commit

---

## 단계별 commit 계획 요약

| 단계 | commit 메시지 | 변경 파일 |
|---|---|---|
| 1 | `Task #824 Stage 1 (RED): 회귀 테스트 + sample11 fixture + FAIL 확인` | `tests/issue_824.rs`, samples/, pdf/, `_stage1.md` |
| 2 | `Task #824 Stage 2 (GREEN): pic_type == 0 만 external_path 설정` | `src/parser/hwp3/mod.rs`, `_stage2.md` |
| 3 | `Task #824 Stage 3 (회귀): cargo test + HWP3 sample 5개 SVG 회귀 검증` | `_stage3.md` |
| 4 | `Task #824 Stage 4 (최종): 시각 판정 통과 + 보고서 + closes #824` | `_report.md`, orders, body 의 `closes #824` |

## 위험 / 가정

- **가정**: `samples/hwp3-sample10.hwp` 의 첫 그림이 외부 file path 그림 (`pic_type == 0`) — 사전 `dump` 확인됨
- **가정**: `samples/hwp3-sample11.hwp` 의 첫 그림이 임베디드 그림 (`pic_type == 2`) — 사전 `dump` 확인됨 (`E$$00000.jpg` 패턴이 증거)
- **잔여 OLE (pic_type == 1) 검증 미흡**: 보유 sample 중 OLE 그림 케이스 부재 — 본 task 범위에서 별도 회귀 검증 불가. 본 수정은 안전한 방향 (`external_path = None`) 이므로 OLE sample 발견 시 별도 task 로 처리.
- **PR 절차**: 본 수정은 외부 컨트리뷰터 PR 이 아닌 내부 task 이므로 fork → upstream PR 절차는 메인테이너 워크플로우 적용 (단계 4 후 별도 협의).
