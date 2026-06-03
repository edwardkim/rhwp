# Task #463 Stage 8 완료보고서 (확장 바탕쪽 헤더 중복 렌더링 수정)

## 발견 (Stage 7 검토 중)

작업지시자가 사용자 보고 (Stage 6 후) 의 "2와 4 겹쳐 보임" 을 검증하던 중, 16페이지 (sec1 마지막 짝수 쪽) 좌측 헤더에 **2** (master[0] Both) 와 **4** (master[2] Both is_ext=true overlap=true) 가 같은 위치에 그려져 시각적 겹침을 확인.

PDF 참조: 좌측에 "**4**" 만 표시. 동일 패턴이 sec2 의 마지막 쪽 (page 20) 에도 존재.

> 사용자가 페이지 번호를 "15" 로 인지한 것은 booklet 의 보이는 페이지 번호 ("3" → 다음 페이지 4 = 16) 와 SVG export -p 인덱스 (0-base) 사이 혼동으로 추정.

## 원인

`src/document_core/queries/rendering.rs:996-1019` 의 확장 바탕쪽 적용 로직:

```rust
if is_last && !ext_mp_indices.is_empty() {
    let overlap_exts: Vec<_> = ... filter(|m| m.overlap) ...;
    let replace_exts: Vec<_> = ... filter(|m| !m.overlap) ...;
    
    if let Some(replace_idx) = replace_exts.last() {
        page.active_master_page = Some(...);  // 대체
    }
    if !overlap_exts.is_empty() {
        page.extra_master_pages = ...;  // 추가 (overlay)
    }
}
```

`overlap=true` 확장 master 는 항상 `extra_master_pages` 로 추가 → 기존 active master 와 함께 둘 다 렌더 → 같은 위치 셀 컨텐츠 시각 겹침.

samples/exam_kor.hwp 의 master 구성 (모든 sec):
- master[0] Both (정규)
- master[1] Odd (정규)
- master[2] Both, is_ext=true, **overlap=true** (확장)

마지막 쪽 (짝수): active=master[0], extra=master[2] → 헤더 1×3 표가 같은 위치 (vert=Paper/9921, horz=Paper/8788) 에 두 번 그려짐. 셀 fill_type=None alpha=0 이라 시각적으로 중첩.

## HWP 스펙 해석

기존 코드 주석: "겹치기(overlap): 기존 바탕쪽 위에 추가". 그러나 한컴 PDF 출력은 "대체" 로 동작. 작성자 의도:
- master[2] = 마지막 쪽 전용 헤더 (다른 페이지 번호) + 추가 본문 ("확인 사항")
- 원래 헤더 (master[0]) 는 마지막 쪽에서 표시되지 않아야 함

따라서 **`overlap=true` 확장이 active master 와 같은 `apply_to`** 인 경우, 의미적으로 "같은 종류 헤더의 마지막-쪽-전용 변형" 이므로 active 를 대체하는 것이 맞음.

## 수정

`apply_to` 비교 후 같으면 대체, 다르면 extra:

```rust
// 겹침형 확장:
// - apply_to 가 active 와 동일: active 대체 (한컴 PDF 동작 일치)
// - apply_to 가 다름: extra 로 추가
let active_apply = page.active_master_page.as_ref()
    .and_then(|mp_ref| mps.get(mp_ref.master_page_index))
    .map(|m| m.apply_to);
let mut remaining_overlap_exts: Vec<usize> = Vec::new();
for &i in &overlap_exts {
    if Some(mps[i].apply_to) == active_apply {
        page.active_master_page = Some(MasterPageRef {
            section_index: idx,
            master_page_index: i,
        });
    } else {
        remaining_overlap_exts.push(i);
    }
}
if !remaining_overlap_exts.is_empty() {
    page.extra_master_pages = remaining_overlap_exts.iter()...;
}
```

## 검증

### 페이지 16 (sec1 마지막, 짝수) 헤더

| 단계 | 좌측 | 중앙 | 결과 |
|------|------|------|------|
| Stage 7 (이전) | **2** + **4** 겹침 | 국어 영역(화법과 작문) | NG |
| Stage 8 (이후) | **4** ✓ | 국어 영역(화법과 작문) | OK |
| PDF (참조) | 4 | 국어 영역(화법과 작문) | — |

### 다른 마지막 쪽

| 페이지 | active master | 결과 |
|--------|---------------|------|
| 12 (sec0 last, even) | master[2] (Both ext) | "12 국어 영역" 정상 (master[0] 셀 비어있어 영향 미미했으나 일관성 확보) |
| 16 (sec1 last, even) | master[2] (Both ext) | "4" 단일, 겹침 해소 ✓ |
| 20 (sec2 last, even) | master[2] (Both ext) | "4" 단일, 겹침 해소 ✓ |

### 본문 추가 (확인 사항) 정상

각 sec 마지막 쪽의 master[2] ctrl[2] (1×1 표 "확인 사항" 본문) 도 master[2] 가 active 가 되었으므로 그대로 렌더됨 (regression 없음).

### 단위 테스트

```
cargo test --release --lib
test result: ok. 1069 passed; 0 failed
```

### 회귀 (5종 샘플)

- `2010-01-06.hwp` (6p) ✓
- `biz_plan.hwp` (6p) ✓
- `21_언어_기출_편집가능본.hwp` (15p) ✓
- `exam_eng.hwp` (8p) ✓
- `2022년 국립국어원 업무계획.hwp` (40p) ✓

모두 정상 SVG 내보내기.

## 변경 파일

| 파일 | 변경 라인 |
|------|----------|
| `src/document_core/queries/rendering.rs` | +18/-3 (overlap 분기에 apply_to 비교 추가) |

## 잔존 한계

- 다른 `apply_to` (예: active=Odd, ext=Both) 의 겹침 확장은 여전히 extra 로 추가. 이 케이스는 본 샘플에 없어 PDF 동작 미확인. 발견 시 별도 분석 필요.
- 16p PDF 의 우측 "홀수형" 뱃지는 master 내용이 아닌 본문 별도 요소로 추정 — 본 fix 와 무관. 관련 이슈는 별도 조사 필요.
