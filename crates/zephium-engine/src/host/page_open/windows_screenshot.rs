//! Windows WebView2 custom context menu items:
//! 1. "Web'de ara" for selected text (opens in a new tab grouped under the source tab).
//! 2. "Ekran görüntüsü al" modern scrolling area / full-page screenshot tool using
//!    WebView2 CDP `Page.captureScreenshot` with `captureBeyondViewport: true`.

use std::cell::Cell;
use std::rc::Rc;

use base64::Engine as _;
use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2, ICoreWebView2ContextMenuItem, ICoreWebView2ContextMenuRequestedEventArgs,
    ICoreWebView2Controller, ICoreWebView2Environment9, ICoreWebView2_2,
    COREWEBVIEW2_CONTEXT_MENU_ITEM_KIND_COMMAND,
};
use windows::core::{Interface, HSTRING, PWSTR};
use windows::Win32::System::Com::CoTaskMemFree;
use zephium_core::ids::ItemId;
use zephium_core::ports::engine::EngineEvent;

use crate::host::{EventPermit, Sink};

/// Trusted-Types-safe (zero `innerHTML`) modern scrolling screenshot selector.
/// Resolves with JSON `{"action":"save"|"copy","x":...,"y":...,"width":...,"height":...}`
/// or `null` if cancelled.
const SCREENSHOT_OVERLAY_JS: &str = r#"(function(){
  return new Promise(function(resolve){
    try {
      var prev = document.getElementById('__aether_shot_root');
      if (prev) prev.remove();

      var root = document.createElement('div');
      root.id = '__aether_shot_root';
      root.style.cssText = 'position:fixed;inset:0;z-index:2147483647;cursor:crosshair;user-select:none;font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;';

      var veil = document.createElement('div');
      veil.style.cssText = 'position:fixed;inset:0;background:rgba(8,12,20,0.42);pointer-events:none;transition:opacity 120ms ease;';
      root.appendChild(veil);

      var sel = document.createElement('div');
      sel.style.cssText = 'position:fixed;display:none;border:2px solid #38bdf8;border-radius:4px;box-shadow:0 0 0 99999px rgba(8,12,20,0.48),0 0 22px rgba(56,189,248,0.35);pointer-events:none;';
      root.appendChild(sel);

      var dimBadge = document.createElement('div');
      dimBadge.style.cssText = 'position:fixed;display:none;padding:4px 9px;border-radius:7px;background:rgba(15,19,28,0.92);color:#e2e8f0;border:1px solid rgba(56,189,248,0.35);font-size:11.5px;font-weight:600;pointer-events:none;font-variant-numeric:tabular-nums;';
      root.appendChild(dimBadge);

      var topBar = document.createElement('div');
      topBar.style.cssText = 'position:fixed;top:16px;left:50%;transform:translateX(-50%);display:flex;align-items:center;gap:8px;padding:7px 10px 7px 14px;border-radius:12px;background:rgba(15,19,28,0.92);color:#f1f5f9;border:1px solid rgba(255,255,255,0.14);box-shadow:0 12px 32px rgba(0,0,0,0.48);backdrop-filter:blur(16px);cursor:default;';

      var hint = document.createElement('span');
      hint.textContent = 'Alan seçmek için sürükle · Aşağı kaydırarak uzun görüntü al';
      hint.style.cssText = 'font-size:12px;color:#cbd5e1;margin-right:4px;white-space:nowrap;';
      topBar.appendChild(hint);

      function makeBtn(label, primary) {
        var b = document.createElement('button');
        b.type = 'button';
        b.textContent = label;
        b.style.cssText = primary
          ? 'padding:5px 11px;border-radius:8px;border:0;background:linear-gradient(135deg,#0ea5e9,#6366f1);color:#fff;font-size:12px;font-weight:600;cursor:pointer;white-space:nowrap;'
          : 'padding:5px 10px;border-radius:8px;border:1px solid rgba(255,255,255,0.14);background:rgba(255,255,255,0.07);color:#f1f5f9;font-size:12px;font-weight:550;cursor:pointer;white-space:nowrap;';
        return b;
      }

      var btnFull = makeBtn('Tüm Sayfa', false);
      var btnVis = makeBtn('Görünür Alan', false);
      var btnCancelTop = makeBtn('İptal', false);
      topBar.appendChild(btnFull);
      topBar.appendChild(btnVis);
      topBar.appendChild(btnCancelTop);
      root.appendChild(topBar);

      var actions = document.createElement('div');
      actions.style.cssText = 'position:fixed;display:none;align-items:center;gap:6px;padding:6px 8px;border-radius:11px;background:rgba(15,19,28,0.94);border:1px solid rgba(56,189,248,0.35);box-shadow:0 10px 28px rgba(0,0,0,0.5);backdrop-filter:blur(16px);cursor:default;';
      var btnCopy = makeBtn('Panoya Kopyala', true);
      var btnSave = makeBtn('PNG Kaydet', false);
      var btnRetry = makeBtn('Yeniden Seç', false);
      var btnCancel = makeBtn('İptal', false);
      actions.appendChild(btnCopy);
      actions.appendChild(btnSave);
      actions.appendChild(btnRetry);
      actions.appendChild(btnCancel);
      root.appendChild(actions);

      var dragging = false;
      var hasBox = false;
      var startX = 0, startY = 0, endX = 0, endY = 0;
      var curClientX = 0, curClientY = 0;
      var scrollRaf = 0;
      var finished = false;

      function docSize() {
        var de = document.documentElement, b = document.body;
        return {
          w: Math.max(de.scrollWidth, de.clientWidth, b ? b.scrollWidth : 0),
          h: Math.max(de.scrollHeight, de.clientHeight, b ? b.scrollHeight : 0)
        };
      }

      function rect() {
        var x = Math.min(startX, endX);
        var y = Math.min(startY, endY);
        var w = Math.abs(endX - startX);
        var h = Math.abs(endY - startY);
        return { x: Math.max(0, x), y: Math.max(0, y), w: w, h: h };
      }

      function renderBox() {
        var r = rect();
        if (r.w < 4 || r.h < 4) {
          sel.style.display = 'none';
          dimBadge.style.display = 'none';
          veil.style.display = 'block';
          return;
        }
        veil.style.display = 'none';
        var vx = r.x - window.scrollX;
        var vy = r.y - window.scrollY;
        sel.style.display = 'block';
        sel.style.left = Math.round(vx) + 'px';
        sel.style.top = Math.round(vy) + 'px';
        sel.style.width = Math.round(r.w) + 'px';
        sel.style.height = Math.round(r.h) + 'px';

        dimBadge.style.display = 'block';
        dimBadge.textContent = Math.round(r.w) + ' × ' + Math.round(r.h) + ' px';
        var bx = Math.min(Math.max(vx, 8), window.innerWidth - 120);
        var by = Math.min(Math.max(vy - 30, 68), window.innerHeight - 36);
        dimBadge.style.left = Math.round(bx) + 'px';
        dimBadge.style.top = Math.round(by) + 'px';

        if (!dragging && hasBox) {
          actions.style.display = 'flex';
          var ax = Math.min(Math.max(vx + r.w - 310, 12), Math.max(12, window.innerWidth - 330));
          var ay = vy + r.h + 10;
          if (ay > window.innerHeight - 54) ay = Math.max(68, vy - 48);
          actions.style.left = Math.round(ax) + 'px';
          actions.style.top = Math.round(ay) + 'px';
        } else {
          actions.style.display = 'none';
        }
      }

      function tickScroll() {
        scrollRaf = 0;
        if (!dragging) return;
        var margin = 64;
        var dy = 0, dx = 0;
        if (curClientY > window.innerHeight - margin) {
          dy = Math.round(((curClientY - (window.innerHeight - margin)) / margin) * 24) + 6;
        } else if (curClientY < margin) {
          dy = -Math.round(((margin - curClientY) / margin) * 24) - 6;
        }
        if (curClientX > window.innerWidth - margin) {
          dx = 14;
        } else if (curClientX < margin) {
          dx = -14;
        }
        if (dx !== 0 || dy !== 0) {
          window.scrollBy(dx, dy);
          endX = curClientX + window.scrollX;
          endY = curClientY + window.scrollY;
          renderBox();
        }
        scrollRaf = requestAnimationFrame(tickScroll);
      }

      function cleanup() {
        if (scrollRaf) cancelAnimationFrame(scrollRaf);
        window.removeEventListener('keydown', onKey, true);
        window.removeEventListener('scroll', renderBox, true);
        window.removeEventListener('wheel', onWheel, { passive: true });
        if (root.parentNode) root.remove();
      }

      function finish(payload) {
        if (finished) return;
        finished = true;
        cleanup();
        if (!payload) { resolve(null); return; }
        requestAnimationFrame(function(){
          requestAnimationFrame(function(){
            resolve(JSON.stringify(payload));
          });
        });
      }

      function onKey(e) {
        if (e.key === 'Escape') {
          e.preventDefault();
          e.stopPropagation();
          finish(null);
        }
      }

      function onWheel() {
        if (dragging) {
          requestAnimationFrame(function(){
            endX = curClientX + window.scrollX;
            endY = curClientY + window.scrollY;
            renderBox();
          });
        }
      }

      root.addEventListener('mousedown', function(e) {
        if (e.button !== 0) return;
        if (topBar.contains(e.target) || actions.contains(e.target)) return;
        e.preventDefault();
        dragging = true;
        hasBox = false;
        curClientX = e.clientX;
        curClientY = e.clientY;
        startX = e.clientX + window.scrollX;
        startY = e.clientY + window.scrollY;
        endX = startX;
        endY = startY;
        renderBox();
        if (!scrollRaf) scrollRaf = requestAnimationFrame(tickScroll);
      });

      window.addEventListener('mousemove', function(e) {
        if (!dragging) return;
        curClientX = e.clientX;
        curClientY = e.clientY;
        endX = e.clientX + window.scrollX;
        endY = e.clientY + window.scrollY;
        renderBox();
      }, true);

      window.addEventListener('mouseup', function() {
        if (!dragging) return;
        dragging = false;
        if (scrollRaf) { cancelAnimationFrame(scrollRaf); scrollRaf = 0; }
        var r = rect();
        hasBox = r.w >= 6 && r.h >= 6;
        renderBox();
      }, true);

      function buildPayload(action, r) {
        var beyond =
          r.y < window.scrollY - 1 ||
          r.y + r.h > window.scrollY + window.innerHeight + 1 ||
          r.x < window.scrollX - 1 ||
          r.x + r.w > window.scrollX + window.innerWidth + 1;
        return {
          action: action,
          beyond: beyond,
          x: beyond ? r.x : Math.max(0, r.x - window.scrollX),
          y: beyond ? r.y : Math.max(0, r.y - window.scrollY),
          width: r.w,
          height: r.h,
          vw: window.innerWidth || 1,
          vh: window.innerHeight || 1
        };
      }

      btnCopy.onclick = function() {
        finish(buildPayload('copy', rect()));
      };
      btnSave.onclick = function() {
        finish(buildPayload('save', rect()));
      };
      btnRetry.onclick = function() {
        hasBox = false;
        startX = startY = endX = endY = 0;
        renderBox();
      };
      btnFull.onclick = function() {
        var s = docSize();
        finish({ action: 'save', beyond: true, x: 0, y: 0, width: s.w, height: Math.min(s.h, 16000), vw: window.innerWidth || 1, vh: window.innerHeight || 1 });
      };
      btnVis.onclick = function() {
        finish({ action: 'save', beyond: false, x: 0, y: 0, width: window.innerWidth, height: window.innerHeight, vw: window.innerWidth || 1, vh: window.innerHeight || 1 });
      };
      btnCancel.onclick = function() { finish(null); };
      btnCancelTop.onclick = function() { finish(null); };

      window.addEventListener('keydown', onKey, true);
      window.addEventListener('scroll', renderBox, true);
      window.addEventListener('wheel', onWheel, { passive: true });
      (document.body || document.documentElement).appendChild(root);
    } catch(_) {
      resolve(null);
    }
  });
})();"#;

fn pwstr_to_string(ptr: PWSTR) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let len = (0..=4096).find(|offset| unsafe { *ptr.0.add(*offset) } == 0)?;
    let text = String::from_utf16(unsafe { std::slice::from_raw_parts(ptr.0, len) }).ok();
    unsafe { CoTaskMemFree(Some(ptr.0.cast())) };
    text
}

fn show_page_notice(core: &ICoreWebView2, text: &str) {
    let Ok(literal) = serde_json::to_string(text) else {
        return;
    };
    let script = format!(
        r#"(function(msg){{
  try {{
    var old = document.getElementById('__aether_shot_toast');
    if (old) old.remove();
    var box = document.createElement('div');
    box.id = '__aether_shot_toast';
    box.textContent = msg;
    box.style.cssText = 'position:fixed;top:18px;left:50%;transform:translate(-50%,-6px);opacity:0;z-index:2147483647;pointer-events:none;padding:8px 15px;border-radius:999px;background:rgba(15,19,28,0.94);color:#f1f5f9;border:1px solid rgba(52,211,153,0.45);box-shadow:0 8px 26px rgba(0,0,0,0.45);font:600 12.5px/1.2 -apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;backdrop-filter:blur(14px);transition:opacity 160ms ease,transform 180ms ease;';
    (document.body || document.documentElement).appendChild(box);
    requestAnimationFrame(function(){{ box.style.opacity='1'; box.style.transform='translate(-50%,0)'; }});
    setTimeout(function(){{ box.style.opacity='0'; setTimeout(function(){{ if(box.parentNode) box.remove(); }}, 200); }}, 2100);
  }} catch(_) {{}}
}})({literal});"#
    );
    let _ = unsafe { core.ExecuteScript(&HSTRING::from(script), None) };
}

fn save_png_to_downloads(bytes: &[u8]) -> Option<std::path::PathBuf> {
    let home = std::env::var_os("USERPROFILE")?;
    let dir = std::path::PathBuf::from(home).join("Downloads");
    let _ = std::fs::create_dir_all(&dir);
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let path = dir.join(format!("aether-screenshot-{ts}.png"));
    std::fs::write(&path, bytes).ok()?;
    Some(path)
}

fn copy_png_in_page_or_save(core: &ICoreWebView2, b64: &str, bytes: &[u8]) {
    let Ok(literal) = serde_json::to_string(b64) else {
        return;
    };
    let saved_fallback = bytes.to_vec();
    let core_clone = core.clone();
    let script = format!(
        r#"(async function(b64){{
  try {{
    var bin = atob(b64);
    var arr = new Uint8Array(bin.length);
    for (var i = 0; i < bin.length; i++) arr[i] = bin.charCodeAt(i);
    var blob = new Blob([arr], {{ type: 'image/png' }});
    await navigator.clipboard.write([new ClipboardItem({{ 'image/png': blob }})]);
    return 'ok';
  }} catch(_) {{
    return 'fail';
  }}
}})({literal})"#
    );
    let params = serde_json::json!({
        "expression": script,
        "awaitPromise": true,
        "returnByValue": true
    })
    .to_string();
    let handler =
        webview2_com::CallDevToolsProtocolMethodCompletedHandler::create(Box::new(move |hr, raw| {
            let ok = hr.is_ok()
                && serde_json::from_str::<serde_json::Value>(&raw)
                    .ok()
                    .and_then(|v| v.pointer("/result/value")?.as_str().map(|s| s == "ok"))
                    .unwrap_or(false);
            if ok {
                show_page_notice(&core_clone, "Ekran görüntüsü panoya kopyalandı ✓");
            } else if save_png_to_downloads(&saved_fallback).is_some() {
                show_page_notice(&core_clone, "Pano izni yok — İndirilenler'e kaydedildi ✓");
            }
            Ok(())
        }));
    let _ = unsafe {
        core.CallDevToolsProtocolMethod(
            &HSTRING::from("Runtime.evaluate"),
            &HSTRING::from(params),
            &handler,
        )
    };
}

fn crop_and_finish_in_page(
    core: &ICoreWebView2,
    b64: &str,
    action: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    vw: f64,
    vh: f64,
) {
    let Ok(literal) = serde_json::to_string(b64) else {
        return;
    };
    let Ok(act_literal) = serde_json::to_string(action) else {
        return;
    };
    let core_clone = core.clone();
    let script = format!(
        r#"(async function(b64, act, rx, ry, rw, rh, vw, vh){{
  try {{
    var bin = atob(b64);
    var arr = new Uint8Array(bin.length);
    for (var i = 0; i < bin.length; i++) arr[i] = bin.charCodeAt(i);
    var srcBlob = new Blob([arr], {{ type: 'image/png' }});
    var bmp = await createImageBitmap(srcBlob);
    var scaleX = bmp.width / Math.max(1, vw);
    var scaleY = bmp.height / Math.max(1, vh);
    var sx = Math.max(0, Math.round(rx * scaleX));
    var sy = Math.max(0, Math.round(ry * scaleY));
    var sw = Math.max(1, Math.min(bmp.width - sx, Math.round(rw * scaleX)));
    var sh = Math.max(1, Math.min(bmp.height - sy, Math.round(rh * scaleY)));
    var outBlob;
    if (typeof OffscreenCanvas !== 'undefined') {{
      var oc = new OffscreenCanvas(sw, sh);
      var octx = oc.getContext('2d');
      octx.drawImage(bmp, sx, sy, sw, sh, 0, 0, sw, sh);
      outBlob = await oc.convertToBlob({{ type: 'image/png' }});
    }} else {{
      var c = document.createElement('canvas');
      c.width = sw;
      c.height = sh;
      var ctx = c.getContext('2d');
      ctx.drawImage(bmp, sx, sy, sw, sh, 0, 0, sw, sh);
      outBlob = await new Promise(function(res){{ c.toBlob(res, 'image/png'); }});
    }}
    bmp.close();
    if (act === 'copy') {{
      try {{
        await navigator.clipboard.write([new ClipboardItem({{ 'image/png': outBlob }})]);
        return 'ok';
      }} catch(_) {{}}
    }}
    var buf = new Uint8Array(await outBlob.arrayBuffer());
    var chunks = [];
    for (var j = 0; j < buf.length; j += 0x8000) {{
      chunks.push(String.fromCharCode.apply(null, buf.subarray(j, j + 0x8000)));
    }}
    return (act === 'copy' ? 'fallback:' : 'save:') + btoa(chunks.join(''));
  }} catch(_) {{
    return 'fail';
  }}
}})({literal}, {act_literal}, {x}, {y}, {width}, {height}, {vw}, {vh})"#
    );
    let params = serde_json::json!({
        "expression": script,
        "awaitPromise": true,
        "returnByValue": true
    })
    .to_string();
    let handler =
        webview2_com::CallDevToolsProtocolMethodCompletedHandler::create(Box::new(move |hr, raw| {
            let reply = if hr.is_ok() {
                serde_json::from_str::<serde_json::Value>(&raw)
                    .ok()
                    .and_then(|v| v.pointer("/result/value")?.as_str().map(str::to_owned))
            } else {
                None
            };
            match reply.as_deref() {
                Some("ok") => {
                    show_page_notice(&core_clone, "Ekran görüntüsü panoya kopyalandı ✓");
                }
                Some(rest) if rest.starts_with("fallback:") => {
                    if let Ok(bytes) =
                        base64::engine::general_purpose::STANDARD.decode(&rest["fallback:".len()..])
                    {
                        if save_png_to_downloads(&bytes).is_some() {
                            show_page_notice(
                                &core_clone,
                                "Pano izni yok — İndirilenler'e kaydedildi ✓",
                            );
                        }
                    }
                }
                Some(rest) if rest.starts_with("save:") => {
                    if let Ok(bytes) =
                        base64::engine::general_purpose::STANDARD.decode(&rest["save:".len()..])
                    {
                        if save_png_to_downloads(&bytes).is_some() {
                            show_page_notice(
                                &core_clone,
                                "Ekran görüntüsü İndirilenler'e kaydedildi ✓",
                            );
                        }
                    }
                }
                _ => {}
            }
            Ok(())
        }));
    let _ = unsafe {
        core.CallDevToolsProtocolMethod(
            &HSTRING::from("Runtime.evaluate"),
            &HSTRING::from(params),
            &handler,
        )
    };
}

fn start_screenshot_overlay(core: ICoreWebView2) {
    let params = serde_json::json!({
        "expression": SCREENSHOT_OVERLAY_JS,
        "awaitPromise": true,
        "returnByValue": true
    })
    .to_string();
    let core_for_reply = core.clone();
    let handler =
        webview2_com::CallDevToolsProtocolMethodCompletedHandler::create(Box::new(move |hr, raw| {
            if hr.is_err() {
                return Ok(());
            }
            let Some(json_str) = serde_json::from_str::<serde_json::Value>(&raw)
                .ok()
                .and_then(|v| v.pointer("/result/value")?.as_str().map(str::to_owned))
            else {
                return Ok(());
            };
            let Ok(req) = serde_json::from_str::<serde_json::Value>(&json_str) else {
                return Ok(());
            };
            let action = req
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("save")
                .to_owned();
            let x = req.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0).max(0.0);
            let y = req.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0).max(0.0);
            let width = req
                .get("width")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0)
                .clamp(1.0, 16000.0);
            let height = req
                .get("height")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0)
                .clamp(1.0, 16000.0);
            let vw = req.get("vw").and_then(|v| v.as_f64()).unwrap_or(1.0).max(1.0);
            let vh = req.get("vh").and_then(|v| v.as_f64()).unwrap_or(1.0).max(1.0);

            let beyond = req.get("beyond").and_then(|v| v.as_bool()).unwrap_or(false);
            // For in-viewport captures, omit `clip` and `captureBeyondViewport`
            // so Chromium reads the current compositor frame without resizing
            // or scrolling the viewport (zero flash, zero page movement), then
            // crop offscreen in memory.
            let shot_params = if beyond {
                serde_json::json!({
                    "format": "png",
                    "captureBeyondViewport": true,
                    "fromSurface": true,
                    "clip": {
                        "x": x,
                        "y": y,
                        "width": width,
                        "height": height,
                        "scale": 1
                    }
                })
                .to_string()
            } else {
                serde_json::json!({
                    "format": "png",
                    "fromSurface": true
                })
                .to_string()
            };

            let core_done = core_for_reply.clone();
            let shot_handler =
                webview2_com::CallDevToolsProtocolMethodCompletedHandler::create(Box::new(
                    move |shot_hr, shot_raw| {
                        if shot_hr.is_err() {
                            return Ok(());
                        }
                        let Some(b64) = serde_json::from_str::<serde_json::Value>(&shot_raw)
                            .ok()
                            .and_then(|v| v.get("data")?.as_str().map(str::to_owned))
                        else {
                            return Ok(());
                        };
                        if !beyond {
                            crop_and_finish_in_page(
                                &core_done, &b64, &action, x, y, width, height, vw, vh,
                            );
                            return Ok(());
                        }
                        let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(&b64)
                        else {
                            return Ok(());
                        };
                        if action == "copy" {
                            copy_png_in_page_or_save(&core_done, &b64, &bytes);
                        } else if save_png_to_downloads(&bytes).is_some() {
                            show_page_notice(&core_done, "Ekran görüntüsü İndirilenler'e kaydedildi ✓");
                        }
                        Ok(())
                    },
                ));
            let _ = unsafe {
                core_for_reply.CallDevToolsProtocolMethod(
                    &HSTRING::from("Page.captureScreenshot"),
                    &HSTRING::from(shot_params),
                    &shot_handler,
                )
            };
            Ok(())
        }));
    let _ = unsafe {
        core.CallDevToolsProtocolMethod(
            &HSTRING::from("Runtime.evaluate"),
            &HSTRING::from(params),
            &handler,
        )
    };
}

pub(super) fn inject_custom_items(
    source: &Rc<Cell<ItemId>>,
    permit: &EventPermit,
    sink: &Sink,
    controller: &ICoreWebView2Controller,
    args: &ICoreWebView2ContextMenuRequestedEventArgs,
) {
    let Ok(core) = (unsafe { controller.CoreWebView2() }) else {
        return;
    };
    let Ok(core2) = core.cast::<ICoreWebView2_2>() else {
        return;
    };
    let Ok(env) = (unsafe { core2.Environment() }) else {
        return;
    };
    let Ok(env9) = env.cast::<ICoreWebView2Environment9>() else {
        return;
    };
    let Ok(items) = (unsafe { args.MenuItems() }) else {
        return;
    };

    let mut insert_at = 0u32;

    // 1. "Web'de ara" when text is selected.
    if let Ok(target) = unsafe { args.ContextMenuTarget() } {
        let mut has_sel = windows::core::BOOL(0);
        if unsafe { target.HasSelection(&mut has_sel) }.is_ok() && has_sel.as_bool() {
            let mut raw = PWSTR::null();
            if unsafe { target.SelectionText(&mut raw) }.is_ok() {
                if let Some(selected) = pwstr_to_string(raw) {
                    let trimmed = selected.trim();
                    if !trimmed.is_empty() {
                        let preview: String = trimmed.chars().take(28).collect();
                        let label = if trimmed.chars().count() > 28 {
                            format!("Web'de ara: \"{preview}…\"")
                        } else {
                            format!("Web'de ara: \"{preview}\"")
                        };
                        if let Ok(item) = unsafe {
                            env9.CreateContextMenuItem(
                                &HSTRING::from(label),
                                None,
                                COREWEBVIEW2_CONTEXT_MENU_ITEM_KIND_COMMAND,
                            )
                        } {
                            let query = trimmed.to_owned();
                            let src = source.clone();
                            let prm = permit.clone();
                            let snk = sink.clone();
                            let mut token = 0i64;
                            let handler = webview2_com::CustomItemSelectedEventHandler::create(
                                Box::new(move |_sender: Option<ICoreWebView2ContextMenuItem>, _| {
                                    prm.emit(
                                        &snk,
                                        EngineEvent::ShortcutPressed {
                                            item: src.get(),
                                            command: format!("context.search={query}"),
                                        },
                                    );
                                    Ok(())
                                }),
                            );
                            if unsafe { item.add_CustomItemSelected(&handler, &mut token) }.is_ok()
                                && unsafe { items.InsertValueAtIndex(insert_at, &item) }.is_ok()
                            {
                                insert_at += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. "Ekran görüntüsü al" always available on webpage right-click.
    if let Ok(shot_item) = unsafe {
        env9.CreateContextMenuItem(
            &HSTRING::from("Ekran görüntüsü al"),
            None,
            COREWEBVIEW2_CONTEXT_MENU_ITEM_KIND_COMMAND,
        )
    } {
        let core_clone = core.clone();
        let mut token = 0i64;
        let handler = webview2_com::CustomItemSelectedEventHandler::create(Box::new(
            move |_sender: Option<ICoreWebView2ContextMenuItem>, _| {
                start_screenshot_overlay(core_clone.clone());
                Ok(())
            },
        ));
        if unsafe { shot_item.add_CustomItemSelected(&handler, &mut token) }.is_ok() {
            let _ = unsafe { items.InsertValueAtIndex(insert_at, &shot_item) };
        }
    }
}
