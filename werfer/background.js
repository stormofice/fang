browser.runtime.onInstalled.addListener(() => {
    console.log("Werfer loaded");
});

async function setToolbarButton(saved) {
    console.log("Setting toolbar button", saved);
    await browser.browserAction.setIcon({path: saved ? "icons/saved.svg" : "icons/save.svg"});
    await browser.browserAction.setTitle({title: saved ? "Forget" : "Save"});
}

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

async function saveTab(tabInfo) {
    const options = await browser.storage.sync.get();

    const resp = await fetch(`${options.backend_url}/faenge/save`, {
        method: "POST",
        body: JSON.stringify({
            title: tabInfo.title,
            url: tabInfo.url,
        }),
        headers: {"Content-Type": "application/json", "X-Api-Key": options.api_key},
    });

    if (resp.status === 200) {
        knownUrlsCache.set(tabInfo.url, true);
        await setToolbarButton(true);
    } else {
        console.error(resp);
    }
}

browser.browserAction.onClicked.addListener(async (currentTab) => {
    if (await isUrlKnown(currentTab.url)) {
        /// Unsave
        return;
    }
    await saveTab(currentTab);
});

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