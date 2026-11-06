#ifndef RHWP_FLUTTER_LINUX_GL_STATE_GUARD_H_
#define RHWP_FLUTTER_LINUX_GL_STATE_GUARD_H_

#include <epoxy/gl.h>
#include <array>
#include <vector>

namespace rhwp_flutter {

// Desktop GL 3.3 state modified by Ganesh and target allocation. Flutter's
// public texture callback supplies the current context; no engine Skia object
// is accessed. Texture-unit capture is bounded and includes sampler bindings.
class GlStateGuard {
 public:
  GlStateGuard();
  ~GlStateGuard();
  GlStateGuard(const GlStateGuard&) = delete;
  GlStateGuard& operator=(const GlStateGuard&) = delete;
 private:
  GLint draw_fbo_, read_fbo_, renderbuffer_, program_, vao_, array_, element_;
  GLint copy_read_, copy_write_, pack_buffer_, unpack_buffer_, active_;
  GLint uniform_buffer_, indirect_buffer_ = 0, transform_buffer_;
  GLint blend_src_rgb_, blend_dst_rgb_, blend_src_alpha_, blend_dst_alpha_;
  GLint blend_eq_rgb_, blend_eq_alpha_, depth_func_, cull_mode_, front_face_;
  GLint pack_alignment_, unpack_alignment_, pack_row_, unpack_row_;
  GLint pack_skip_rows_, pack_skip_pixels_, unpack_skip_rows_, unpack_skip_pixels_;
  GLfloat blend_color_[4], clear_color_[4], line_width_, polygon_factor_, polygon_units_, coverage_;
  GLdouble depth_range_[2], clear_depth_;
  GLint clear_stencil_;
  GLboolean color_mask_[4], depth_mask_, coverage_invert_;
  GLint viewport_[4], scissor_[4], polygon_mode_[2];
  struct Stencil { GLint function, reference, value_mask, write_mask, fail, depth_fail, pass; };
  Stencil stencil_[2];
  struct Unit { std::array<GLint, 10> textures; GLint sampler; };
  std::vector<Unit> units_;
  struct Uniform { GLint buffer; GLint64 start, size; };
  std::vector<Uniform> uniforms_;
  std::array<GLboolean, 16> enabled_;
  bool indirect_;
};
}  // namespace rhwp_flutter
#endif
