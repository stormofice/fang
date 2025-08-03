console.log("popup opened");


const overviewBtn = document.getElementById("overview_btn");
const actionTitle = document.getElementById("action_title");

// Get active tab and send to BG
// yeah i love mixing promise handling patterns, why do you ask?
browser.tabs.query({currentWindow: true, active: true}).then((tabs) => {
    if (tabs.length !== 1) {
        console.error("Unexpected active tab count", tabs);
    } else {
        const activeTab = tabs[0];
        browser.runtime.sendMessage({action: "tabInteraction", url: activeTab.url, title: activeTab.title}).then(
            (response) => {
                actionTitle.textContent = response.didSave ? "Nice catch!" : "I forgor :(";
                actionTitle.classList.add(response.didSave ? "green" : "red");
            }
        );
    }
}, (err) => console.error(err));

overviewBtn.addEventListener("click", async () => {
    console.log("Overview btn click");
    await browser.tabs.create({url: "/overview.html"});
});