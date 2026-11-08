use super::{element_max_x, element_max_y, Node, Rect, SVGPlayer};
use crate::wmf::imports::BTreeMap;

/// 클립을 지정한 시점의 창 변환으로 고정한 장치 좌표다.
/// 뒤의 창 원점 변경이나 DC 복원이 이미 발행한 그림의 클립을 다시 옮기지 않는다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct DeviceClipBounds {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

impl DeviceClipBounds {
    fn intersect(self, other: Self) -> Self {
        let left = self.left.max(other.left);
        let top = self.top.max(other.top);
        Self {
            left,
            top,
            right: self.right.min(other.right).max(left),
            bottom: self.bottom.min(other.bottom).max(top),
        }
    }

    fn is_empty(self) -> bool {
        self.left >= self.right || self.top >= self.bottom
    }
}

impl SVGPlayer {
    pub(super) fn install_device_clip(&mut self, record: usize, rect: &Rect, intersect: bool) {
        let (x1, y1) = self
            .context_current
            .window
            .to_device(f32::from(rect.left), f32::from(rect.top));
        let (x2, y2) = self
            .context_current
            .window
            .to_device(f32::from(rect.right), f32::from(rect.bottom));
        let mut bounds = DeviceClipBounds {
            left: x1.min(x2),
            top: y1.min(y2),
            right: x1.max(x2),
            bottom: y1.max(y2),
        };
        if intersect {
            if let Some(previous) = self
                .current_clip_id
                .as_ref()
                .and_then(|id| self.clip_bounds.get(id))
            {
                bounds = previous.intersect(bounds);
            }
        }
        if self
            .current_clip_id
            .as_ref()
            .and_then(|id| self.clip_bounds.get(id))
            == Some(&bounds)
        {
            // 같은 영역의 재지정은 ROP의 동일 클립 신원도 유지한다.
            return;
        }
        let id = format!("clip{record}");
        let rectangle = Node::new("rect")
            .set("x", bounds.left)
            .set("y", bounds.top)
            .set("width", bounds.right - bounds.left)
            .set("height", bounds.bottom - bounds.top);
        self.definitions.push(
            Node::new("clipPath")
                .set("id", &id)
                .set("clipPathUnits", "userSpaceOnUse")
                .add(rectangle),
        );
        self.clip_bounds.insert(id.clone(), bounds);
        self.current_clip_id = Some(id);
    }
}

/// 클립이 없는 요소는 기존 자동 확장을 유지한다. 있는 요소는 같은 클립의
/// 실제 표시 부분만 외곽에 포함하며, 파일의 원래 그림 좌표·크기는 바꾸지 않는다.
pub(super) fn visible_element_maxima(
    node: &Node,
    clips: &BTreeMap<String, DeviceClipBounds>,
) -> (Option<i32>, Option<i32>) {
    let x = element_max_x(node);
    let y = element_max_y(node);
    let clip = node
        .shared_clip_reference()
        .and_then(|reference| reference.strip_prefix("url(#"))
        .and_then(|reference| reference.strip_suffix(')'))
        .and_then(|id| clips.get(id));
    let Some(clip) = clip else {
        return (x, y);
    };
    let left = node.attr("x").and_then(|x| x.parse::<f32>().ok());
    let top = node.attr("y").and_then(|y| y.parse::<f32>().ok());
    if clip.is_empty()
        || x.is_some_and(|right| right as f32 <= clip.left)
        || y.is_some_and(|bottom| bottom as f32 <= clip.top)
        || left.is_some_and(|left| left >= clip.right)
        || top.is_some_and(|top| top >= clip.bottom)
    {
        return (None, None);
    }
    (
        x.map(|right| right.min(clip.right.ceil() as i32)),
        y.map(|bottom| bottom.min(clip.bottom.ceil() as i32)),
    )
}
