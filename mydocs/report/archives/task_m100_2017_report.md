# 최종 결과보고 — #2017 curSz=0 sentinel HWPX 재직렬화 충실도 복원

브랜치 `fix/2017-cursz-fidelity` (base: devel).

## 1. 배경
#2017(IR-invisible HWPX 저장 페이지붕괴, #1589 잔여). 구조 IR diff=0인데 한글 재열림 시 페이지수 변동.

## 2. 근본원인 (XML orig↔rt 직접 대조, 오라클 불요)
`materialize_shape_current_size_from_original`(`src/parser/hwpx/section.rs`)가 HWPX **파싱** 시 `curSz.w/h==0 && orgSz>0`이면 curSz를 orgSz로 치환(HWP5 저장·렌더가 실크기 필요). 이 치환이 IR을 오염 → **HWPX 재직렬화가 원본 `curSz=0` 대신 materialize값 기록** → 한글이 도형 geometry를 다르게 읽음.

계측(21965845 응시원서): 원본 `curSz 0×0`(×12)·`44752×0`(×2)이 rt에서 `2794×2792`·`44752×100`으로 변형.

## 3. 수정 (충실도 복원, materialize 유지)
`ShapeComponentAttr`에 `current_width_was_zero`/`current_height_was_zero` 플래그 추가. 파싱 materialize 시 dimension별로 기록. **HWPX 직렬화(picture.rs·shape.rs의 write_cur_sz 2곳)만** 플래그가 서면 `0` 출력해 원본 sentinel 복원. **HWP5 저장(control.rs 독립 materialize)·rhwp 렌더는 materialize된 실크기 그대로 사용**(무영향).

변경: `src/model/shape.rs`(+필드), `src/parser/hwpx/section.rs`(플래그 set), `src/serializer/hwpx/picture.rs`·`shape.rs`(플래그 반영).

## 4. 검증
| 항목 | 결과 |
|---|---|
| curSz 충실도 | orig↔rt **완전 일치** (`0×0`×12, `44752×0`×2 복원) |
| 구조 roundtrip | **[PASS] diff=0 r2=0** |
| rhwp 렌더 | orig=rt=**3** (불변) |
| 전체 테스트 | **2945 / 0** (#1389 그림 curSz·roundtrip baseline 포함) |

## 5. 판정
HWPX roundtrip이 curSz=0 sentinel을 정확히 보존 → #1589-계열 IR-invisible 직렬화 드롭 1종 해소. 한글 페이지 delta 확증은 원본 collapse repro(코퍼스 ID 재배정으로 드리프트) + 한글 COM 오라클 필요(별도). 충실도 개선은 roundtrip이 입력을 보존해야 한다는 원칙상 그 자체로 정당.
