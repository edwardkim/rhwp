//! 렌더링 스케줄러 (Observer + Worker 패턴)
//!
//! - **RenderObserver**: 뷰포트 변경, 줌 변경, 문서 변경 등의 이벤트를 감지
//! - **RenderWorker**: 렌더링 작업을 우선순위에 따라 실행
//! - **RenderScheduler**: Observer와 Worker를 연결하여 효율적인 렌더링을 조율

use super::render_tree::{BoundingBox, PageRenderTree};

/// 렌더링 작업 우선순위
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderPriority {
    /// 즉시 렌더링 (현재 뷰포트 내 페이지)
    Immediate = 0,
    /// 사전 렌더링 (뷰포트 인접 페이지)
    Prefetch = 1,
    /// 백그라운드 렌더링 (나머지 페이지)
    Background = 2,
}

/// 렌더링 작업 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// 대기 중
    Pending,
    /// 렌더링 중
    InProgress,
    /// 완료
    Completed,
    /// 취소됨
    Cancelled,
}

/// 렌더링 작업
#[derive(Debug)]
pub struct RenderTask {
    /// 작업 ID
    pub id: u32,
    /// 대상 페이지 인덱스
    pub page_index: u32,
    /// 우선순위
    pub priority: RenderPriority,
    /// 작업 상태
    pub status: TaskStatus,
}

impl RenderTask {
    pub fn new(id: u32, page_index: u32, priority: RenderPriority) -> Self {
        Self {
            id,
            page_index,
            priority,
            status: TaskStatus::Pending,
        }
    }
}

/// 뷰포트 정보
#[derive(Debug, Clone, Default)]
pub struct Viewport {
    /// 뷰포트 X 오프셋 (스크롤 위치)
    pub scroll_x: f64,
    /// 뷰포트 Y 오프셋 (스크롤 위치)
    pub scroll_y: f64,
    /// 뷰포트 폭
    pub width: f64,
    /// 뷰포트 높이
    pub height: f64,
    /// 줌 배율 (1.0 = 100%)
    pub zoom: f64,
}

impl Viewport {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            scroll_x: 0.0,
            scroll_y: 0.0,
            width,
            height,
            zoom: 1.0,
        }
    }

    /// 뷰포트 영역을 BoundingBox로 반환
    pub fn as_bbox(&self) -> BoundingBox {
        BoundingBox::new(self.scroll_x, self.scroll_y, self.width, self.height)
    }
}

/// 렌더 옵져버 이벤트 종류
#[derive(Debug, Clone)]
pub enum RenderEvent {
    /// 뷰포트 변경 (스크롤, 리사이즈)
    ViewportChanged(Viewport),
    /// 줌 변경
    ZoomChanged(f64),
    /// 문서 내용 변경 (특정 페이지)
    ContentChanged(u32),
    /// 전체 무효화 요청
    InvalidateAll,
}

/// 렌더 옵져버 트레이트
///
/// 뷰포트, 줌, 문서 변경 등의 이벤트를 감지하여
/// 스케줄러에 렌더링 작업을 요청한다.
pub trait RenderObserver {
    /// 이벤트 발생 시 호출
    fn on_event(&mut self, event: &RenderEvent);

    /// 현재 뷰포트 내에서 보이는 페이지 인덱스 목록 반환
    fn visible_pages(&self) -> Vec<u32>;

    /// 프리페치 대상 페이지 인덱스 목록 반환
    fn prefetch_pages(&self) -> Vec<u32>;
}

/// 렌더 워커 트레이트
///
/// 실제 렌더링 작업을 수행한다.
/// 백엔드(Canvas/SVG/HTML)에 따라 구현이 다르다.
pub trait RenderWorker {
    /// 한 페이지의 렌더 트리를 렌더링한다.
    fn render_page(&mut self, tree: &PageRenderTree) -> Result<(), RenderError>;

    /// 렌더링 결과를 캐시에서 가져온다.
    fn get_cached(&self, page_index: u32) -> Option<&PageRenderTree>;

    /// 캐시를 무효화한다.
    fn invalidate_cache(&mut self, page_index: u32);
}

/// 렌더링 오류
#[derive(Debug)]
pub enum RenderError {
    /// 렌더 트리가 비어있음
    EmptyTree,
    /// 백엔드 오류
    BackendError(String),
    /// 캐시 미스
    CacheMiss(u32),
}

/// 렌더링 스케줄러
///
/// Observer가 감지한 이벤트를 기반으로 RenderTask를 생성하고
/// 우선순위에 따라 Worker에게 작업을 배분한다.
pub struct RenderScheduler {
    /// 작업 큐
    task_queue: Vec<RenderTask>,
    /// 다음 작업 ID
    next_task_id: u32,
    /// 현재 뷰포트
    viewport: Viewport,
    /// 페이지별 Y 오프셋 (연속 스크롤용)
    page_offsets: Vec<f64>,
    /// 페이지 간 간격 (px)
    page_gap: f64,
    /// 프리페치 범위 (뷰포트 위/아래 페이지 수)
    prefetch_range: u32,
    /// 총 페이지 수
    total_pages: u32,
}

impl RenderScheduler {
    pub fn new(total_pages: u32) -> Self {
        Self {
            task_queue: Vec::new(),
            next_task_id: 0,
            viewport: Viewport::default(),
            page_offsets: Vec::new(),
            page_gap: 10.0,
            prefetch_range: 2,
            total_pages,
        }
    }

    /// 페이지 높이 목록으로 오프셋 계산
    pub fn set_page_heights(&mut self, heights: &[f64]) {
        self.page_offsets.clear();
        let mut offset = 0.0;
        for &h in heights {
            self.page_offsets.push(offset);
            offset += h + self.page_gap;
        }
    }

    /// 뷰포트 업데이트 및 작업 스케줄링
    pub fn update_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
        self.schedule_visible_pages();
    }

    /// 줌 변경 처리
    pub fn update_zoom(&mut self, zoom: f64) {
        self.viewport.zoom = zoom;
        // 줌 변경 시 모든 캐시를 무효화하고 재스케줄링
        self.cancel_all_tasks();
        self.schedule_visible_pages();
    }

    /// 특정 페이지 무효화
    pub fn invalidate_page(&mut self, page_index: u32) {
        // 기존 해당 페이지 작업 취소
        for task in &mut self.task_queue {
            if task.page_index == page_index {
                task.status = TaskStatus::Cancelled;
            }
        }

        // 보이는 페이지면 즉시 렌더링 예약
        let priority = self.page_priority(page_index);
        self.enqueue_task(page_index, priority);
    }

    /// 현재 뷰포트에서 보이는 페이지 인덱스 목록
    pub fn visible_pages(&self) -> Vec<u32> {
        if self.page_offsets.is_empty() {
            return if self.total_pages > 0 {
                vec![0]
            } else {
                vec![]
            };
        }

        let vp_top = self.viewport.scroll_y;
        let vp_bottom = vp_top + self.viewport.height;

        let mut visible = Vec::new();
        for (i, &offset) in self.page_offsets.iter().enumerate() {
            let page_bottom = if i + 1 < self.page_offsets.len() {
                self.page_offsets[i + 1] - self.page_gap
            } else {
                offset + self.viewport.height // 마지막 페이지 추정
            };

            if offset < vp_bottom && page_bottom > vp_top {
                visible.push(i as u32);
            }
        }

        visible
    }

    /// 다음에 처리할 작업 가져오기 (우선순위 순)
    pub fn next_task(&mut self) -> Option<&RenderTask> {
        // Pending 작업 중 우선순위가 가장 높은 것
        self.task_queue.sort_by_key(|t| t.priority);
        self.task_queue
            .iter()
            .find(|t| t.status == TaskStatus::Pending)
    }

    /// 작업 완료 마킹
    pub fn complete_task(&mut self, task_id: u32) {
        if let Some(task) = self.task_queue.iter_mut().find(|t| t.id == task_id) {
            task.status = TaskStatus::Completed;
        }
    }

    /// 완료/취소된 작업 정리
    pub fn cleanup_tasks(&mut self) {
        self.task_queue
            .retain(|t| t.status == TaskStatus::Pending || t.status == TaskStatus::InProgress);
    }

    /// 대기 중인 작업 수
    pub fn pending_count(&self) -> usize {
        self.task_queue
            .iter()
            .filter(|t| t.status == TaskStatus::Pending)
            .count()
    }

    // --- 내부 메서드 ---

    /// 보이는 페이지와 프리페치 페이지를 스케줄링
    fn schedule_visible_pages(&mut self) {
        let visible = self.visible_pages();

        // 1) 보이는 페이지: Immediate
        for &page_idx in &visible {
            if !self.has_pending_task(page_idx) {
                self.enqueue_task(page_idx, RenderPriority::Immediate);
            }
        }

        // 2) 프리페치 페이지
        for &page_idx in &visible {
            let start = page_idx.saturating_sub(self.prefetch_range);
            let end = (page_idx + self.prefetch_range + 1).min(self.total_pages);
            for p in start..end {
                if !visible.contains(&p) && !self.has_pending_task(p) {
                    self.enqueue_task(p, RenderPriority::Prefetch);
                }
            }
        }
    }

    /// 작업 큐에 추가
    fn enqueue_task(&mut self, page_index: u32, priority: RenderPriority) {
        let id = self.next_task_id;
        self.next_task_id += 1;
        self.task_queue
            .push(RenderTask::new(id, page_index, priority));
    }

    /// 특정 페이지에 대기 중인 작업이 있는지 확인
    fn has_pending_task(&self, page_index: u32) -> bool {
        self.task_queue
            .iter()
            .any(|t| t.page_index == page_index && t.status == TaskStatus::Pending)
    }

    /// 모든 대기 작업 취소
    fn cancel_all_tasks(&mut self) {
        for task in &mut self.task_queue {
            if task.status == TaskStatus::Pending {
                task.status = TaskStatus::Cancelled;
            }
        }
    }

    /// 페이지의 렌더링 우선순위 결정
    fn page_priority(&self, page_index: u32) -> RenderPriority {
        let visible = self.visible_pages();
        if visible.contains(&page_index) {
            RenderPriority::Immediate
        } else {
            // 인접 페이지이면 Prefetch
            let near_visible = visible
                .iter()
                .any(|&v| page_index.abs_diff(v) <= self.prefetch_range);
            if near_visible {
                RenderPriority::Prefetch
            } else {
                RenderPriority::Background
            }
        }
    }
}

impl RenderObserver for RenderScheduler {
    fn on_event(&mut self, event: &RenderEvent) {
        match event {
            RenderEvent::ViewportChanged(vp) => {
                self.update_viewport(vp.clone());
            }
            RenderEvent::ZoomChanged(zoom) => {
                self.update_zoom(*zoom);
            }
            RenderEvent::ContentChanged(page_idx) => {
                self.invalidate_page(*page_idx);
            }
            RenderEvent::InvalidateAll => {
                self.cancel_all_tasks();
                self.schedule_visible_pages();
            }
        }
    }

    fn visible_pages(&self) -> Vec<u32> {
        self.visible_pages()
    }

    fn prefetch_pages(&self) -> Vec<u32> {
        let visible = self.visible_pages();
        let mut prefetch = Vec::new();
        for &page_idx in &visible {
            let start = page_idx.saturating_sub(self.prefetch_range);
            let end = (page_idx + self.prefetch_range + 1).min(self.total_pages);
            for p in start..end {
                if !visible.contains(&p) && !prefetch.contains(&p) {
                    prefetch.push(p);
                }
            }
        }
        prefetch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_priority_order() {
        assert!(RenderPriority::Immediate < RenderPriority::Prefetch);
        assert!(RenderPriority::Prefetch < RenderPriority::Background);
    }

    #[test]
    fn test_render_task_creation() {
        let task = RenderTask::new(0, 5, RenderPriority::Immediate);
        assert_eq!(task.page_index, 5);
        assert_eq!(task.status, TaskStatus::Pending);
    }

    #[test]
    fn test_viewport() {
        let vp = Viewport::new(800.0, 600.0);
        assert!((vp.width - 800.0).abs() < 0.01);
        assert!((vp.zoom - 1.0).abs() < 0.01);
        let bbox = vp.as_bbox();
        assert!((bbox.width - 800.0).abs() < 0.01);
    }

    #[test]
    fn test_scheduler_creation() {
        let scheduler = RenderScheduler::new(10);
        assert_eq!(scheduler.pending_count(), 0);
        assert_eq!(scheduler.total_pages, 10);
    }

    #[test]
    fn test_scheduler_visible_pages() {
        let mut scheduler = RenderScheduler::new(5);
        // 각 페이지 높이 1000px, 간격 10px
        scheduler.set_page_heights(&[1000.0, 1000.0, 1000.0, 1000.0, 1000.0]);
        scheduler.update_viewport(Viewport {
            scroll_x: 0.0,
            scroll_y: 0.0,
            width: 800.0,
            height: 600.0,
            zoom: 1.0,
        });
        let visible = RenderScheduler::visible_pages(&scheduler);
        assert!(visible.contains(&0));
    }

    #[test]
    fn test_scheduler_enqueue_on_viewport_change() {
        let mut scheduler = RenderScheduler::new(5);
        scheduler.set_page_heights(&[1000.0, 1000.0, 1000.0, 1000.0, 1000.0]);
        scheduler.update_viewport(Viewport::new(800.0, 600.0));
        // 보이는 페이지에 대한 Immediate 작업이 생겨야 함
        assert!(scheduler.pending_count() > 0);
    }

    #[test]
    fn test_scheduler_task_lifecycle() {
        let mut scheduler = RenderScheduler::new(3);
        scheduler.set_page_heights(&[500.0, 500.0, 500.0]);
        scheduler.update_viewport(Viewport::new(800.0, 600.0));

        // 작업 가져오기
        let task_id = scheduler.next_task().map(|t| t.id);
        assert!(task_id.is_some());

        // 작업 완료
        scheduler.complete_task(task_id.unwrap());

        // 정리
        let before = scheduler.task_queue.len();
        scheduler.cleanup_tasks();
        // 완료된 작업은 제거됨
        assert!(
            scheduler.task_queue.len() < before
                || scheduler
                    .task_queue
                    .iter()
                    .all(|t| t.status != TaskStatus::Completed)
        );
    }

    #[test]
    fn test_scheduler_invalidate_page() {
        let mut scheduler = RenderScheduler::new(5);
        scheduler.set_page_heights(&[1000.0; 5]);
        scheduler.update_viewport(Viewport::new(800.0, 600.0));
        let _initial_count = scheduler.pending_count();

        scheduler.invalidate_page(0);
        // 무효화 후 다시 작업이 추가됨
        assert!(scheduler.pending_count() > 0);
    }

    #[test]
    fn test_observer_trait() {
        let mut scheduler = RenderScheduler::new(3);
        scheduler.set_page_heights(&[500.0, 500.0, 500.0]);

        // Observer 트레이트로 이벤트 처리
        let event = RenderEvent::ViewportChanged(Viewport::new(800.0, 600.0));
        scheduler.on_event(&event);
        assert!(scheduler.pending_count() > 0);

        // 줌 변경
        let event = RenderEvent::ZoomChanged(1.5);
        scheduler.on_event(&event);

        // 전체 무효화
        let event = RenderEvent::InvalidateAll;
        scheduler.on_event(&event);
    }
}
