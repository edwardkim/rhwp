# #3820 Stage 180 - native RowBreak source-frame owner

## 목적

Stage 179에서 renderer `row_filter` 문단 조각 선택을 원인에서 제외한 뒤,
pagination 단계에서 p4/p18/p35 HWP RowBreak owner가 이동하는 공통 원인을
저장 프레임 좌표로 교정한다.

## 계측 근거

`RHWP_TABLE_DRIFT=1`으로 #3820 HWP 회귀를 단일 스레드 실행했다.

그 전에 #4698 renderer `row_filter` fragment 선택에 `RHWP_TRACE_STORED_FRAGMENT`
진단을 넣어 같은 회귀를 실행했다. 이벤트는 0건이었다. 즉 이 회귀는 renderer의
문단 조각 선택이 아니라 그 이전 pagination owner 결정에서 발생한다.

```text
pi=15: declared=929.0, table_avail=971.3, fn=0.0
  row 0(header)=32.4만 첫 조각에 배치되고 body row는 continuation으로 이월

pi=173: declared=938.0, table_avail=971.3, fn=0.0
pi=347: declared=938.0, table_avail=971.3, fn=0.0
```

세 표는 native HWP5 `RowBreak`이며 저장 앵커와 선언 object frame이 현재 조각
안에 정합한다. 하지만 첫 조각 allowance는 empty host와 rowspan이라는 table
topology 조건도 요구해 모두 0이 되었다. 이 조건 때문에 저장 first-frame body가
header-only 조각으로 밀려 p4/p18/p35의 owner가 함께 어긋났다.

## 수정

`source_first_fragment_overflow_allowance`의 자격을 실제 물리 source frame으로
한정했다.

- native HWP5, non-TAC, `RowBreak`, 첫 조각, 각주 없음
- row geometry table과 현재 table의 동일성
- 현재 흐름과 일치하는 비합성 저장 앵커
- `anchor + declared object height`가 현재 fragment scan bottom 안에 있음

empty host와 rowspan은 문서 형상일 뿐 first physical frame의 소유 증거가 아니므로
제거했다. allowance 값 자체는 선언 bottom과 scan bottom의 측정 차이로 계산하며,
문서 크기, 표 차원, 문단 수, 페이지 번호, 고정 pixel tail을 사용하지 않는다.

Stage 179의 `RHWP_TRACE_STORED_FRAGMENT` 무동작 진단은 같은 수정에서 제거했다.

## 검증

```text
cargo test --profile release-test --target-dir target/task-3820-stage168 \
  --test issue_3820_rowbreak_rowspan_band

PASS: 4 passed
```

- p4 첫 저장 body fragment
- p18/p19 short rowspan owner
- p35/p36 blank-tail band
- p35 control-only nested table

## 다음 스테이지

코드 수정 없이 새 산출물로 다음 최종 게이트를 실행한다.

1. 두 실제 2025 편람의 HWP/HWPX 383쪽
2. `issue_3930_hwpx_hwp_save_layout`
3. `wasm-pack build --target web --out-dir pkg`
4. `npm run e2e:issue-3820`

## 상태

HWP RowBreak 회귀 수정 완료. 전체 완료 판정은 다음 검증 스테이지에 보류한다.
