varying vec2 v_pos;
varying vec4 v_color;
varying float v_circle;

#ifdef VERTEX_SHADER
uniform mat3 u_view_matrix;
uniform mat3 u_projection_matrix;

attribute vec2 a_pos;
attribute vec2 a_vt;
attribute vec4 i_color;
attribute float i_circle;

attribute mat3 i_model_matrix;

void main() {
    v_circle = i_circle;
    v_color = i_color;
    v_pos = a_pos;
    vec3 pos = i_model_matrix * vec3(a_pos, 1.0);
    pos = u_projection_matrix * u_view_matrix * pos;
    gl_Position = vec4(pos.xy, 0.0, pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
void main() {
    if (v_circle > 0.0 && length(v_pos) > 1.0) {
        discard;
    }
    gl_FragColor = v_color;
}
#endif
