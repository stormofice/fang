const overviewContainer = document.getElementsByClassName("overview_url_view")[0];

const resolveMap = new Map();

browser.runtime.sendMessage({action: "getFaenge"}).then(
    (response) => {
        response.faenge.sort((a, b) => {
            if (a.time_created < b.time_created) return 1;
            if (a.time_created > b.time_created) return -1;
            return 0;
        }).forEach((entry) => {
            const template = document.getElementById("url_entry");
            const fangCard = template.content.cloneNode(true);

            const fangCardElement = fangCard.querySelector(".overview_url_card");

            const fangTitleP = fangCard.querySelector(".url_entry_title");
            fangTitleP.textContent = limitString(entry.title);

            const fangTimeCreatedP = fangCard.querySelector(".url_entry_time_created");
            fangTimeCreatedP.textContent = entry.time_created;

            const fangUrlA = fangCard.querySelector(".url_entry_url");
            fangUrlA.href = entry.url;
            fangUrlA.textContent = limitString(entry.url);

            const fangForgetButton = fangCard.querySelector(".url_entry_forget_btn");
            fangForgetButton.addEventListener("click", (event) => {
                console.log("Want to forget fang from overview", entry);
                fangCardElement.remove();
                browser.runtime.sendMessage({action: "forget", url: entry.url});
            });

            const fangResolveLobstersButton = fangCard.querySelector(".url_entry_resolve_lobsters_btn");
            fangResolveLobstersButton.addEventListener("click", async _ => {
                console.log("Trying to resolve on lobsters", entry);

                const resolveEntry = resolveMap.get(entry.url);
                if (resolveEntry) {
                    // TODO: Also here, just choose first for now
                    window.open(resolveEntry.lobsters_links[0], "_blank");
                    return;
                }

                const options = await browser.storage.sync.get();
                const response = await fetch(`${options.backend_url}/backlinks/resolve?url=${entry.url}`, {
                    headers: {
                        "Content-Type": "application/json", "X-Api-Key": options.api_key
                    }
                });

                // {lobsters_links: [], hn_links: []}
                const backlinks = await response.json();
                console.log(backlinks);

                if (response.status === 404 || backlinks.lobsters_links == null) {
                    fangResolveLobstersButton.textContent = "No link!";
                    fangResolveLobstersButton.style.color = "#f38ba8";
                } else {
                    // TODO: For now just use the first one
                    let first = backlinks.lobsters_links[0];

                    console.log("Found story", first);
                    window.open(first, "_blank");
                    // TODO: Unify colors
                    fangResolveLobstersButton.style.color = "#a6e3a1";
                    fangResolveLobstersButton.textContent = "Open Lobsters";
                    resolveMap.set(entry.url, backlinks);
                }
            });

            overviewContainer.appendChild(fangCard);
        });
    }
);

function limitString(str) {
    if (str.length > 120) {
        return str.substring(0, 120) + "[...]";
    }
    return str;
}
