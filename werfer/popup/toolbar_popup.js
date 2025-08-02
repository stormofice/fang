console.log("popup opened");

// Get active tab and send to BG
// yeah i love mixing promise handling patterns, why do you ask?
browser.tabs.query({currentWindow: true, active: true}).then((tabs) => {
    if (tabs.length !== 1) {
        console.error("Unexpected active tab count", tabs);
    } else {
        const activeTab = tabs[0];
        browser.runtime.sendMessage({action: "tabInteraction", url: activeTab.url, title: activeTab.title});
    }
}, (err) => console.error(err));

const overview_btn = document.getElementById("overview_btn");
overview_btn.addEventListener("click", async () => {
    console.log("Overview btn click");
    await browser.tabs.create({url: "/overview.html"});
});