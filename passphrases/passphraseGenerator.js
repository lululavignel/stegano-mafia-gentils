const fs = require('fs');

/**
 * Generate a random passphrase of the specified length.
 *
 * @param {number} length - The length of the passphrase (default is 16).
 * @returns {string} - The generated passphrase.
 */
function generatePassphrase(length = 16) {
    const characters = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!"#$%&\'()*+,-./:;<=>?@[\\]^_`{|}~';
    let passphrase = '';
    for (let i = 0; i < length; i++) {
        const randomIndex = Math.floor(Math.random() * characters.length);
        passphrase += characters.charAt(randomIndex);
    }
    return passphrase;
}

/**
 * Write the given passphrase to a file.
 *
 * @param {string} filename - The name of the file to write the passphrase to.
 * @param {string} passphrase - The passphrase to write to the file.
 */
function writePassphraseToFile(filename, passphrase) {
    fs.writeFileSync(filename, passphrase, 'utf8');
}

const passphrase = generatePassphrase();
writePassphraseToFile('passphrases/serverPassphrase.txt', passphrase);
