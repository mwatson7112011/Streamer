<<<<<<< HEAD
=======
<<<<<<< HEAD
>>>>>>> 76a7c7276b3f900f511887a091fa0d30c186f720
Streamer (Rust + Tao/Wry)
Streamer is a fullscreen desktop streaming launcher built using Rust, Tao, and Wry.
Unlike the web version, this desktop version can load streaming services such as Netflix, Hulu, Max, Paramount+, and others inside the application window, thanks to a native WebView.

This README explains:

Why the HTML and images were embedded directly into the Rust binary

How the custom protocol (streamer://) works

How the Shadow DOM overlay was injected into external websites

How navigation, IPC, and the hub rebuild system worked

Why this architecture allowed streaming services to load without breaking

Why the HTML Was Embedded Inside main.rs
In the Rust version, the UI files were not loaded from disk.
Instead, they were compiled directly into the binary using:


const INDEX_HTML: &str = include_str!("../ui/index.html");
const PEACOCK_JPG: &[u8] = include_bytes!("../ui/peacock.jpg");


This was done for three reasons:

1. Portability
The app becomes a single executable with no external folders required.

2. Security
No external HTML or image files can be modified or deleted by the user.

3. Custom Protocol Support
The app serves its own HTML and images through a virtual protocol:

streamer://index.html


This allows the WebView to load local assets without using file://, which many streaming services block.

How the Custom Protocol Works
This section of the code intercepts all requests to streamer://...:

.with_custom_protocol("streamer".into(), move |request| {
    let path = request.uri().path();
    match path {
        "index.html" => INDEX_HTML.as_bytes().to_vec(),
        "peacock.jpg" => PEACOCK_JPG.to_vec(),
        ...
    }
})


This system:

Pretends to be a real web server

Serves HTML and images directly from memory

Avoids CORS issues

Avoids filesystem access

Makes the app self‑contained

This is why the hub loads instantly and reliably

Why Streaming Services Load in the Desktop Version (But Not in Browsers)
Web browsers block embedding of major streaming services using headers like:

X-Frame-Options: SAMEORIGIN
Content-Security-Policy: frame-ancestors 'none'


A native WebView is not an iframe.
It is a full browser engine instance, so these restrictions do not apply.

This is why:

Netflix loads

Hulu loads

Max loads

Paramount+ loads

Apple TV loads

The desktop version behaves like a standalone browser.

How the Shadow DOM Overlay Works
This is the most advanced part of your architecture.

You inject a UI overlay into every external website using:

.with_initialization_script(injected_ui)


The injected JavaScript:

1. Creates a fixed-position host element

const host = document.createElement('div');
host.id = 'streamer-host';
document.documentElement.appendChild(host);


2. Attaches a Shadow DOM

const shadow = host.attachShadow({ mode: 'open' });
<<<<<<< HEAD
=======



This isolates your UI from:

The website’s CSS

The website’s JavaScript

Conflicting styles

DOM mutations

3. Builds a custom top bar inside the Shadow DOM

bar.innerHTML = `
    <button id="backHub">Back to Hub</button>
    <button id="exitApp">Exit App</button>
`;



4. Adds hover‑to‑reveal behavior


hoverZone.addEventListener('mouseenter', () => {
    bar.style.top = "0px";
});


5. Sends IPC messages back to Rust

window.ipc.postMessage("go_home");
window.ipc.postMessage("exit_app");


This allows the overlay to control the entire application.



How Navigation Works
When a tile is clicked, JavaScript sends:


open:netflix


Rust receives it:


if msg.starts_with("open:") {
    q.push(url.into());
    proxy.send_event(());
}


if msg.starts_with("open:") {
    q.push(url.into());
    proxy.send_event(());
}

How the Hub Rebuild System Works
When returning to the hub, Rust rebuilds the entire HTML dynamically:

document.documentElement.innerHTML = `...grid HTML...`;

Then it reattaches:

Click handlers

Animations

Shadow DOM overlay

This ensures the hub always loads cleanly, even after visiting external sites.

Why This Architecture Works So Well
1. The WebView bypasses iframe restrictions
This is the key reason streaming services load.

2. The Shadow DOM prevents CSS conflicts
External sites cannot break your UI.

3. The custom protocol makes the app self‑contained
No external files are needed.

4. IPC provides clean communication between JS and Rust
Navigation is instant and reliable.

5. The hub rebuild ensures a consistent UI
No matter what site the user visits.
=======
>>>>>>> 76a7c7276b3f900f511887a091fa0d30c186f720



This isolates your UI from:

The website’s CSS

The website’s JavaScript

Conflicting styles

DOM mutations

3. Builds a custom top bar inside the Shadow DOM

bar.innerHTML = `
    <button id="backHub">Back to Hub</button>
    <button id="exitApp">Exit App</button>
`;



4. Adds hover‑to‑reveal behavior


hoverZone.addEventListener('mouseenter', () => {
    bar.style.top = "0px";
});


5. Sends IPC messages back to Rust

window.ipc.postMessage("go_home");
window.ipc.postMessage("exit_app");


This allows the overlay to control the entire application.



How Navigation Works
When a tile is clicked, JavaScript sends:


open:netflix


Rust receives it:


if msg.starts_with("open:") {
    q.push(url.into());
    proxy.send_event(());
}


<<<<<<< HEAD
if msg.starts_with("open:") {
    q.push(url.into());
    proxy.send_event(());
}

How the Hub Rebuild System Works
When returning to the hub, Rust rebuilds the entire HTML dynamically:

document.documentElement.innerHTML = `...grid HTML...`;

Then it reattaches:

Click handlers

Animations

Shadow DOM overlay

This ensures the hub always loads cleanly, even after visiting external sites.

Why This Architecture Works So Well
1. The WebView bypasses iframe restrictions
This is the key reason streaming services load.

2. The Shadow DOM prevents CSS conflicts
External sites cannot break your UI.

3. The custom protocol makes the app self‑contained
No external files are needed.

4. IPC provides clean communication between JS and Rust
Navigation is instant and reliable.

5. The hub rebuild ensures a consistent UI
No matter what site the user visits.
=======
>>>>>>> 29101c4dd0941d2a15ff6501d1e2d67082c20fc2










>>>>>>> 76a7c7276b3f900f511887a091fa0d30c186f720
