#include "gl_state_guard.h"
#include "texture_state.h"
#include <iterator>

namespace rhwp_flutter {
namespace {
constexpr GLenum kCommonEnables[] = {GL_BLEND, GL_CULL_FACE, GL_DEPTH_TEST, GL_STENCIL_TEST,
    GL_SCISSOR_TEST, GL_DITHER, GL_RASTERIZER_DISCARD, GL_SAMPLE_ALPHA_TO_COVERAGE,
    GL_SAMPLE_COVERAGE, GL_POLYGON_OFFSET_FILL};
constexpr GLenum kDesktopEnables[] = {GL_FRAMEBUFFER_SRGB, GL_MULTISAMPLE,
    GL_PRIMITIVE_RESTART, GL_SAMPLE_ALPHA_TO_ONE, GL_TEXTURE_CUBE_MAP_SEAMLESS, GL_DEPTH_CLAMP};
constexpr GLenum kTextureTargets[] = {GL_TEXTURE_1D, GL_TEXTURE_2D, GL_TEXTURE_3D,
    GL_TEXTURE_CUBE_MAP, GL_TEXTURE_1D_ARRAY, GL_TEXTURE_2D_ARRAY,
    GL_TEXTURE_RECTANGLE, GL_TEXTURE_BUFFER, GL_TEXTURE_2D_MULTISAMPLE,
    GL_TEXTURE_2D_MULTISAMPLE_ARRAY};
constexpr GLenum kTextureBindings[] = {GL_TEXTURE_BINDING_1D, GL_TEXTURE_BINDING_2D,
    GL_TEXTURE_BINDING_3D, GL_TEXTURE_BINDING_CUBE_MAP, GL_TEXTURE_BINDING_1D_ARRAY,
    GL_TEXTURE_BINDING_2D_ARRAY, GL_TEXTURE_BINDING_RECTANGLE, GL_TEXTURE_BINDING_BUFFER,
    GL_TEXTURE_BINDING_2D_MULTISAMPLE, GL_TEXTURE_BINDING_2D_MULTISAMPLE_ARRAY};
}  // namespace

thread_local GlStateGuard* GlStateGuard::active_guard_ = nullptr;

void GlStateGuard::forget_deleted_texture(GLuint name) noexcept {
  if (!name) return;
  for (auto* guard = active_guard_; guard; guard = guard->previous_)
    for (auto& unit : guard->units_)
      for (auto& texture : unit.textures)
        if (GLuint(texture) == name) texture = 0;
}

void GlStateGuard::forget_deleted_framebuffer(GLuint name) noexcept {
  if (!name) return;
  for (auto* guard = active_guard_; guard; guard = guard->previous_) {
    if (GLuint(guard->draw_fbo_) == name) guard->draw_fbo_ = 0;
    if (GLuint(guard->read_fbo_) == name) guard->read_fbo_ = 0;
  }
}

GlStateGuard::GlStateGuard() {
  desktop_ = epoxy_is_desktop_gl();
  const auto version = epoxy_gl_version();
  if ((desktop_ && version < 33) || (!desktop_ && version < 30))
    throw Failure("unsupported", "desktop GL 3.3 or GLES 3 is required for state handoff");
  enable_caps_.assign(std::begin(kCommonEnables), std::end(kCommonEnables));
  if (desktop_) {
    enable_caps_.insert(enable_caps_.end(), std::begin(kDesktopEnables), std::end(kDesktopEnables));
    texture_targets_.assign(std::begin(kTextureTargets), std::end(kTextureTargets));
    texture_bindings_.assign(std::begin(kTextureBindings), std::end(kTextureBindings));
    if (version >= 40) {
      texture_targets_.push_back(GL_TEXTURE_CUBE_MAP_ARRAY);
      texture_bindings_.push_back(GL_TEXTURE_BINDING_CUBE_MAP_ARRAY);
    }
  } else {
    // No desktop-only enums/functions may be issued in an ES context. Capture
    // newer ES core bindings only when that version actually exposes them.
    enable_caps_.push_back(GL_PRIMITIVE_RESTART_FIXED_INDEX);
    texture_targets_ = {GL_TEXTURE_2D, GL_TEXTURE_3D, GL_TEXTURE_CUBE_MAP, GL_TEXTURE_2D_ARRAY};
    texture_bindings_ = {GL_TEXTURE_BINDING_2D, GL_TEXTURE_BINDING_3D,
                        GL_TEXTURE_BINDING_CUBE_MAP, GL_TEXTURE_BINDING_2D_ARRAY};
    if (version >= 31) {
      texture_targets_.push_back(GL_TEXTURE_2D_MULTISAMPLE);
      texture_bindings_.push_back(GL_TEXTURE_BINDING_2D_MULTISAMPLE);
    }
    if (version >= 32) {
      texture_targets_.insert(texture_targets_.end(), {GL_TEXTURE_BUFFER, GL_TEXTURE_CUBE_MAP_ARRAY,
                                                     GL_TEXTURE_2D_MULTISAMPLE_ARRAY});
      texture_bindings_.insert(texture_bindings_.end(), {GL_TEXTURE_BINDING_BUFFER,
          GL_TEXTURE_BINDING_CUBE_MAP_ARRAY, GL_TEXTURE_BINDING_2D_MULTISAMPLE_ARRAY});
    }
  }
  enabled_.resize(enable_caps_.size());
  GLint count;
  glGetIntegerv(GL_MAX_COMBINED_TEXTURE_IMAGE_UNITS, &count);
  if (count < 1 || count > 256) throw Failure("unsupported", "GL texture-unit state exceeds guard budget");
  units_.resize(size_t(count));  // Allocate before changing any GL state.
  glGetIntegerv(GL_MAX_UNIFORM_BUFFER_BINDINGS, &count);
  if (count < 1 || count > 256) throw Failure("unsupported", "GL uniform-binding state exceeds guard budget");
  uniforms_.resize(size_t(count));
  indirect_ = desktop_ ? version >= 40 : version >= 31;
  glGetIntegerv(GL_DRAW_FRAMEBUFFER_BINDING, &draw_fbo_);
  glGetIntegerv(GL_READ_FRAMEBUFFER_BINDING, &read_fbo_);
  glGetIntegerv(GL_RENDERBUFFER_BINDING, &renderbuffer_);
  glGetIntegerv(GL_CURRENT_PROGRAM, &program_);
  glGetIntegerv(GL_VERTEX_ARRAY_BINDING, &vao_);
  glGetIntegerv(GL_ARRAY_BUFFER_BINDING, &array_);
  glGetIntegerv(GL_ELEMENT_ARRAY_BUFFER_BINDING, &element_);
  glGetIntegerv(GL_COPY_READ_BUFFER_BINDING, &copy_read_);
  glGetIntegerv(GL_COPY_WRITE_BUFFER_BINDING, &copy_write_);
  glGetIntegerv(GL_PIXEL_PACK_BUFFER_BINDING, &pack_buffer_);
  glGetIntegerv(GL_PIXEL_UNPACK_BUFFER_BINDING, &unpack_buffer_);
  glGetIntegerv(GL_UNIFORM_BUFFER_BINDING, &uniform_buffer_);
  glGetIntegerv(GL_TRANSFORM_FEEDBACK_BUFFER_BINDING, &transform_buffer_);
  if (indirect_) glGetIntegerv(GL_DRAW_INDIRECT_BUFFER_BINDING, &indirect_buffer_);
  for (size_t binding = 0; binding < uniforms_.size(); ++binding) {
    glGetIntegeri_v(GL_UNIFORM_BUFFER_BINDING, GLuint(binding), &uniforms_[binding].buffer);
    glGetInteger64i_v(GL_UNIFORM_BUFFER_START, GLuint(binding), &uniforms_[binding].start);
    glGetInteger64i_v(GL_UNIFORM_BUFFER_SIZE, GLuint(binding), &uniforms_[binding].size);
  }
  glGetIntegerv(GL_ACTIVE_TEXTURE, &active_);
  for (size_t unit = 0; unit < units_.size(); ++unit) {
    glActiveTexture(GL_TEXTURE0 + GLenum(unit));
    for (size_t binding = 0; binding < texture_bindings_.size(); ++binding)
      glGetIntegerv(texture_bindings_[binding], &units_[unit].textures[binding]);
    glGetIntegerv(GL_SAMPLER_BINDING, &units_[unit].sampler);
  }
  glActiveTexture(GLenum(active_));
  glGetIntegerv(GL_BLEND_SRC_RGB, &blend_src_rgb_);
  glGetIntegerv(GL_BLEND_DST_RGB, &blend_dst_rgb_);
  glGetIntegerv(GL_BLEND_SRC_ALPHA, &blend_src_alpha_);
  glGetIntegerv(GL_BLEND_DST_ALPHA, &blend_dst_alpha_);
  glGetIntegerv(GL_BLEND_EQUATION_RGB, &blend_eq_rgb_);
  glGetIntegerv(GL_BLEND_EQUATION_ALPHA, &blend_eq_alpha_);
  glGetFloatv(GL_BLEND_COLOR, blend_color_);
  glGetFloatv(GL_COLOR_CLEAR_VALUE, clear_color_);
  glGetFloatv(GL_LINE_WIDTH, &line_width_);
  glGetFloatv(GL_POLYGON_OFFSET_FACTOR, &polygon_factor_);
  glGetFloatv(GL_POLYGON_OFFSET_UNITS, &polygon_units_);
  glGetFloatv(GL_SAMPLE_COVERAGE_VALUE, &coverage_);
  glGetBooleanv(GL_SAMPLE_COVERAGE_INVERT, &coverage_invert_);
  if (desktop_) {
    glGetDoublev(GL_DEPTH_RANGE, depth_range_);
    glGetDoublev(GL_DEPTH_CLEAR_VALUE, &clear_depth_);
  } else {
    GLfloat range[2], depth;
    glGetFloatv(GL_DEPTH_RANGE, range);
    glGetFloatv(GL_DEPTH_CLEAR_VALUE, &depth);
    depth_range_[0] = range[0]; depth_range_[1] = range[1]; clear_depth_ = depth;
  }
  glGetIntegerv(GL_STENCIL_CLEAR_VALUE, &clear_stencil_);
  glGetBooleanv(GL_COLOR_WRITEMASK, color_mask_);
  glGetBooleanv(GL_DEPTH_WRITEMASK, &depth_mask_);
  glGetIntegerv(GL_DEPTH_FUNC, &depth_func_);
  glGetIntegerv(GL_CULL_FACE_MODE, &cull_mode_);
  glGetIntegerv(GL_FRONT_FACE, &front_face_);
  glGetIntegerv(GL_VIEWPORT, viewport_);
  glGetIntegerv(GL_SCISSOR_BOX, scissor_);
  if (desktop_) glGetIntegerv(GL_POLYGON_MODE, polygon_mode_);
  glGetIntegerv(GL_PACK_ALIGNMENT, &pack_alignment_);
  glGetIntegerv(GL_UNPACK_ALIGNMENT, &unpack_alignment_);
  glGetIntegerv(GL_PACK_ROW_LENGTH, &pack_row_);
  glGetIntegerv(GL_UNPACK_ROW_LENGTH, &unpack_row_);
  glGetIntegerv(GL_PACK_SKIP_ROWS, &pack_skip_rows_);
  glGetIntegerv(GL_PACK_SKIP_PIXELS, &pack_skip_pixels_);
  glGetIntegerv(GL_UNPACK_SKIP_ROWS, &unpack_skip_rows_);
  glGetIntegerv(GL_UNPACK_SKIP_PIXELS, &unpack_skip_pixels_);
  for (size_t index = 0; index < enabled_.size(); ++index) enabled_[index] = glIsEnabled(enable_caps_[index]);
  constexpr GLenum params[2][7] = {
      {GL_STENCIL_FUNC, GL_STENCIL_REF, GL_STENCIL_VALUE_MASK, GL_STENCIL_WRITEMASK,
       GL_STENCIL_FAIL, GL_STENCIL_PASS_DEPTH_FAIL, GL_STENCIL_PASS_DEPTH_PASS},
      {GL_STENCIL_BACK_FUNC, GL_STENCIL_BACK_REF, GL_STENCIL_BACK_VALUE_MASK, GL_STENCIL_BACK_WRITEMASK,
       GL_STENCIL_BACK_FAIL, GL_STENCIL_BACK_PASS_DEPTH_FAIL, GL_STENCIL_BACK_PASS_DEPTH_PASS}};
  for (size_t face = 0; face < 2; ++face) {
    auto& value = stencil_[face];
    GLint* fields[] = {&value.function, &value.reference, &value.value_mask, &value.write_mask,
                      &value.fail, &value.depth_fail, &value.pass};
    for (size_t field = 0; field < 7; ++field) glGetIntegerv(params[face][field], fields[field]);
  }
  // Register only after construction succeeds; nested guards have the same
  // current callback context and must also forget names deleted inside them.
  previous_ = active_guard_;
  active_guard_ = this;
}

GlStateGuard::~GlStateGuard() {
  glBindFramebuffer(GL_DRAW_FRAMEBUFFER, GLuint(draw_fbo_));
  glBindFramebuffer(GL_READ_FRAMEBUFFER, GLuint(read_fbo_));
  glBindRenderbuffer(GL_RENDERBUFFER, GLuint(renderbuffer_));
  glUseProgram(GLuint(program_));
  glBindVertexArray(GLuint(vao_));
  glBindBuffer(GL_ARRAY_BUFFER, GLuint(array_));
  if (vao_ || !desktop_) glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, GLuint(element_));
  glBindBuffer(GL_COPY_READ_BUFFER, GLuint(copy_read_));
  glBindBuffer(GL_COPY_WRITE_BUFFER, GLuint(copy_write_));
  glBindBuffer(GL_PIXEL_PACK_BUFFER, GLuint(pack_buffer_));
  glBindBuffer(GL_PIXEL_UNPACK_BUFFER, GLuint(unpack_buffer_));
  for (size_t binding = 0; binding < uniforms_.size(); ++binding) {
    const auto& uniform = uniforms_[binding];
    if (uniform.buffer && uniform.size > 0)
      glBindBufferRange(GL_UNIFORM_BUFFER, GLuint(binding), GLuint(uniform.buffer), uniform.start, uniform.size);
    else glBindBufferBase(GL_UNIFORM_BUFFER, GLuint(binding), GLuint(uniform.buffer));
  }
  glBindBuffer(GL_UNIFORM_BUFFER, GLuint(uniform_buffer_));
  glBindBuffer(GL_TRANSFORM_FEEDBACK_BUFFER, GLuint(transform_buffer_));
  if (indirect_) glBindBuffer(GL_DRAW_INDIRECT_BUFFER, GLuint(indirect_buffer_));
  for (size_t unit = 0; unit < units_.size(); ++unit) {
    glActiveTexture(GL_TEXTURE0 + GLenum(unit));
    for (size_t binding = 0; binding < texture_targets_.size(); ++binding)
      glBindTexture(texture_targets_[binding], GLuint(units_[unit].textures[binding]));
    glBindSampler(GLuint(unit), GLuint(units_[unit].sampler));
  }
  glActiveTexture(GLenum(active_));
  glBlendFuncSeparate(GLenum(blend_src_rgb_), GLenum(blend_dst_rgb_), GLenum(blend_src_alpha_), GLenum(blend_dst_alpha_));
  glBlendEquationSeparate(GLenum(blend_eq_rgb_), GLenum(blend_eq_alpha_));
  glBlendColor(blend_color_[0], blend_color_[1], blend_color_[2], blend_color_[3]);
  glClearColor(clear_color_[0], clear_color_[1], clear_color_[2], clear_color_[3]);
  glLineWidth(line_width_);
  glPolygonOffset(polygon_factor_, polygon_units_);
  glSampleCoverage(coverage_, coverage_invert_);
  if (desktop_) {
    glDepthRange(depth_range_[0], depth_range_[1]);
    glClearDepth(clear_depth_);
  } else {
    glDepthRangef(GLfloat(depth_range_[0]), GLfloat(depth_range_[1]));
    glClearDepthf(GLfloat(clear_depth_));
  }
  glClearStencil(clear_stencil_);
  glColorMask(color_mask_[0], color_mask_[1], color_mask_[2], color_mask_[3]);
  glDepthMask(depth_mask_);
  glDepthFunc(GLenum(depth_func_));
  glCullFace(GLenum(cull_mode_));
  glFrontFace(GLenum(front_face_));
  glViewport(viewport_[0], viewport_[1], viewport_[2], viewport_[3]);
  glScissor(scissor_[0], scissor_[1], scissor_[2], scissor_[3]);
  if (desktop_) glPolygonMode(GL_FRONT_AND_BACK, GLenum(polygon_mode_[0]));
  glPixelStorei(GL_PACK_ALIGNMENT, pack_alignment_);
  glPixelStorei(GL_UNPACK_ALIGNMENT, unpack_alignment_);
  glPixelStorei(GL_PACK_ROW_LENGTH, pack_row_);
  glPixelStorei(GL_UNPACK_ROW_LENGTH, unpack_row_);
  glPixelStorei(GL_PACK_SKIP_ROWS, pack_skip_rows_);
  glPixelStorei(GL_PACK_SKIP_PIXELS, pack_skip_pixels_);
  glPixelStorei(GL_UNPACK_SKIP_ROWS, unpack_skip_rows_);
  glPixelStorei(GL_UNPACK_SKIP_PIXELS, unpack_skip_pixels_);
  for (size_t index = 0; index < enabled_.size(); ++index) {
    if (enabled_[index]) glEnable(enable_caps_[index]); else glDisable(enable_caps_[index]);
  }
  for (size_t face = 0; face < 2; ++face) {
    const GLenum which = face ? GL_BACK : GL_FRONT;
    const auto& value = stencil_[face];
    glStencilFuncSeparate(which, GLenum(value.function), value.reference, GLuint(value.value_mask));
    glStencilMaskSeparate(which, GLuint(value.write_mask));
    glStencilOpSeparate(which, GLenum(value.fail), GLenum(value.depth_fail), GLenum(value.pass));
  }
  active_guard_ = previous_;
}
}  // namespace rhwp_flutter
