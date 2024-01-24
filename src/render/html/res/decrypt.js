const KEYS = {};

function stringToBytes(string) {
    return Uint8Array.from(string, c => c.codePointAt(0));
}

function base64toBytes(base64) {
    return stringToBytes(atob(base64));
}

function createKey(key) {
    const KEY_LENGTH = 32;

    let key_bytes = new Uint8Array(KEY_LENGTH);
    for (let i = 0; i < KEY_LENGTH; i++) {
        if (i < key.length) {
            key_bytes[i] = key.codePointAt(i);
        } else {
            key_bytes[i] = 0;
        }
    }

    return crypto.subtle.importKey(
        "raw", key_bytes, "AES-GCM", false, ["decrypt"]
    );
}

async function decrypt(nonce, data, keyText) {
    const iv = base64toBytes(nonce);
    const key = await createKey(keyText);
    return await crypto.subtle.decrypt({ name: "AES-GCM", iv }, key, data);
}

async function decryptChained(el, data, tags, nonces, keys) {
    if (tags.length == 0) {
        // Once all tags have been decrypted, this should be plain text.
        return new TextDecoder().decode(data);;
    } else {
        let key = keys[tags[0]];
        if (key) {
            const nextData = await decrypt(nonces[0], data, key);
            return decryptChained(
                el, nextData, tags.slice(1), nonces.slice(1), keys
            );
        } else {
            return null;
        }
    }
}

function decryptEl(el, keys) {
    const SEP = ";";

    const cipherText = stringToBytes(atob(el.innerText));
    const tags = el.getAttribute("tags").split(SEP).reverse();
    const nonces = el.getAttribute("nonces").split(SEP).reverse();

    decryptChained(el, cipherText, tags, nonces, keys).then(plainText => {
        if (plainText) {
            el.innerText = plainText;
            el.classList.remove("secret");
        }
    });
}

function addKey(tag, key) {
    KEYS[tag] = key;

    document.querySelectorAll(".secret").forEach(el => {
        if (el.getAttribute("tags").includes(tag)) {
            decryptEl(el, KEYS);
        }
    });
}
