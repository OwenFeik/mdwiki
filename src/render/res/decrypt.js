/**
 * This script handles decrypting elements and updating the page based on
 * available tag keys. Certain elements are encoded uses AES-GCM with passwords
 * configured at render time. The user can then provide the relevant passwords
 * to reveal these elements._Any tag keys provided are stored in local storage
 * and elements encrypted with those tags are automatically revealed on page]
 * load.
 */

/**
 * Separator for nonce and tag attributes. For example an encrypted element
 * might have an attribute `tags="dm;paladin;phymme"`; the separator used is
 * this constant.
 */
const SEP = ";";

/**
 * Key in local storage where added keys are stored.
 */
const LOCAL_STORAGE_KEY = "tagKeys";

/**
 * When the page loads, decrypt all elements for which we have keys available
 * in local storage.
 */
window.addEventListener("load", () => {
    decryptAvailable(loadKeys());
    setupEventListeners();
});

/**
 * Set up all event listeners at page load.
 */
function setupEventListeners() {
    // Update tag key inputs to save keys when entered.
    document.getElementById("tag-keys-menu")
        .querySelectorAll("li")
        .forEach(entry => {
            const tag = entry.querySelector(".tag-keys-label").innerText;
            const key = entry.querySelector("input");

            key.onchange = () => decryptAvailable(addKey(tag, key.value));
        });
}

/**
 * Given a string, return an array containing the bytes of the string.
 * @param {string} string To convert to byte array.
 * @returns {Uint8Array} Byte array of provided string.
 */
function stringToBytes(string) {
    return Uint8Array.from(string, c => c.codePointAt(0));
}

/**
 * Given a base64 string, decode the string and return a byte array of the
 * decoded data. 
 * @param {string} base64 Base64 encoded string.
 * @returns {Uint8Array} Byte array of decoded string.
 */
function base64toBytes(base64) {
    return stringToBytes(atob(base64));
}

/**
 * Given a string, return a 32-byte CryptoKey created by truncating the
 * provided key to 32 characters and then zero padding the end.
 * @param {string} key Password to create key from. 
 * @returns {Promise<CryptoKey>} Key created from the provided string.
 */
async function createKey(key) {
    const KEY_LENGTH = 32;

    let key_bytes = new Uint8Array(KEY_LENGTH);
    for (let i = 0; i < KEY_LENGTH; i++) {
        if (i < key.length) {
            key_bytes[i] = key.codePointAt(i);
        } else {
            key_bytes[i] = 0;
        }
    }

    return await crypto.subtle.importKey(
        "raw", key_bytes, "AES-GCM", false, ["decrypt"]
    );
}

/**
 * Decrypt the provided data using a nonce and password.
 * @param {string} nonce Base64 encoded initialisation vector.
 * @param {ArrayBuffer} data AES encrypted raw data.
 * @param {string} keyText Text of the key to use to decrypt.
 * @returns {Promise<ArrayBuffer>} Decoded data.
 */
async function decrypt(nonce, data, keyText) {
    const iv = base64toBytes(nonce);
    const key = await createKey(keyText);
    return await crypto.subtle.decrypt({ name: "AES-GCM", iv }, key, data);
}

/**
 * Given a possibly multiply encrypted block of raw data, attempt to
 * consecutively decrypt it using the provided nonces and passwords found in
 * keys for each tag.
 * @param {ArrayBuffer} data AES encrypted data to decrypt
 * @param {string[]} tags Tags to find keys for. 
 * @param {string[]} nonces Base64 encoded nonces associated with tags. 
 * @param {object} keys Map from tag to associated password. 
 * @returns {Promise<ArrayBuffer | null>} Decoded data on success, or null.
 */
async function decryptChained(data, tags, nonces, keys) {
    if (tags.length == 0) {
        // Once all tags have been decrypted, this should be plain text.
        return new TextDecoder().decode(data);;
    }

    let key = keys[tags[0]];
    if (!key) {
        // No password available for this key. Fail.
        return null;
    }

    const nextData = await decrypt(nonces[0], data, key);
    return await decryptChained(nextData, tags.slice(1), nonces.slice(1), keys);
}


/**
 * Given an element, return a list of tags for which a key is required to
 * decrypt the content of the given element.
 * @param {HTMLElement} el Element to get list of required tag keys for.
 * @return {string[]} Tags from the given element.
 */
function elTags(el) {
    const tagList = el.getAttribute("tags");
    if (tagList) {
        return tagList.split(SEP).reverse();
    } else {
        return [];
    }
}

/**
 * Render HTML to elements for insertion.
 * @param {string} html HTML to load as elements.
 * @returns {DocumentFragment} Elements interpreted from the given HTML.
 */
function htmlToElements(html) {
    return document.createRange().createContextualFragment(html);
}

/**
 * Given an element and a map from tag to encryption key, attempt to decrypt
 * the content of that element and replace the element by the HTML decrypted.
 * @param {HTMLElement} el Element to replace by it's decrypted content.
 * @param {object} keys Map from tag to decryption key.
 * @param {boolean} replace Whether to replace the element by the result. 
 * @return {Promise<boolean>} Whether the element was successfully updated.
 */
async function decryptEl(el, keys, replace = true) {
    const cipherText = stringToBytes(atob(el.innerText));
    const tags = elTags(el);
    const nonces = el.getAttribute("nonces").split(SEP).reverse();

    let plainText;
    try {
        plainText = await decryptChained(cipherText, tags, nonces, keys);
    } catch {
        plainText = null;
    }
    if (plainText) {
        if (replace) {
            return el.parentNode.replaceChild(htmlToElements(plainText), el);
        } else {
            return plainText;
        }
    } else {
        return replace ? false : null;
    }
}

/**
 * Test an entry from the tag keys menu to check that the key is correct,
 * updating the input to reflect the result.
 * 
 * @param {HTMLElement} entry Entry to test key for.
 * @param {object} keys Map from tag to tag key.
 */
async function testKey(entry, keys) {
    const tag = entry.querySelector(".tag-keys-label").innerText;
    const key = entry.querySelector("input");
    const test = entry.querySelector(".tag-keys-test");
    if (keys[tag] !== undefined) {
        key.value = keys[tag];
        const testResult = await decryptEl(test, keys, false);
        if (testResult === "correct") {
            key.disabled = true;
            key.style.backgroundColor = "";
            key.title = "Unlocked";
        } else {
            key.disabled = false;
            key.style.backgroundColor = "var(--bg3)";
            key.title = "Incorrect";
        }
    }
}

/**
 * Loads the tag key map from local storage.
 * @returns {object} Map from tag to tag key from local storage.
 */
function loadKeys() {
    return JSON.parse(localStorage.getItem(LOCAL_STORAGE_KEY)) || {};
}

/**
 * Add a key with an associated tag, then attempt to decrypt any elements we now
 * have all required keys for.
 * @param {string} tag Tag to associate the given key with.
 * @param {string} key Key to use to decode data tagged with the given tag.
 * @returns {object} Updated keys object.
 */
function addKey(tag, key) {
    let keys = loadKeys();
    keys[tag] = key;
    localStorage.setItem(LOCAL_STORAGE_KEY, JSON.stringify(keys));
    return keys;
}

/**
 * Check if each tag in tags has an associated key present in the given map.
 * @param {string[]} tags Tags to check for associated keys.
 * @param {string[]} keys Key map to check in.
 * @return {boolean} Whether all keys are available.
 */
function allKeysAvailable(tags, keys) {
    for (const tag of tags) {
        if (!(tag in keys)) {
            return false;
        }
    }
    return true;
}

/**
 * Decrypt all elements which have all keys available. Also updates the tag key
 * menu to reflect available and correct keys.
 * 
 * @param {object} keys Map from tag to tag key.
 */
function decryptAvailable(keys) {
    document.getElementById("tag-keys-menu")
        .querySelectorAll("li")
        .forEach(entry => testKey(entry, keys));

    document.querySelectorAll(".secret").forEach(el => {
        if (allKeysAvailable(elTags(el), keys)) {
            decryptEl(el, keys);
        }
    });
}
