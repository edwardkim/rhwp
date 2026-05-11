# Task #824 Stage 2 (GREEN) 보고서

**브랜치**: `local/task824`
**선행**: Stage 1 (RED) 완료 — embedded FAIL, external PASS
**목표**: 최소 변경으로 RED 테스트 PASS 전환

## 수정 내용

[src/parser/hwp3/mod.rs:944-955](../../src/parser/hwp3/mod.rs#L944-L955)

```diff
                                 if !pic_name.is_empty() {
-                                    // [Task #741] 외부 file path IR 전달 (Renderer placeholder
-                                    // 처리용). HWP3 spec offset 74 그림 종류 0=외부 파일,
-                                    // 1=OLE, 2=Embedded Image / offset 83~339 그림 파일 이름.
-                                    pic.image_attr.external_path = Some(pic_name.clone());
+                                    // [Task #824] pic_type == 0 (외부 파일) 만 external_path
+                                    // 설정. pic_type == 1 (OLE) / 2 (Embedded) 는 pic_name 이
+                                    // 내부 참조명 (예: "E$$00000.jpg") 이므로 external_path
+                                    // 설정 시 그림 속성 dialog 가 외부 파일로 오표시됨
+                                    // (한컴오피스 2022 정합).
+                                    if pic_type == 0 {
+                                        pic.image_attr.external_path = Some(pic_name.clone());
+                                    }
                                     let next_id = (pic_name_to_id.len() + 1) as u16;
                                     let id = *pic_name_to_id.entry(pic_name).or_insert(next_id);
                                     pic.image_attr.bin_data_id = id;
                                 }
```

핵심 변경: 1줄 if 추가 (`if pic_type == 0`). `bin_data_id` 매핑은 type 무관하게 유지.

## 검증 결과

```
$ cargo test --test issue_824
running 2 tests
test issue_824_embedded_picture_no_external_path ... ok
test issue_824_external_picture_keeps_external_path ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured
```

**해석**:
- ✅ embedded 테스트 PASS — sample11 의 임베디드 그림 `external_path == None`
- ✅ external 테스트 PASS (회귀 가드 유지) — sample10 의 외부 file path 그림 `external_path = Some(_)`

## 다음 단계

Stage 3 (회귀) — `cargo test` 전체 통과 + 보유 HWP3 sample 5개 SVG 출력 회귀 검증.
