---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-17
name: 작업지시자 정체성 + 역할
description: edwardkim — rhwp 프로젝트 메인테이너 + Rust/WASM HWP 뷰어 단독 설계자
type: user
originSessionId: 4861649d-834a-43c6-a262-9f08333360e8
---
**GitHub**: edwardkim / **이메일**: tangokorea@gmail.com
**프로젝트**: rhwp — Rust + WebAssembly 기반 HWP/HWPX 오픈소스 뷰어/에디터 ("All HWP, Open for Everyone" / "알(R), 모두의 한글")
**현재 버전 시점**: v0.7.9 (2026-05-05 기준), 다음 patch (v0.7.10 또는 v0.8.0) 또는 v1.0.0 진입 준비 중

## 환경
- 주 환경: Windows + WSL2 (Ubuntu) + 한컴 2010 + 한컴 2022 (Windows = 시각 판정 정답지)
- 별도 환경: macOS (`ios/devel` 전용, 알한글 iPad/iPhone 앱 — `alhangeul-macos` 코드네임)
- 저장소: `/home/edward/mygithub/rhwp/` (Linux), `/Users/edwardkim/vspace/rhwp/` (Mac)

## 비전
- **AI X (AI Experience)** — 인간과 AI 사이 정보 문서화
- **공공 자산화** (v3.0.0): 한컴 대등 + 공공기관 실무 투입
- **혼자 → 함께** (v0.5.0까지 단독, v1.0.0+부터 커뮤니티 개방)
- **닫힌 포맷의 벽 깨기** — 한컴이 30년간 안 푼 오픈소스 HWP 도전

## 배포 채널 (6+)
npm @rhwp/core / @rhwp/editor / GitHub Pages / VS Code Marketplace / Open VSX / Chrome Web Store / Edge Add-ons / AMO (Firefox)

## 프로젝트 분리 정책
- `ios/devel` — 맥북 전용 iOS 앱 (장기 분리, 추후 통합)
- `skia/devel` — 별도 브랜치 운영 검토 중
