---
name: 알한글 iOS 프로젝트
description: iPad HWP 학습 도구 — 맥북에서만 진행, 리눅스 환경과 무관
type: project
originSessionId: 1f035a49-cf55-4427-a5b6-ba6a493aa832
---
**프로젝트**: 알한글 — iPad HWP 학습 도구 (GitHub Projects #2)
**브랜치**: `ios/devel`
**작업 환경**: 맥북 전용 (aarch64-apple-ios 빌드 필요, 리눅스에서 빌드 불가)

**Why:** iOS/iPadOS 네이티브 개발로 전환 결정. 리눅스(현재 WSL2 환경)에서는 빌드가 안 되므로 이 환경에서는 연결하지 않음.

**How to apply:** 이 환경(리눅스/WSL2)에서 ios/devel 브랜치나 알한글 관련 작업 요청이 오면 맥북에서 진행해야 함을 안내한다. rhwp 메인 작업과 별개로 관리.

**진행 상황** (2026-04-12 기준):
- Done: #90 크로스컴파일, #91 SwiftUI+Rust FFI, #92 최소 뷰어앱, #93 Core Graphics/Metal 렌더러
- Todo: #87 전체 앱, #94 Apple Pencil, #95 .rhwp 포맷, #96 레이어 UI, #97 학습 워크플로우, #98 AI 채점, #99 App Store 출시
