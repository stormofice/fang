async function saveOptions(e) {
    e.preventDefault();
    await browser.storage.sync.set({
        backend_url: document.querySelector("#backend_url").value,
        api_key: document.querySelector("#api_key").value,
        encryption_password: document.querySelector("#encryption_password").value,
    });
}

async function restoreOptions() {
    const options = await browser.storage.sync.get();
    document.querySelector("#backend_url").value = options.backend_url;
    document.querySelector("#api_key").value = options.api_key;
    document.querySelector("#encryption_password").value = options.encryption_password;
}

document.addEventListener('DOMContentLoaded', restoreOptions);
document.querySelector("form").addEventListener("submit", saveOptions);