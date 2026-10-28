# Task #1587 — Stage 1 완료보고서

**단계**: Ruby 드롭 재현 테스트 (RED)
**브랜치**: `local/task1587`

## 작업 내용

`src/serializer/hwpx/mod.rs` 테스트 모듈에 회귀 가드 추가:
`task1587_ruby_control_roundtrips`.

- `Control::Ruby{ruby_text:"덧말", ..Default::default()}` 를 둔 최소 Document 구성
  (`..Default::default()` 사용 → Stage 2 모델 필드 확장에 견고).
- `serialize_hwpx → parse_hwpx` roundtrip 후 첫 문단 controls 의 Ruby 보존 + ruby_text 검증.

## 결과 (RED 확인)

```
assertion failed: Ruby 컨트롤이 roundtrip 후 보존돼야 한다 (현재 드롭)
  controls = [SectionDef, ColumnDef]   ← 템플릿 자동 주입, Ruby 소실
  left: 0  right: 1
```

- **현재 코드: Ruby 드롭** — `is_hwpx_inline_slot` 에 등록(인식)됐으나 `render_control_slot`
  방출 arm 부재로 `_ => {}` 처리. 근본원인(구현 계획서 §1) 일치.

## 그라운딩 재확인

실문서 `36389301` 문단 0.6 dump: cc=59, text_len=50 → (59−1−50)/8 = **1 슬롯**.
ruby subText="전술훈련 30% + 현지훈련 20%". mainText("팀단위 훈련")는 para.text 에 **부재**
→ 모델 손실 구조(mainText 미보존) 입증.

## 다음 단계

Stage 2 — 모델(`control.rs` Ruby 필드 확장) + 파서(`parse_dutmal` 전 속성/요소 보존).
