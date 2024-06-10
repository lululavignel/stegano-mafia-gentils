/**
 * Client-side code for handling user registration functionality.
 * It establishes a socket connection with the server, validates user input,
 * encrypts sensitive data, and redirects the user based on the response.
 * Additionally, it sets cookies with user information for future sessions.
 */

$(function () {
    let socket = io();

    const registerButton = document.getElementById('registerButton');
    if (registerButton) {
        const urlParams = new URLSearchParams(window.location.search);
        const usernameParam = urlParams.get('username');

        if (usernameParam)
            document.getElementById('username').value = usernameParam;

        registerButton.addEventListener('click', () => {
            const username = document.getElementById('username').value;
            socket.emit('check_username_registered_validated', (username));
        });

        socket.on('username_unauthorized', username => {
            window.location.href = '/unauthorized.html?username=' + username;
        });

        socket.on('username_known', username => {
            window.location.href = '/connection.html?username=' + username;
        });

        socket.on('username_unknown', username => {
            const email = document.getElementById('email').value;
            socket.emit('check_email_available', { username: username, email: email });
        });

        socket.on('email_unavailable', data => {
            window.location.href = '/already_in_use.html?data=' + data.email;
        });

        socket.on('email_available', data => {
            socket.emit('check_email_validated', data);
        })

        socket.on('email_authorized', data => {
            const userData = { 
                username: data.username,
                email: data.email,
                userType: document.querySelector('input[name="userType"]:checked').value,
                password: document.getElementById('password').value
            };
            const serverPublicKey = data.serverPublicKey;
            
            const userDataRegistration = encryptConnectionData(userData, serverPublicKey);
            const hashedPassword = hashSHA256toString(userData.password);

            userKeyPair = generateKeyPair({ username: userData.username, hashedPassword: hashedPassword });
            const userPublicKey = cryptico.publicKeyString(userKeyPair);
            userDataRegistration.userPublicKey = userPublicKey;
            userDataRegistration.userType = document.querySelector('input[name="userType"]:checked').value;

            const chunkSize = 1024;

            const jsonData = JSON.stringify(userDataRegistration);
            const totalChunks = Math.ceil(jsonData.length / chunkSize);

            for (let i = 0; i < totalChunks; i++) {
                const start = i * chunkSize;
                const end = (i + 1) * chunkSize;
                const chunkData = jsonData.substring(start, end);

                socket.emit('register_user_on_server_chunk', {
                    chunkData: chunkData,
                    currentChunk: i,
                    totalChunks: totalChunks
                });
            }
        });

        socket.on('email_unauthorized', data => {
            window.location.href = '/unauthorized.html?email=' + data.email;
        });

        socket.on('user_registered_successfully', data => {
            userDecrypted = decryptConnectionData(data, userKeyPair);
            setCookie("username", userDecrypted.username, 7);
            setCookie("hashedPassword", userDecrypted.hashedPassword, 7);
            setCookie("email", userDecrypted.email, 7);
            setCookie("serverPublicKey", data.serverPublicKey, 7);
            setCookie("userType", data.userType, 7);

            window.location.href = '/chat.html';
        });
    }
});