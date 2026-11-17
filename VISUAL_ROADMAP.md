# Visual modernization plan

## Direction

A midnight-blue arcade arena with saturated jewel-colored bricks, satin metal, a luminous pearl ball and restrained cyan/violet lighting. Prioritize the ball silhouette, readable bonuses and immediate paddle response. Keep the existing 640×480 arena, 50 boards, hit boxes and 60 Hz simulation; render the presentation at 3840×2160.

## Phase 1 — implemented

- Replace the pixel-art playfield with resolution-independent procedural materials: rounded glass bricks, beveled edges, reflected light strips, metal bricks and chrome paddle.
- Add layered soft contact shadows, a floor grid, colored ambient light pools and local illumination that follows the ball.
- Render the ball as a shaded luminous sphere and impacts as fading glass fragments. Preserve fireball, small-ball, grab and shooting-paddle visual cues.
- Carry the same glass styling through the frame, score panel, menus, pause/result cards, high scores, bonus guide and board editor. Use matching vector icons on falling bonuses and their guide entries; retain blue/good, red/danger and gray/mixed colors.
- Keep durable-brick pips, metal-brick bars and explosive-brick marks readable. Hidden bricks remain hidden in play and visible as outlines in the editor.
- Batch material parameters per vertex; add no dependencies; bundle DejaVu Sans with its redistribution license for smooth modern typography. Stop loading obsolete bitmap graphics. Animate colored-brick highlights and explosive-brick diamond pulses using simulation time; share the same materials with the editor.

The renderer simulates lighting, surface reflections and depth in screen space. It does **not** implement hardware ray tracing, traced refraction, physically accurate shadows, scene blur or global illumination. The panels use tinted translucent surfaces and highlights to establish the glass style.

## Visual refinement — implemented

Responding to feedback that the first glass pass looked less appealing: brighten and saturate the brick palette; narrow the bevels and reflections; replace heavy glossy interface borders with quiet navy panels; soften shadows; give the paddle a clear cyan center and warm end caps; replace large explosive-brick crosses with small ember diamonds; and add a colorful three-row brick motif to the menu. Gameplay geometry, input and rules remain unchanged.

## Animation and bonus readability — implemented

Colored tiles now have moving light bands and explosive tiles have pulsing diamond patterns. Falling bonuses use 32×30 badges with crisp vector symbols, matching their existing catch area and the F1 guide. All 19 bonus entries share the same icon renderer; blue/red/gray meaning is preserved. UI font sizes are cached before drawing so atlas growth cannot invalidate queued guide labels.

Verified with actual 4K guide captures and two editor captures: 58,471 pixels in the animated tile toolbar changed while the board and selection stayed fixed. The eight gameplay/layout tests and Linux release build pass. Exact animation parity with the original remains unverified.

## Phase 2 — next visual-quality milestone

1. Add bounded ball trails and richer impact fragments, with reduced-effects and brightness controls.
2. Prototype a downsampled bloom pass and scene blur behind interface cards; compare GPU frame timings at 1080p and 4K before adopting them.
3. Add shadow masks, filtered penumbrae and a low-resolution reflection buffer, keeping ball/brick contrast clear during busy multiball scenes.
4. Profile low-end and integrated GPUs; introduce quality presets based on measured costs. Validate actual Windows and macOS shader compilation/rendering on native machines.

## Phase 3 — evaluate true 3D / ray tracing

Build a separate renderer prototype with extruded glass geometry, physically based roughness/metalness, an environment map, shadow maps and screen-space reflections. Keep simulation and pointer mapping shared. Establish 4K frame-time and visual-readability targets on actual hardware before considering Vulkan/DirectX/Metal ray tracing; the current macroquad/OpenGL renderer has no hardware ray-tracing path. A new backend, acceleration structures, denoising, capability detection and a non-RT fallback would be a substantial engine project.

## Verification boundaries

Use the existing eight tests for rules and viewport/pointer alignment. Build the release executable, inspect actual 4K images of gameplay and other screens, and run Craft Anything's `rebuild_check` for compilation, drawing, a genuine 600-frame gameplay probe, mechanics documentation and credits. Original runtime equivalence remains unverified. Automated rendering uses software OpenGL for repeatability; it does not establish real-GPU performance or cross-platform compatibility.

Validated in this pass on Linux: release build, all eight tests, clean exits from menu/help/editor captures, a 600-frame release gameplay probe (123 brick hits, five paddle rebounds, score 628), and `rebuild_check` build/3840×2160 screenshot/gameplay/credits checks. The font-atlas shutdown crash found during capture was fixed by releasing font resources before the graphics context. Native Windows/macOS execution and comparisons against the original runtime remain unverified.
