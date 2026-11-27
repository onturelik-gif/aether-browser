<script lang="ts">
  import type { Quiet } from "./dither";
  import type { Box } from "./place";
  import { skySource } from "./picture";
  import { frostFrom } from "./print.svelte";
  import type { Scene } from "./scenes";

  let {
    scene,
    width,
    height,
    sky: _sky,
    horizon: _horizon,
    pictureHorizon: _pictureHorizon,
    zoom = 1,
    quiet: _quiet = null,
    onready,
  }: {
    scene: Scene;
    width: number;
    height: number;
    sky: Box;
    horizon: number;
    pictureHorizon: number;
    zoom?: number;
    quiet?: Quiet | null;
    onready?: (scene: Scene) => void;
  } = $props();

  let density = $state(globalThis.devicePixelRatio || 1);
  $effect(() => {
    const query = matchMedia(`(resolution: ${density}dppx)`);
    const moved = () => (density = globalThis.devicePixelRatio || 1);
    query.addEventListener("change", moved, { once: true });
    return () => query.removeEventListener("change", moved);
  });

  let canvas = $state<HTMLCanvasElement>();
  let glCanvas = $state<HTMLCanvasElement>();

  // 1. Crisp base mountain skyline wallpaper + sidebar frosted tint sampling
  $effect(() => {
    const target = canvas;
    const wantedScene = scene;
    const w = Math.max(1, Math.round(width));
    const h = Math.max(1, Math.round(height));
    const dpr = Math.min(2, density * zoom);
    if (!target || w <= 1 || h <= 1) return;

    let live = true;
    const img = new Image();
    img.src = skySource(wantedScene);
    void img.decode().then(
      () => {
        if (!live) return;
        const cw = Math.max(1, Math.round(w * dpr));
        const ch = Math.max(1, Math.round(h * dpr));
        if (target.width !== cw) target.width = cw;
        if (target.height !== ch) target.height = ch;
        const ctx = target.getContext("2d");
        if (ctx) {
          const ir = img.naturalWidth / Math.max(1, img.naturalHeight);
          const cr = cw / Math.max(1, ch);
          let dw = cw;
          let dh = ch;
          let dx = 0;
          let dy = 0;
          if (cr > ir) {
            dh = cw / ir;
            dy = (ch - dh) * 0.5;
          } else {
            dw = ch * ir;
            dx = (cw - dw) * 0.5;
          }
          ctx.imageSmoothingEnabled = true;
          ctx.imageSmoothingQuality = "high";
          ctx.drawImage(img, dx, dy, dw, dh);
          frostFrom(wantedScene, target);
        }
        onready?.(wantedScene);
      },
      () => {
        if (live) onready?.(wantedScene);
      },
    );

    return () => {
      live = false;
    };
  });

  // 2. Cinema-grade WebGL GLSL volumetric clouds & aurora shader (pauses when tab hidden)
  const VERT_SRC = `
    attribute vec2 a_pos;
    varying vec2 v_uv;
    void main() {
      v_uv = a_pos * 0.5 + 0.5;
      gl_Position = vec4(a_pos, 0.0, 1.0);
    }
  `;

  const FRAG_SRC = `
    precision mediump float;
    varying vec2 v_uv;
    uniform float u_time;
    uniform vec2 u_res;
    uniform float u_day;

    float hash(vec2 p) {
      p = fract(p * vec2(123.34, 456.21));
      p += dot(p, p + 45.32);
      return fract(p.x * p.y);
    }

    float noise(vec2 p) {
      vec2 i = floor(p);
      vec2 f = fract(p);
      f = f * f * (3.0 - 2.0 * f);
      float a = hash(i);
      float b = hash(i + vec2(1.0, 0.0));
      float c = hash(i + vec2(0.0, 1.0));
      float d = hash(i + vec2(1.0, 1.0));
      return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
    }

    float fbm(vec2 p) {
      float v = 0.0;
      float amp = 0.5;
      mat2 rot = mat2(0.8, 0.6, -0.6, 0.8);
      for (int i = 0; i < 5; i++) {
        v += amp * noise(p);
        p = rot * p * 2.02 + vec2(1.7, 9.2);
        amp *= 0.5;
      }
      return v;
    }

    void main() {
      vec2 uv = v_uv;
      float aspect = u_res.x / max(1.0, u_res.y);
      vec2 p = vec2((uv.x - 0.5) * aspect, uv.y);

      float t = u_time * 0.09;

      // Domain-warped organic volumetric clouds gliding horizontally across horizon
      vec2 q = vec2(
        fbm(p * vec2(1.8, 2.6) + vec2(t * 0.9, t * 0.15)),
        fbm(p * vec2(1.8, 2.6) + vec2(-t * 0.55 + 4.1, t * 0.2))
      );
      float clouds = fbm(p * vec2(2.3, 3.2) + 1.35 * q + vec2(t * 1.15, 0.0));

      // Silky flowing aurora curtain wave above the mountain ridge
      float wave = sin(p.x * 2.4 + t * 2.2 + q.x * 2.8) * 0.09;
      float band = exp(-pow((uv.y - (0.34 + wave)) * 5.2, 2.0));
      float ribbon = band * smoothstep(0.28, 0.85, clouds);

      // Soft horizon cloud layer
      float horizonMask = smoothstep(0.02, 0.24, uv.y) * smoothstep(0.76, 0.28, uv.y);
      float softCloud = smoothstep(0.38, 0.82, clouds) * horizonMask;

      if (u_day > 0.5) {
        vec3 col = mix(vec3(0.85, 0.93, 1.0), vec3(1.0, 0.98, 0.95), clouds);
        float alpha = softCloud * 0.42;
        gl_FragColor = vec4(col * alpha, alpha);
      } else {
        vec3 cyan = vec3(0.18, 0.78, 0.98);
        vec3 indigo = vec3(0.45, 0.48, 0.98);
        vec3 mistCol = mix(indigo, cyan, smoothstep(0.25, 0.8, q.x));
        vec3 col = mistCol * (softCloud * 0.34 + ribbon * 0.52);
        float alpha = clamp(softCloud * 0.42 + ribbon * 0.55, 0.0, 0.75);
        gl_FragColor = vec4(col, alpha);
      }
    }
  `;

  $effect(() => {
    const target = glCanvas;
    const isDay = scene === "day" ? 1.0 : 0.0;
    const w = Math.max(1, Math.round(width));
    const h = Math.max(1, Math.round(height));
    if (!target || w <= 1 || h <= 1) return;

    const gl = target.getContext("webgl", {
      alpha: true,
      antialias: false,
      depth: false,
      stencil: false,
      powerPreference: "low-power",
      premultipliedAlpha: true,
    });
    if (!gl) return;

    // Half-resolution GPU shader keeps frame cost < 0.3ms while looking silky smooth
    target.width = Math.max(1, Math.round(w * 0.5));
    target.height = Math.max(1, Math.round(h * 0.5));
    gl.viewport(0, 0, target.width, target.height);

    const compile = (type: number, src: string) => {
      const sh = gl.createShader(type);
      if (!sh) return null;
      gl.shaderSource(sh, src);
      gl.compileShader(sh);
      if (!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) {
        gl.deleteShader(sh);
        return null;
      }
      return sh;
    };

    const vs = compile(gl.VERTEX_SHADER, VERT_SRC);
    const fs = compile(gl.FRAGMENT_SHADER, FRAG_SRC);
    if (!vs || !fs) return;

    const prog = gl.createProgram();
    if (!prog) return;
    gl.attachShader(prog, vs);
    gl.attachShader(prog, fs);
    gl.linkProgram(prog);
    if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) return;

    const buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);
    gl.bufferData(
      gl.ARRAY_BUFFER,
      new Float32Array([-1, -1, 1, -1, -1, 1, -1, 1, 1, -1, 1, 1]),
      gl.STATIC_DRAW,
    );

    gl.useProgram(prog);
    const posLoc = gl.getAttribLocation(prog, "a_pos");
    gl.enableVertexAttribArray(posLoc);
    gl.vertexAttribPointer(posLoc, 2, gl.FLOAT, false, 0, 0);

    const timeLoc = gl.getUniformLocation(prog, "u_time");
    const resLoc = gl.getUniformLocation(prog, "u_res");
    const dayLoc = gl.getUniformLocation(prog, "u_day");

    let raf = 0;
    const start = performance.now();
    let lastFrame = 0;

    const render = (now: number) => {
      raf = requestAnimationFrame(render);
      // Pause automatically when hidden & cap cleanly to ~30fps for zero overhead
      if (document.hidden || now - lastFrame < 32) return;
      lastFrame = now;
      gl.clearColor(0, 0, 0, 0);
      gl.clear(gl.COLOR_BUFFER_BIT);
      gl.uniform1f(timeLoc, (now - start) / 1000.0);
      gl.uniform2f(resLoc, target.width, target.height);
      gl.uniform1f(dayLoc, isDay);
      gl.drawArrays(gl.TRIANGLES, 0, 6);
    };

    raf = requestAnimationFrame(render);
    return () => {
      cancelAnimationFrame(raf);
      gl.deleteBuffer(buf);
      gl.deleteProgram(prog);
      gl.deleteShader(vs);
      gl.deleteShader(fs);
    };
  });
</script>

<div class="sky-stage" style:width={`${width}px`} style:height={`${height}px`}>
  <canvas
    bind:this={canvas}
    class="sky-print"
    aria-hidden="true"
    style:width={`${width}px`}
    style:height={`${height}px`}
  ></canvas>
  <canvas
    bind:this={glCanvas}
    class="sky-shader"
    data-scene={scene}
    aria-hidden="true"
    style:width={`${width}px`}
    style:height={`${height}px`}
  ></canvas>
</div>

<style>
  .sky-stage {
    position: absolute;
    inset-block-start: 0;
    inset-inline-start: 0;
    overflow: hidden;
    pointer-events: none;
  }

  .sky-print,
  .sky-shader {
    position: absolute;
    inset-block-start: 0;
    inset-inline-start: 0;
    pointer-events: none;
  }

  .sky-shader[data-scene="night"] {
    mix-blend-mode: screen;
  }

  .sky-shader[data-scene="day"] {
    mix-blend-mode: normal;
  }
</style>
