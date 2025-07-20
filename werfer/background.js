browser.runtime.onInstalled.addListener(() => {
    console.log("Werfer loaded");
});

browser.pageAction.onClicked.addListener(async (currentTab) => {
    const options = await browser.storage.sync.get();

    const resp = await fetch(`${options.backend_url}/faenge/save`, {
        method: "POST",
        body: JSON.stringify({
            title: currentTab.title,
            url: currentTab.url,
        }),
        headers: {"Content-Type": "application/json", "X-Api-Key": options.api_key},
    });
    console.log("got response", resp);
});