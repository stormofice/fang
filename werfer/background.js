browser.runtime.onInstalled.addListener(async () => {
    console.log("Werfer loaded");

    await populateSettings();
});

async function populateSettings() {
    const options = await browser.storage.sync.get();
    const force = false;
    const dev = false;

    const defaults = {
        backend_url: dev ? "http://localhost:4567" : "set-your-own",
        api_key: dev ? "IHsw2IQoPYYVT5c8d9V2JRQ0JuPq27qV" : "set-your-own",
    };

    if (force) {
        await browser.storage.sync.set(defaults);
    }
}

async function setToolbarButton(saved) {
    console.log("Setting toolbar button", saved);
    // TODO(cross): I think svg icon support was not universal
    await browser.browserAction.setIcon({path: saved ? "icons/saved.svg" : "icons/save.svg"});
    await browser.browserAction.setTitle({title: saved ? "Forget" : "Save"});
}

// === Extension Internal Message Handling ===
// TODO(cross): I think chrome has other syntax for this
browser.runtime.onMessage.addListener((message, sender, sendResponse) => {
    console.log("[BG] Received runtime message", message);

    if ("action" in message) {
        switch (message.action) {
            case "tabInteraction":
                // Skip internal URLs
                if (message.url.startsWith("about:")) {
                    break;
                }

                // We do NOT await here, but I think it is fine. We do not care about a response and only want it done.
                // Adding async to the listener does not work (I think), due to the warnings on:
                // https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/runtime/onMessage
                // Let's just hope for the best 👍
                // Edit: I think this is an issue, noticed "Error: Promised response from onMessage listener went out of scope"
                //       I'll wait a bit to see if this matters matters or if it just matters.
                handleTabInteractionMessage(message.url, message.title).then((didSave) => {
                    sendResponse({didSave: didSave});
                }).catch(console.error);
                break;
            case "getFaenge":
                getAll().then((faenge) => sendResponse({faenge: faenge})).catch(console.error);
                break;
            case "forget":
                forgetUrl(message.url).then(() => console.log(`[BG] Forgot ${message.url}`)).catch(console.error)
                break;
            default:
                console.warn("[BG] Unhandled runtime message", message);
                break;
        }
    } else {
        console.error("Malformed message", message);
    }
    return true;
});

async function handleTabInteractionMessage(url, title) {
    if (await isUrlKnown(url)) {
        await forgetUrl(url);
        return false;
    } else {
        await saveTab(url, title);
        return true;
    }
}

// === Backend Handling & Caching ===
const knownUrlsCache = new Map();

async function isUrlKnown(url) {

    if (knownUrlsCache.has(url)) {
        return knownUrlsCache.get(url);
    }

    const options = await browser.storage.sync.get();

    // lol, you can await in string interpolation
    const resp = await fetch(`${options.backend_url}/faenge/has?url=${url}`, {
        headers: {
            "Content-Type": "application/json", "X-Api-Key": options.api_key
        }
    });
    if (resp.status === 302) {
        knownUrlsCache.set(url, true);
        return true;
    } else if (resp.status === 404) {
        knownUrlsCache.set(url, false);
        return false;
    } else {
        console.error(resp);
        return false;
    }
}

async function saveTab(url, title, set_btn = true) {
    const options = await browser.storage.sync.get();

    const resp = await fetch(`${options.backend_url}/faenge/save`, {
        method: "POST", body: JSON.stringify({
            url,
            title,
        }), headers: {"Content-Type": "application/json", "X-Api-Key": options.api_key},
    });

    if (resp.status === 200) {
        knownUrlsCache.set(url, true);
        if (set_btn)
            await setToolbarButton(true);
    } else {
        console.error(resp);
    }
}

async function forgetUrl(url, set_btn = true) {
    const options = await browser.storage.sync.get();

    const resp = await fetch(`${options.backend_url}/faenge/forget`, {
        method: "DELETE", body: JSON.stringify({
            url,
        }), headers: {"Content-Type": "application/json", "X-Api-Key": options.api_key},
    });

    if (resp.status === 200) {
        knownUrlsCache.set(url, false);
        if (set_btn)
            await setToolbarButton(false);
    } else {
        console.error(resp);
    }
}

async function getAll() {
    const options = await browser.storage.sync.get();
    const resp = await fetch(`${options.backend_url}/faenge/list`, {
        headers: {
            "Content-Type": "application/json", "X-Api-Key": options.api_key
        }
    });

    return await resp.json();
}

// === Tab handling ===

browser.tabs.onUpdated.addListener(async (tabId, changeInfo, tabInfo) => {
    if (changeInfo.status === "complete") {
        console.log("finished opening tab, checking if url is known");
        await setToolbarButton(await isUrlKnown(tabInfo.url));
    }
});

browser.tabs.onActivated.addListener(async ({tabId}) => {
    const tab = await browser.tabs.get(tabId);
    console.log("tab activated, checking url ", tab);
    await setToolbarButton(await isUrlKnown(tab.url));
});

function onCtxMenuCreated() {
    if (browser.runtime.lastError) {
        console.log(`[BG] Error while creating ctx menu: ${browser.runtime.lastError}`);
    }
}


// === Context menu setup ===
browser.contextMenus.create(
    {
        id: "save-link-ctx",
        title: "Save Link",
        contexts: ["link"],
        type: "normal",
    },
    onCtxMenuCreated,
);

browser.contextMenus.onClicked.addListener(async (info, _) => {
    console.log(`[BG] Clicked ctx item ${info}`)
    switch (info.menuItemId) {
        case "save-link-ctx":
            // TODO: This is not ideal, as the link text in most cases != the title when visiting the page
            await saveTab(info.linkUrl, info.linkText, false);
            console.log("[BG] Saved via ctx", info);
            break;
        default:
            console.warn("[BG] Unhandled context click", info);
            break;
    }
});