
browser.runtime.onInstalled.addListener(() => {
    console.log("Werfer loaded");
});

browser.pageAction.onClicked.addListener(() => {
    console.log("Click");
});