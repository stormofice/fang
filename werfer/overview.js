const overviewContainer = document.getElementsByClassName("overview_url_view")[0];

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