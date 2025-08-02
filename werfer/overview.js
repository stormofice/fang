const overviewContainer = document.getElementsByClassName("overview_url_view")[0];

browser.runtime.sendMessage({action: "getFaenge"}).then(
    (response) => {
        response.faenge.forEach((entry) => {
            const template = document.getElementById("url_entry");
            const fangCard = template.content.cloneNode(true);

            const fangTitleP = fangCard.querySelector(".url_entry_title");
            fangTitleP.textContent = entry.title;


            const fangUrlA = fangCard.querySelector(".url_entry_url");
            fangUrlA.href = entry.url;
            fangUrlA.textContent = entry.url;


            overviewContainer.appendChild(fangCard);
        });
    }
);
