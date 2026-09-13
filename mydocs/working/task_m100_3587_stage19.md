# #3587 Stage 19 — D1 글상자 지원 경계 조사

- 승인: Stage 18 시각 판정 「성공입니다」 이후 「다음 절차를 진행하세요」.
- 시작: `99ba905e6`, 제품 기준 `cc9f49a8f` 유지.
- 목표: `samples/table-in-tbox.hwp` pi4의 거부 원인을 포맷·기존 저장 계약·실제 바이트로 분리한다.
- 비범위: 미해석 raw 삭제, 내부 컨트롤 제거, renderer 수정, D2 연결, 원격 게시.

## 조사 순서

1. 파서·IR·직렬화기에서 LIST_HEADER tail의 범위와 이력을 확인한다.
2. 원본 pi4의 tail 및 내부 컨트롤을 덤프하고 지원 검사와 대조한다.
3. 알려진 저장 계약으로 안전하게 허용할 수 있는 구조와 추가 해석이 필요한 구조를 구분한다.
4. 근거와 권고 구현 범위를 기록한다. 단일 샘플의 바이트 길이만으로 허용 범위를 넓히지 않는다.

## 결론 — 서로 다른 두 지원 누락

### 1. 기본 글상자 헤더를 과도하게 거부함

실제 pi4의 `raw_list_header_extra`는 **정확히 13바이트의 0**이다.
파서는 LIST_HEADER 앞 20바이트(문단 수, 속성, 여백, 폭)를 의미 필드로 읽고 이후를 이 필드에 보존한다.
이름이 raw/extra라는 사실만으로 참조 번호가 들어 있는 미해석 확장이라고 단정할 수 없다.

근거를 다음 순서로 대조했다.

- HWP 5.0 revision1.3 표 65·89·90은 목록 및 여백/폭 구조를 설명하지만 이 13바이트 전체를 설명하지 않는다.
  공식 스펙에 13바이트 의미가 모두 명시됐다고 주장하지 않는다.
- `79c074c36e`(#1058, 2026-05-21)에서 저장기의 기본 tail 누락을 보정했다.
  [당시 조사](archives/task_m100_1058_stage1.md)와 현재 serializer는
  예약 8바이트 + 양식 편집 플래그 4바이트 + 이름 존재 플래그 1바이트를 기록한다.
- 당시 참조했던 로컬 hwplib `ForTextBox.java`도 직접 확인했다.
  `writeZero(8)`, `editableAtFormMode` 0/1, `fieldNameFlag` 0/0xff와 선택적 ParameterSet을 쓴다.
  위치: `/home/edward/vsworks/shwp/hwplib/src/main/java/kr/dogfoot/hwplib/writer/bodytext/paragraph/control/gso/part/ForTextBox.java`.
- 현재 `serialize_text_box_if_present`도 HWP raw는 보존하고 raw 없는 HWPX 출처에는 13개의 0을 작성한다.
  `tests/issue_1058_textbox_list_header.rs`는 같은 `table-in-tbox.hwp`의 33바이트 헤더 보존을 검사한다.
- 이번 #3587의 `c32c8475d`(2026-09-12)가 추가한 `Scan::shape`는 tail이 빈 경우만 허용한다.
  따라서 기존 저장 계약을 반영하지 못한 **신규 strict 가져오기 지원 검사의 누락**이다.
  기존 일반 열기·저장 기능이 이번에 손상됐다는 의미의 회귀와는 구분한다.

기본 13바이트 허용은 `len == 13`만 검사하면 안 된다. 현재 양 형식에서 의미 보존 근거가 있는
예약 0·양식 편집 0·이름 없음의 기본 구조를 확인하고 원본 바이트를 보존해야 한다.
비기본 양식 편집 값·이름 ParameterSet·미지 확장까지 허용하는 것은 별도 계약이 필요하다.

### 2. 글상자 안 하이퍼링크 필드는 현재 승인 범위 밖

pi4의 글상자는 문단 21개이며 재귀적으로 표 4개·그림 8개·Field 1개를 포함한다.
Field는 글상자 내부 pi20의 `FieldType::Hyperlink`다. 최상위 `Control::Hyperlink`와 구분해야 한다.
원본의 링크 주소는 덤프에 보존했지만 주소를 열거나 네트워크 요청을 하지 않았다.

- `field_id=2014994988`, `raw_type=null`, 별도 ParameterSet 없음, CTRL_DATA는 null.
- 문자 범위는 `[13,43)`, `control_idx=0`, `end_field_id=0`. 원본 시작/끝 ID 표현을
  재채번에서 어떻게 보존할지도 확인 대상이다. 0을 무조건 새 시작 ID로 바꾸면 안 된다.
- 현재 `Scan::control`은 plain ClickHere(누름틀)만 허용하므로 해당 Field는 조건상 거부 대상이다.
  이는 정적 확인이며, 원본에서 tail을 삭제해 실제 두 번째 오류를 억지로 실행한 결과가 아니다.

따라서 **tail 보정만으로 이 글상자 전체의 성공을 약속할 수 없다**.
링크를 삭제하거나 마지막 문단을 제외하여 실물 성공 사례로 제시하지 않는다.

## 실행 증거

제품 기준 `cc9f49a8f`는 그대로다. 새 source는 읽기 전용 ignored 진단
`tests/cases/issue_3587_textbox_import_probe.rs`이며 문서 수정/저장을 실행하지 않고 pi4의 IR JSON만 기록한다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
rg -l 'cases/issue_3587_textbox_import_probe.rs' tests/generated/regression_suite_*.rs
RHWP_3587_TEXTBOX_OUTPUT=/home/edward/mygithub/rhwp/output/3587/d1-textbox/pi4-source.json \
cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review \
  --test regression_suite_001 --run-ignored only -E 'test(dump_real_textbox_import_source)'
```

review cwd `/home/edward/mygithub/rhwp-review-3587`, run `29a4a81e-fe56-4ffd-ad79-7b106f76167a`:
**진단 1 PASS**. 이는 가져오기 통과가 아니라 원본 덤프 생성 성공이다.
원본 SHA-256은 Stage 18과 동일하다.
진단 source SHA-256: `6ebf25abecc32cdeb1957627a56c10ebd6276557ee5274a4d35078e3a695941c`.
JSON SHA-256: `7b44eee53f099d8aef887de04bac645d71a99df72a03c94b1d5b468e0f7256b4`.
`cargo fmt --all -- --check`, manifest `--check`, `git diff --check` PASS.
기존 #1058 테스트는 코드를 읽었으며 이번 절편에서 재실행하지 않았다.

## 권고 및 승인 경계

동일 #3587에서 다음 묶음으로 진행할 것을 권고한다.

1. B/D 공통 지원 검사에 기존 저장 계약의 기본 글상자 tail을 반영한다.
   HWPX→HWP 재열기에서도 같은 기본 글상자가 다시 거부되지 않도록 계약 테스트를 둔다.
2. 기존 plain ClickHere 한정을 **일반 Hyperlink 필드까지 명시적으로 확대하는 수정 계획 승인을 받는다**.
   A의 ID·필드 범위 매핑과 D 자원 준비를 재사용하고, 주소는 데이터로 보존할 뿐 실행/접속하지 않는다.
   memo·unknown raw·매개변수 확장 등의 거부 및 실패 원자성은 유지한다.
3. 원본 pi4 전체를 변경 없이 가져오고 표 4개·그림 8개·링크 및 표시 문자열, HWP/HWPX 재열기를 검증한다.
   두 번 가져온 ID/범위 독립성, 잘린 필드 범위 거부, 앞뒤 문단과 용지 설정 보존도 확인한다.
4. 한컴 PDF와 메인테이너 시각 판정 후 D1 종료 검토로 돌아간다.

이 단계에서는 제품 코드를 바꾸지 않았다. D2·원격 push·PR·새 이슈 생성도 수행하지 않았다.
