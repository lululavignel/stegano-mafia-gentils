var username, hashedPassword, userKeyPair;

/**
 * Sets a cookie with the specified name, value, and expiration days.
 *
 * @param {string} name - Name of the cookie.
 * @param {string} value - Value to be stored in the cookie.
 * @param {number} days - Expiration days for the cookie.
 */
function setCookie(name, value, days) {
    const expirationDate = new Date();
    expirationDate.setDate(expirationDate.getDate() + days);
    const cookieValue = encodeURIComponent(value) + (days ? `; expires=${expirationDate.toUTCString()}` : '');
    // Set the cookie in the browser with the specified name, value, path, and secure flag
    document.cookie = `${name}=${cookieValue}; path=/; Secure`;
}

/**
 * Retrieves the value of the cookie with the specified name.
 *
 * @param {string} name - Name of the cookie.
 * @returns {string|null} - Value of the cookie, or null if the cookie does not exist.
 */
function getCookie(name) {
    const cookies = document.cookie.split(';');
    for (let i = 0; i < cookies.length; i++) {
        const cookie = cookies[i].trim();
        if (cookie.startsWith(name + '=')) {
            return decodeURIComponent(cookie.substring(name.length + 1));
        }
    }
    return null;
}

/**
 * Function that use CrytoJS to generate a symmetric key
 * @returns a symmetric key
 */
function generateSymmetricKey() {
    const symmetricKey = CryptoJS.lib.WordArray.random(256 / 8);

    return symmetricKey;
}

/**
 * Function to generate a RSA key pair based on username and password (passphrase creation)
 * @param {Object} data
 * @param {string} data.username - the username
 * @param {string} data.password - Hashed Password
 * @returns 
 */
function generateKeyPair(data) {
    return cryptico.generateRSAKey(data.username + '+' + data.password, 2048);
}


function hashSHA256toString(string) {
    return CryptoJS.SHA256(string).toString();
}

/**
 * Encrypts data using a symmetric key.
 * @param {string} data - The data to be encrypted.
 * @param {string} symmetricKey - The symmetric key used for encryption.
 * @returns {string} The encrypted data as a string.
 */
function encryptWithSymmetricKeyString(data, symmetricKey) {
    return CryptoJS.AES.encrypt(data, symmetricKey).toString();
}

/**
 * Encrypts data using a public key.
 * @param {string} data - The data to be encrypted.
 * @param {string} publicKey - The public key used for encryption.
 * @returns {string} The encrypted data as a string.
 */
function encryptWithPublicKeyString(data, publicKey) {
    return cryptico.encrypt(data, publicKey).cipher;
}

/**
 * Decrypts data using a symmetric key.
 * @param {string} data - The data to be decrypted.
 * @param {string} symmetricKey - The symmetric key used for decryption.
 * @returns {string} The decrypted data as a string.
 */
function decryptWithSymmetricKeyString(data, symmetricKey) {
    return CryptoJS.AES.decrypt(data, symmetricKey).toString(CryptoJS.enc.Utf8);
}

/**
 * Decrypts data using a public key.
 * @param {string} data - The data to be decrypted.
 * @param {string} publicKey - The public key used for decryption.
 * @returns {string} The decrypted data as a string.
 */
function decryptWithPublicKeyString(data, publicKey) {
    return cryptico.decrypt(data, publicKey).plaintext.toString();
}

/**
 * Encrypts user connection data using public key.
 *
 * @param {Object} data - User connection data (username, email, password).
 * @param {string} publicKey - Public key used for encryption.
 * @returns {Object} - Encrypted user connection data.
 */
function encryptConnectionData(data, publicKey) {
    const { username, email, password } = data;
    const hashedPassword = hashSHA256toString(password);
    const symmetricKeyEncrypt = generateSymmetricKey().toString();
    const encryptedHashedPassword = CryptoJS.AES.encrypt(hashedPassword, symmetricKeyEncrypt).toString();
    const encryptedEmail = CryptoJS.AES.encrypt(email, symmetricKeyEncrypt).toString();
    const encryptedSymmetricKey = cryptico.encrypt(symmetricKeyEncrypt, publicKey).cipher;

    const userDataRegistration = {
        username: username,
        email: encryptedEmail,
        encryptedHashedPassword: encryptedHashedPassword,
        encryptedSymmetricKey: encryptedSymmetricKey
    };

    return userDataRegistration;
}

/**
 * Encrypts user connection data (without hashing the password) using public key.
 *
 * @param {Object} data - User connection data (username, email, hashedPassword).
 * @param {string} publicKey - Public key used for encryption.
 * @returns {Object} - Encrypted user connection data.
 */
function encryptConnectionDataWithoutHashing(data, publicKey) {
    const { username, email, hashedPassword } = data;
    const symmetricKeyEncrypt = generateSymmetricKey().toString();
    const encryptedHashedPassword = CryptoJS.AES.encrypt(hashedPassword, symmetricKeyEncrypt).toString();
    const encryptedEmail = CryptoJS.AES.encrypt(email, symmetricKeyEncrypt).toString();
    const encryptedSymmetricKey = cryptico.encrypt(symmetricKeyEncrypt, publicKey).cipher;

    const userDataRegistration = {
        username: username,
        email: encryptedEmail,
        encryptedHashedPassword: encryptedHashedPassword,
        encryptedSymmetricKey: encryptedSymmetricKey
    };

    return userDataRegistration;
}

/**
 * Decrypts user connection data using private key.
 *
 * @param {Object} data - Encrypted user connection data (username, email, encryptedHashedPassword, encryptedSymmetricKey).
 * @param {string} privateKey - Private key used for decryption.
 * @returns {Object} - Decrypted user connection data.
 */
function decryptConnectionData(data, privateKey) {
    const { username, email, encryptedHashedPassword, encryptedSymmetricKey } = data;
    const decryptedSymmetricKey = cryptico.decrypt(encryptedSymmetricKey, privateKey).plaintext.toString();
    const decryptedHashedPassword = CryptoJS.AES.decrypt(encryptedHashedPassword, decryptedSymmetricKey).toString(CryptoJS.enc.Utf8);
    const decryptedEmail = CryptoJS.AES.decrypt(email, decryptedSymmetricKey).toString(CryptoJS.enc.Utf8);

    const userData = {
        username: username,
        email: decryptedEmail,
        hashedPassword: decryptedHashedPassword
    };

    return userData;
}

$(function () {

});