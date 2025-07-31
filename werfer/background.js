browser.runtime.onInstalled.addListener(async () => {
    console.log("Werfer loaded");

    await populateSettings();
});

async function populateSettings() {
    const options = await browser.storage.sync.get();
    if (options.backend_url === undefined) {
        options.backend_url = "http://localhost:4567";
    }
    if (options.encryption_key === undefined) {
        options.encryption_key = "none";
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
                // We do NOT await here, but I think it is fine. We do not care about a response and only want it done.
                // Adding async to the listener does not work (I think), due to the warnings on:
                // https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/runtime/onMessage
                // Let's just hope for the best 👍
                handleTabInteractionMessage(message.url, message.title).catch(console.error);
                break;
            default:
                console.warn("[BG} Unhandled runtime message", message);
                break;
        }
    } else {
        console.error("Malformed message", message);
    }
});

async function handleTabInteractionMessage(url, title) {
    if (await isUrlKnown(url)) {
        await forgetUrl(url);
    } else {
        await saveTab(url, title);
    }
}

// === Backend Handling & Caching ===

async function encrypt(text) {
    const options = await browser.storage.sync.get();
    if (options.encryption_key !== "none") {}
    else {
        return text;
    }
}

const knownUrlsCache = new Map();

async function isUrlKnown(url) {

    if (knownUrlsCache.has(url)) {
        return knownUrlsCache.get(url);
    }

    const options = await browser.storage.sync.get();

    const resp = await fetch(`${options.backend_url}/faenge/has?url=${url}`, {
        headers: {
            "Content-Type": "application/json",
            "X-Api-Key": options.api_key
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

async function saveTab(url, title) {
    const options = await browser.storage.sync.get();

    const resp = await fetch(`${options.backend_url}/faenge/save`, {
        method: "POST",
        body: JSON.stringify({
            title: title,
            url: url,
        }),
        headers: {"Content-Type": "application/json", "X-Api-Key": options.api_key},
    });

    if (resp.status === 200) {
        knownUrlsCache.set(url, true);
        await setToolbarButton(true);
    } else {
        console.error(resp);
    }
}

async function forgetUrl(url) {
    const options = await browser.storage.sync.get();

    const resp = await fetch(`${options.backend_url}/faenge/forget`, {
        method: "DELETE",
        body: JSON.stringify({
            url: url,
        }),
        headers: {"Content-Type": "application/json", "X-Api-Key": options.api_key},
    });

    if (resp.status === 200) {
        knownUrlsCache.set(url, false);
        await setToolbarButton(false);
    } else {
        console.error(resp);
    }
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
    console.log("tab activated, checking url. ", tab);
    await setToolbarButton(await isUrlKnown(tab.url));
});