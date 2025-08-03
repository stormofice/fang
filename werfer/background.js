browser.runtime.onInstalled.addListener(async () => {
    console.log("Werfer loaded");

    await populateSettings();
});

async function populateSettings() {
    const options = await browser.storage.sync.get();
    const force = true;
    if (force || options.backend_url === undefined) {
        await browser.storage.sync.set({backend_url: "http://localhost:4567"});
    }
    if (force || options.api_key === undefined) {
        await browser.storage.sync.set({api_key: "Is8SzdXblgwet5Z7xFw1Fyqz1ctDK0HO"});
    }
    if (force || options.encryption_password === undefined) {
        await browser.storage.sync.set({encryption_password: "test123test123"});
    }

}

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
            case "tabInteraction":
                // We do NOT await here, but I think it is fine. We do not care about a response and only want it done.
                // Adding async to the listener does not work (I think), due to the warnings on:
                // https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/runtime/onMessage
                // Let's just hope for the best 👍
                handleTabInteractionMessage(message.url, message.title).then((didSave) => {
                    sendResponse({didSave: didSave});
                }).catch(console.error);
                break;
            case "getFaenge":
                getAll().then((faenge) => sendResponse({faenge: faenge})).catch(console.error);
                break;
            default:
                console.warn("[BG] Unhandled runtime message", message);
                break;
        }
    } else {
        console.error("Malformed message", message);
    }
    return true;
});

async function handleTabInteractionMessage(url, title) {
    if (await isUrlKnown(url)) {
        await forgetUrl(url);
        return false;
    } else {
        await saveTab(url, title);
        return true;
    }
}

// === Backend Handling & Caching ===

// Best effort encryption attempt x)
async function encryptData(text) {
    const options = await browser.storage.sync.get();

    // TODO: Think about how to handle "mixed key" scenarios
    //  I.e. when a user has started with no encryption and turns it on, or when they change keys.
    //  Right now, this would break the application, as all checks would now work using the new key (or none).
    //  Basically, we'd require a migration process to happen.
    //  I think this can be ignored for now, but should be handled later on.
    // TODO: Additionally, all of this only works because we do a simple .eq check for urls in the backend.
    //  This should probably be written done somewhere

    // Should we encrypt at all?
    if (options.encryption_password === undefined) {
        return text;
    }

    // Generate key if we did not do it yet
    // TODOx: Should this be a) synced b) persisted?
    //  I think it does not matter security wise, at least when we save the pw anyway
    //  Never mind, can't store key objects anyway

    const enc = new TextEncoder();
    const pwKeyMaterial = await crypto.subtle.importKey("raw", enc.encode(options.encryption_password), "PBKDF2", false, ["deriveKey"]);

    // Honestly, don't really know about the salt, should be fine I guess
    const salt = crypto.getRandomValues(new Uint8Array(32));

    // I think we can crank the iterations, as we ~~derive the key only once?~~ are ballin
    const key = await crypto.subtle.deriveKey({
        name: "PBKDF2",
        hash: "SHA-256",
        iterations: 600_000,
        salt: salt,
    }, pwKeyMaterial, {name: "AES-GCM", length: 256}, false, ["encrypt"]);

    // TODO: Investigate slow down due to keygen / encryption

    const iv = crypto.getRandomValues(new Uint8Array(32));

    const ciphertext = await crypto.subtle.encrypt({name: "AES-GCM", iv}, key, enc.encode(text));

    const combined = new Uint8Array(salt.length + iv.length + ciphertext.byteLength);
    combined.set(salt, 0);
    combined.set(iv, salt.length);
    combined.set(new Uint8Array(ciphertext), salt.length + iv.length);


    // Base64 repr of combined encryption stuff
    return btoa(String.fromCharCode(...combined));
}

async function decryptData(data) {
    const options = await browser.storage.sync.get();

    if (options.encryption_password === undefined) {
        return data;
    }
    const enc = new TextEncoder();
    const pwKeyMaterial = await crypto.subtle.importKey("raw", enc.encode(options.encryption_password), "PBKDF2", false, ["deriveKey"]);

    const dataBytes = new Uint8Array(atob(data).split('').map(c => c.charCodeAt(0)));
    const salt = dataBytes.slice(0, 32);
    const iv = dataBytes.slice(32, 64);
    const encData = dataBytes.slice(64);

    const key = await crypto.subtle.deriveKey({
        name: "PBKDF2",
        hash: "SHA-256",
        iterations: 600_000,
        salt: salt,
    }, pwKeyMaterial, {name: "AES-GCM", length: 256}, false, ["decrypt"]);

    const decrypted = await crypto.subtle.decrypt({name: "AES-GCM", iv}, key, encData);
    return JSON.parse(new TextDecoder().decode(decrypted));
}

async function deriveLookupUrl(url) {

    const options = await browser.storage.sync.get();

    if (options.encryption_password === undefined) {
        return url;
    }

    const enc = new TextEncoder();

    const key = await crypto.subtle.importKey("raw", enc.encode(options.encryption_password), {
        name: "HMAC",
        hash: "SHA-256"
    }, false, ["sign"]);
    const signature = await crypto.subtle.sign({name: "HMAC"}, key, enc.encode(url));
    return btoa(String.fromCharCode(...new Uint8Array(signature)));
}

const knownUrlsCache = new Map();

async function isUrlKnown(url) {

    if (knownUrlsCache.has(url)) {
        return knownUrlsCache.get(url);
    }

    const options = await browser.storage.sync.get();

    // lol, you can await in string interpolation
    const resp = await fetch(`${options.backend_url}/faenge/has?url=${await deriveLookupUrl(url)}`, {
        headers: {
            "Content-Type": "application/json", "X-Api-Key": options.api_key
        }
    });
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
    const options = await browser.storage.sync.get();

    const resp = await fetch(`${options.backend_url}/faenge/save`, {
        method: "POST", body: JSON.stringify({
            url: await deriveLookupUrl(url),
            data: await encryptData(JSON.stringify(
                {url: url, title: title, time_created: new Date().toISOString()}
            )),
        }), headers: {"Content-Type": "application/json", "X-Api-Key": options.api_key},
    });

    if (resp.status === 200) {
        knownUrlsCache.set(url, true);
        await setToolbarButton(true);
    } else {
        console.error(resp);
    }
}

async function forgetUrl(url) {
    const options = await browser.storage.sync.get();

    const resp = await fetch(`${options.backend_url}/faenge/forget`, {
        method: "DELETE", body: JSON.stringify({
            url: await deriveLookupUrl(url),
        }), headers: {"Content-Type": "application/json", "X-Api-Key": options.api_key},
    });

    if (resp.status === 200) {
        knownUrlsCache.set(url, false);
        await setToolbarButton(false);
    } else {
        console.error(resp);
    }
}

async function getAll() {
    const options = await browser.storage.sync.get();
    const resp = await fetch(`${options.backend_url}/faenge/list`, {
        headers: {
            "Content-Type": "application/json", "X-Api-Key": options.api_key
        }
    });

    const encData = await resp.json();
    const data = await Promise.all(encData.map(async (fang) => await decryptData(fang.data)));

    return data;
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
    console.log("tab activated, checking url ", tab);
    await setToolbarButton(await isUrlKnown(tab.url));
});