browser.runtime.onInstalled.addListener(() => {
    console.log("Werfer loaded");
});

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
            case "saveUrl":
                saveTab(message.url, message.title);
                break;
        }
    } else {
        console.error("Malformed message", message);
    }
});


// === Backend Handling & Caching ===
const knownUrlsCache = new Map();

async function isUrlKnown(url) {

    if (knownUrlsCache.has(url)) {
        return knownUrlsCache.get(url);
    }

    const options = await browser.storage.sync.get();

    const resp = await fetch(`${options.backend_url}/faenge/has?url=${url}`, {headers: {"Content-Type": "application/json", "X-Api-Key": options.api_key}});
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
    if (await isUrlKnown(url)) {
        // TODO: Unsave the tab
        return;
    }

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