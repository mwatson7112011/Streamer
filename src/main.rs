use std::{
    sync::{Arc, Mutex},
};
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowBuilder},
};
use wry::{http::Response, WebView, WebViewBuilder};

// ===== Embedded assets (no ui folder needed at runtime) =====
const INDEX_HTML: &str = include_str!("../ui/index.html");

const PEACOCK_JPG: &[u8] = include_bytes!("../ui/peacock.jpg");
const MAX_JPG: &[u8] = include_bytes!("../ui/Max.jpg");
const DISNEY_JPG: &[u8] = include_bytes!("../ui/disney+.jpg");
const STARZ_JPG: &[u8] = include_bytes!("../ui/starz.jpg");
const PARAMOUNT_JPG: &[u8] = include_bytes!("../ui/paramount+.jpg");
const NETFLIX_JPG: &[u8] = include_bytes!("../ui/netflix.jpg");
const HULU_JPG: &[u8] = include_bytes!("../ui/hulu.jpg");
const TUBI_JPG: &[u8] = include_bytes!("../ui/tubi.jpg");
const APPLETV_PNG: &[u8] = include_bytes!("../ui/appletv.png");

fn main() -> Result<(), wry::Error> {
    let event_loop = EventLoop::new();
    let proxy = event_loop.create_proxy();

    let nav_queue: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let nav_queue_clone_for_ipc = nav_queue.clone();

    let window = WindowBuilder::new()
        .with_title("Streamer")
        .with_fullscreen(Some(Fullscreen::Borderless(None)))
        .build(&event_loop)
        .unwrap();

    let is_on_hub = Arc::new(Mutex::new(true));
    let is_on_hub_clone = is_on_hub.clone();

    let webview_holder: Arc<Mutex<Option<WebView>>> = Arc::new(Mutex::new(None));

    // ===== Overlay injected into external sites =====
    let injected_ui = r#"
        window.addEventListener("DOMContentLoaded", () => {
            const oldHost = document.getElementById('streamer-host');
            if (oldHost) oldHost.remove();

            const oldHoverZone = document.getElementById('streamer-hover');
            if (oldHoverZone) oldHoverZone.remove();

            const hoverZone = document.createElement('div');
            hoverZone.id = 'streamer-hover';
            hoverZone.style.position = 'fixed';
            hoverZone.style.top = '0';
            hoverZone.style.left = '0';
            hoverZone.style.width = '100%';
            hoverZone.style.height = '20px';
            hoverZone.style.zIndex = '999999998';
            hoverZone.style.pointerEvents = 'auto';
            hoverZone.style.background = 'transparent';
            document.documentElement.appendChild(hoverZone);

            const host = document.createElement('div');
            host.id = 'streamer-host';
            host.style.position = 'fixed';
            host.style.top = '0';
            host.style.left = '0';
            host.style.width = '100%';
            host.style.zIndex = '999999999';
            document.documentElement.appendChild(host);

            const shadow = host.attachShadow({ mode: 'open' });

            const style = document.createElement('style');
            style.textContent = `
                #bar {
                    position: fixed;
                    top: -90px;
                    left: 0;
                    width: 100%;
                    height: 80px;
                    backdrop-filter: blur(20px);
                    background: rgba(20, 20, 20, 0.5);
                    border-bottom: 1px solid rgba(255, 255, 255, 0.4);
                    box-shadow: 0 4px 30px rgba(0, 0, 0, 0.2);
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    gap: 40px;
                    transition: top 0.25s ease;
                    font-family: sans-serif;
                }
                button {
                    padding: 12px 28px;
                    font-size: 18px;
                    border-radius: 10px;
                    border: none;
                    background: rgba(75, 75, 75, 0.6);
                    color: gold;
                    font-weight: bold;
                    cursor: pointer;
                    backdrop-filter: blur(10px);
                    box-shadow: 0 2px 10px rgba(0,0,0,0.2);
                    transition: transform 0.2s ease, background 0.2s ease;
                }
                button:hover {
                    transform: scale(1.08);
                    background: rgba(255, 255, 255, 0.8);
                }
            `;
            shadow.appendChild(style);

            const bar = document.createElement('div');
            bar.id = 'bar';
            bar.innerHTML = `
                <button id="backHub">Back to Hub</button>
                <button id="exitApp">Exit App</button>
            `;
            shadow.appendChild(bar);

            hoverZone.addEventListener('mouseenter', () => {
                bar.style.top = "0px";
            });

            bar.addEventListener('mouseleave', () => {
                bar.style.top = "-90px";
            });

            const backBtn = shadow.querySelector('#backHub');
            if (backBtn) {
                backBtn.addEventListener('click', (e) => {
                    e.stopPropagation();
                    e.preventDefault();
                    window.ipc.postMessage("go_home");
                });
            }

            const exitBtn = shadow.querySelector('#exitApp');
            if (exitBtn) {
                exitBtn.addEventListener('click', (e) => {
                    e.stopPropagation();
                    window.ipc.postMessage("exit_app");
                });
            }

            bar.addEventListener('click', (e) => e.stopPropagation());
        });
    "#;

    let webview = WebViewBuilder::new(&window)
        .with_initialization_script(injected_ui)
        .with_custom_protocol("streamer".into(), move |request| {
            let mut path = request.uri().path().trim_start_matches('/').to_string();

            if let Some(pos) = path.find('?') {
                path.truncate(pos);
            }

            if path.is_empty() {
                path = "index.html".to_string();
            }

            if path.ends_with('/') {
                path.pop();
            }

            // ===== Serve everything from embedded assets =====
            let body: Vec<u8> = match path.as_str() {
                "" | "index.html" => INDEX_HTML.as_bytes().to_vec(),
                "peacock.jpg" => PEACOCK_JPG.to_vec(),
                "Max.jpg" => MAX_JPG.to_vec(),
                "disney+.jpg" => DISNEY_JPG.to_vec(),
                "starz.jpg" => STARZ_JPG.to_vec(),
                "paramount+.jpg" => PARAMOUNT_JPG.to_vec(),
                "netflix.jpg" => NETFLIX_JPG.to_vec(),
                "hulu.jpg" => HULU_JPG.to_vec(),
                "tubi.jpg" => TUBI_JPG.to_vec(),
                "appletv.png" => APPLETV_PNG.to_vec(),
                _ => b"Not found".to_vec(),
            };

            let mime = if path.ends_with(".html") {
                "text/html"
            } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
                "image/jpeg"
            } else if path.ends_with(".png") {
                "image/png"
            } else if path.ends_with(".css") {
                "text/css"
            } else if path.ends_with(".js") {
                "text/javascript"
            } else {
                "application/octet-stream"
            };

            Response::builder()
                .header("Content-Type", mime)
                .body(std::borrow::Cow::Owned(body))
                .unwrap()
        })
        .with_url("streamer://index.html")
        .with_ipc_handler(move |request| {
            let msg = request.body();

            if msg == "go_home" {
                {
                    let mut hub_state = is_on_hub_clone.lock().unwrap();
                    *hub_state = true;
                }

                {
                    let mut q = nav_queue_clone_for_ipc.lock().unwrap();
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis();
                    q.push(format!("streamer://index.html?t={}", timestamp));
                }

                let _ = proxy.send_event(());
                return;
            }

            if msg == "exit_app" {
                std::process::exit(0);
            }

            if msg.starts_with("open:") {
                let service = msg.replace("open:", "");

                let url = match service.as_str() {
                    "peacock" => "https://www.peacocktv.com",
                    "hbomax" => "https://www.max.com",
                    "disney" => "https://www.disneyplus.com",
                    "starz" => "https://www.starz.com",
                    "paramount" => "https://www.paramountplus.com",
                    "netflix" => "https://www.netflix.com",
                    "hulu" => "https://www.hulu.com",
                    "tubi" => "https://www.tubitv.com",
                    "appletv" => "https://tv.apple.com/",
                    _ => "https://www.google.com",
                };

                {
                    let mut hub_state = is_on_hub_clone.lock().unwrap();
                    *hub_state = false;
                }

                {
                    let mut q = nav_queue_clone_for_ipc.lock().unwrap();
                    q.push(url.into());
                }

                let _ = proxy.send_event(());
            }
        })
        .build()?;

    *webview_holder.lock().unwrap() = Some(webview);

        event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(()) => {
                let mut to_navigate = Vec::new();
                {
                    let mut q = nav_queue.lock().unwrap();
                    to_navigate.append(&mut *q);
                }

                if !to_navigate.is_empty() {
                    if let Some(ref webview) = *webview_holder.lock().unwrap() {
                        for url in to_navigate {
                            if url.starts_with("streamer://") {
                                // ===== Rebuild hub using embedded images (base64) =====
                                let images: Vec<&[u8]> = vec![
                                    PEACOCK_JPG,
                                    MAX_JPG,
                                    DISNEY_JPG,
                                    STARZ_JPG,
                                    PARAMOUNT_JPG,
                                    NETFLIX_JPG,
                                    HULU_JPG,
                                    TUBI_JPG,
                                    APPLETV_PNG,
                                ];

                                let mut image_data_uris = Vec::new();
                                for data in images {
                                    let base64 = base64_encode(data);
                                    let data_uri = format!("data:image/jpeg;base64,{}", base64);
                                    image_data_uris.push(data_uri);
                                }

                                let rebuild_script = format!(
                                    r#"
                                        const imageUris = {};

                                        window.openService = function(service) {{
                                            window.ipc.postMessage('open:' + service);
                                        }};

                                        document.documentElement.innerHTML = `
                                        <!DOCTYPE html>
                                        <html>
                                        <head>
                                            <meta charset="UTF-8">
                                            <meta name="viewport" content="width=device-width, initial-scale=1.0">
                                            <title>Streamer Hub</title>
                                            <style>
                                                * {{ margin: 0; padding: 0; box-sizing: border-box; }}
                                                body {{
                                                    background: #000;
                                                    font-family: Arial, sans-serif;
                                                    display: flex;
                                                    align-items: center;
                                                    justify-content: center;
                                                    min-height: 100vh;
                                                    padding: 20px;
                                                }}
                                                .grid {{
                                                    display: grid;
                                                    grid-template-columns: repeat(3, 1fr);
                                                    gap: 40px;
                                                    max-width: 900px;
                                                }}
                                                img {{
                                                    width: 260px;
                                                    height: auto;
                                                    border-radius: 12px;
                                                    cursor: pointer;
                                                    animation: fadeIn 1s forwards, breathe 4s ease-in-out infinite;
                                                    border: 3px solid transparent;
                                                    transition: transform 0.3s ease;
                                                }}
                                                img:hover {{
                                                    transform: scale(1.1);
                                                    box-shadow: 0 0 30px rgba(255, 215, 0, 0.8);
                                                }}
                                                @keyframes fadeIn {{
                                                    from {{ opacity: 0; }}
                                                    to {{ opacity: 1; }}
                                                }}
                                                @keyframes breathe {{
                                                    0%, 100% {{ transform: scale(1); }}
                                                    50% {{ transform: scale(1.02); }}
                                                }}
                                            </style>
                                        </head>
                                        <body>
                                            <div class="grid">
                                                <img src="${{imageUris[0]}}" alt="Peacock" data-service="peacock">
                                                <img src="${{imageUris[1]}}" alt="HBO Max" data-service="hbomax">
                                                <img src="${{imageUris[2]}}" alt="Disney+" data-service="disney">
                                                <img src="${{imageUris[3]}}" alt="Starz" data-service="starz">
                                                <img src="${{imageUris[4]}}" alt="Paramount+" data-service="paramount">
                                                <img src="${{imageUris[5]}}" alt="Netflix" data-service="netflix">
                                                <img src="${{imageUris[6]}}" alt="Hulu" data-service="hulu">
                                                <img src="${{imageUris[7]}}" alt="Tubi" data-service="tubi">
                                                <img src="${{imageUris[8]}}" alt="Apple TV" data-service="appletv">
                                            </div>
                                        </body>
                                        </html>
                                        `;

                                        setTimeout(() => {{
                                            const images = document.querySelectorAll('img[data-service]');
                                            images.forEach(img => {{
                                                img.addEventListener('click', () => {{
                                                    const service = img.getAttribute('data-service');
                                                    window.openService(service);
                                                }});
                                            }});
                                        }}, 5);

                                        setTimeout(() => {{
                                            const oldHost = document.getElementById('streamer-host');
                                            if (oldHost) oldHost.remove();

                                            const oldHoverZone = document.getElementById('streamer-hover');
                                            if (oldHoverZone) oldHoverZone.remove();

                                            const hoverZone = document.createElement('div');
                                            hoverZone.id = 'streamer-hover';
                                            hoverZone.style.position = 'fixed';
                                            hoverZone.style.top = '0';
                                            hoverZone.style.left = '0';
                                            hoverZone.style.width = '100%';
                                            hoverZone.style.height = '20px';
                                            hoverZone.style.zIndex = '999999998';
                                            hoverZone.style.pointerEvents = 'auto';
                                            hoverZone.style.background = 'transparent';
                                            document.documentElement.appendChild(hoverZone);

                                            const host = document.createElement('div');
                                            host.id = 'streamer-host';
                                            host.style.position = 'fixed';
                                            host.style.top = '0';
                                            host.style.left = '0';
                                            host.style.width = '100%';
                                            host.style.zIndex = '999999999';
                                            document.documentElement.appendChild(host);

                                            const shadow = host.attachShadow({{ mode: 'open' }});

                                            const style = document.createElement('style');
                                            style.textContent = `
                                                #bar {{
                                                    position: fixed;
                                                    top: -90px;
                                                    left: 0;
                                                    width: 100%;
                                                    height: 80px;
                                                    backdrop-filter: blur(20px);
                                                    background: rgba(20, 20, 20, 0.5);
                                                    border-bottom: 1px solid rgba(255, 255, 255, 0.4);
                                                    box-shadow: 0 4px 30px rgba(0, 0, 0, 0.2);
                                                    display: flex;
                                                    align-items: center;
                                                    justify-content: center;
                                                    gap: 40px;
                                                    transition: top 0.25s ease;
                                                    font-family: sans-serif;
                                                }}
                                                button {{
                                                    padding: 12px 28px;
                                                    font-size: 18px;
                                                    border-radius: 10px;
                                                    border: none;
                                                    background: rgba(75, 75, 75, 0.6);
                                                    color: gold;
                                                    font-weight: bold;
                                                    cursor: pointer;
                                                    backdrop-filter: blur(10px);
                                                    box-shadow: 0 2px 10px rgba(0,0,0,0.2);
                                                    transition: transform 0.2s ease, background 0.2s ease;
                                                }}
                                                button:hover {{
                                                    transform: scale(1.08);
                                                    background: rgba(255, 255, 255, 0.8);
                                                }}
                                            `;
                                            shadow.appendChild(style);

                                            const bar = document.createElement('div');
                                            bar.id = 'bar';
                                            bar.innerHTML = `
                                                <button id="backHub">Back to Hub</button>
                                                <button id="exitApp">Exit App</button>
                                            `;
                                            shadow.appendChild(bar);

                                            hoverZone.addEventListener('mouseenter', () => {{
                                                bar.style.top = "0px";
                                            }});

                                            bar.addEventListener('mouseleave', () => {{
                                                bar.style.top = "-90px";
                                            }});

                                            const backBtn = shadow.querySelector('#backHub');
                                            if (backBtn) {{
                                                backBtn.addEventListener('click', (e) => {{
                                                    e.stopPropagation();
                                                    e.preventDefault();
                                                    window.ipc.postMessage("go_home");
                                                }});
                                            }}

                                            const exitBtn = shadow.querySelector('#exitApp');
                                            if (exitBtn) {{
                                                exitBtn.addEventListener('click', (e) => {{
                                                    e.stopPropagation();
                                                    window.ipc.postMessage("exit_app");
                                                }});
                                            }}

                                            bar.addEventListener('click', (e) => e.stopPropagation());
                                        }}, 10);
                                    "#,
                                    serde_json::to_string(&image_data_uris).unwrap()
                                );

                                let _ = webview.evaluate_script(&rebuild_script);
                            } else {
                                let _ = webview.load_url(&url);
                            }
                        }
                    }
                }
            }
            Event::WindowEvent { event, .. } => {
                if let WindowEvent::CloseRequested = event {
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
    });
}

fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::new();

    for chunk in data.chunks(3) {
        let b1 = chunk[0];
        let b2 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b3 = if chunk.len() > 2 { chunk[2] } else { 0 };

        let n = ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);

        result.push(TABLE[((n >> 18) & 0x3F) as usize] as char);
        result.push(TABLE[((n >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            result.push(TABLE[((n >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(TABLE[(n & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }

    result
}